use axum::{Json, extract::{Multipart, State}, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::app_state::AppState;
use crate::db;
use crate::entity::sea_orm_active_enums::ScenarioFormat;

#[derive(Deserialize)]
struct SpecYaml {
    scenario_name: Option<String>,
    map_name: Option<String>,
    ego: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct UploadResult {
    pub total: usize,
    pub results: Vec<ScenarioUploadResult>,
}

#[derive(Serialize)]
pub struct ScenarioUploadResult {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
}

/// Resolve the scenario folder name and file name from a zip entry path.
/// Supports "scenario_name/file" and "wrapper/scenario_name/file".
fn parse_zip_entry(path: &std::path::Path) -> Option<(String, String)> {
    let components: Vec<&str> = path.iter().filter_map(|c| c.to_str()).collect();
    match components.as_slice() {
        [folder, file] => Some((folder.to_string(), file.to_string())),
        [_, folder, file] => Some((folder.to_string(), file.to_string())),
        _ => None,
    }
}

pub async fn upload_scenarios(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResult>, (StatusCode, String)> {
    let mut zip_bytes: Option<Vec<u8>> = None;
    let mut format = ScenarioFormat::OpenScenario1;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                zip_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read file: {e}")))?
                        .to_vec(),
                );
            }
            "format" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read format: {e}")))?;
                format = match text.as_str() {
                    "open_scenario1" => ScenarioFormat::OpenScenario1,
                    "open_scenario2" => ScenarioFormat::OpenScenario2,
                    "carla_lb_route" => ScenarioFormat::CarlaLbRoute,
                    _ => {
                        return Err((StatusCode::BAD_REQUEST, format!("Unknown format: {text}")));
                    }
                };
            }
            _ => {}
        }
    }

    let zip_bytes = zip_bytes.ok_or((StatusCode::BAD_REQUEST, "No file uploaded".to_string()))?;

    // Pass 1: collect spec.yaml and file lists per scenario folder
    let mut specs: HashMap<String, SpecYaml> = HashMap::new();
    let mut scenario_files: HashMap<String, Vec<String>> = HashMap::new();

    {
        let cursor = std::io::Cursor::new(&zip_bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid zip file: {e}")))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Zip read error: {e}")))?;

            if file.is_dir() {
                continue;
            }

            let path = match file.enclosed_name() {
                Some(p) => p.to_owned(),
                None => continue,
            };

            let (folder_name, file_name) = match parse_zip_entry(&path) {
                Some(v) => v,
                None => continue,
            };

            scenario_files
                .entry(folder_name.clone())
                .or_default()
                .push(file_name.clone());

            if file_name == "spec.yaml" {
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read spec.yaml in {folder_name}: {e}")))?;

                let spec: SpecYaml = serde_yaml::from_str(&contents)
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to parse spec.yaml in {folder_name}: {e}")))?;

                specs.insert(folder_name, spec);
            }
        }
    }

    // Process scenarios: create DB records
    let mut results = Vec::new();
    // Track which scenario folders were successfully created so we can extract their files
    let mut created_scenarios: HashMap<String, String> = HashMap::new(); // folder_name -> scenario_name

    for (folder_name, spec) in &specs {
        let scenario_name = spec
            .scenario_name
            .as_deref()
            .unwrap_or(folder_name.as_str());

        let files = scenario_files.get(folder_name).cloned().unwrap_or_default();
        let has_xosc = files.iter().any(|f| f.ends_with(".xosc"));
        if !has_xosc {
            results.push(ScenarioUploadResult {
                name: scenario_name.to_string(),
                status: "skipped".to_string(),
                message: Some("No .xosc file found".to_string()),
            });
            continue;
        }

        let map_id = if let Some(map_name) = &spec.map_name {
            match db::map::find_by_name(&state.db, map_name).await {
                Ok(Some(m)) => Some(m.id),
                Ok(None) => {
                    results.push(ScenarioUploadResult {
                        name: scenario_name.to_string(),
                        status: "error".to_string(),
                        message: Some(format!("Map '{map_name}' not found in database")),
                    });
                    continue;
                }
                Err(e) => {
                    results.push(ScenarioUploadResult {
                        name: scenario_name.to_string(),
                        status: "error".to_string(),
                        message: Some(format!("DB error looking up map: {e}")),
                    });
                    continue;
                }
            }
        } else {
            None
        };

        let goal_config = spec.ego.clone().unwrap_or(serde_json::Value::Null);
        let scenario_path = format!("scenario/{scenario_name}");

        let scenario_id = match db::scenario::create(
            &state.db,
            format.clone(),
            Some(scenario_name.to_string()),
            scenario_path,
            goal_config,
        )
        .await
        {
            Ok(s) => s.id,
            Err(e) => {
                results.push(ScenarioUploadResult {
                    name: scenario_name.to_string(),
                    status: "error".to_string(),
                    message: Some(format!("Failed to create scenario: {e}")),
                });
                continue;
            }
        };

        if let Some(mid) = map_id {
            let plan_name = format!(
                "{}-{scenario_name}",
                spec.map_name.as_deref().unwrap_or("unknown")
            );
            if let Err(e) = db::plan::create(&state.db, plan_name, mid, scenario_id).await {
                results.push(ScenarioUploadResult {
                    name: scenario_name.to_string(),
                    status: "error".to_string(),
                    message: Some(format!("Scenario created but plan failed: {e}")),
                });
                continue;
            }
        }

        created_scenarios.insert(folder_name.clone(), scenario_name.to_string());
    }

    // Pass 2: extract .xosc files for successfully created scenarios
    {
        let cursor = std::io::Cursor::new(&zip_bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Zip re-read error: {e}")))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Zip read error: {e}")))?;

            if file.is_dir() {
                continue;
            }

            let path = match file.enclosed_name() {
                Some(p) => p.to_owned(),
                None => continue,
            };

            let (folder_name, file_name) = match parse_zip_entry(&path) {
                Some(v) => v,
                None => continue,
            };

            if !file_name.ends_with(".xosc") {
                continue;
            }

            let scenario_name = match created_scenarios.get(&folder_name) {
                Some(name) => name,
                None => continue,
            };

            let target_dir = Path::new(&state.scenario_storage_dir)
                .join("scenario")
                .join(scenario_name);

            if let Err(e) = std::fs::create_dir_all(&target_dir) {
                results.push(ScenarioUploadResult {
                    name: scenario_name.clone(),
                    status: "error".to_string(),
                    message: Some(format!("Failed to create directory: {e}")),
                });
                created_scenarios.remove(&folder_name);
                continue;
            }

            let mut contents = Vec::new();
            if let Err(e) = file.read_to_end(&mut contents) {
                results.push(ScenarioUploadResult {
                    name: scenario_name.clone(),
                    status: "error".to_string(),
                    message: Some(format!("Failed to read {file_name}: {e}")),
                });
                created_scenarios.remove(&folder_name);
                continue;
            }

            let dest = target_dir.join(&file_name);
            if let Err(e) = std::fs::write(&dest, &contents) {
                results.push(ScenarioUploadResult {
                    name: scenario_name.clone(),
                    status: "error".to_string(),
                    message: Some(format!("Failed to write {file_name}: {e}")),
                });
                created_scenarios.remove(&folder_name);
                continue;
            }
        }
    }

    // Add success results for scenarios that made it through both passes
    for scenario_name in created_scenarios.values() {
        results.push(ScenarioUploadResult {
            name: scenario_name.clone(),
            status: "created".to_string(),
            message: None,
        });
    }

    let total = results.len();
    Ok(Json(UploadResult { total, results }))
}
