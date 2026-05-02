use axum::{
    Json,
    extract::{Multipart, State},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;

use crate::app_state::AppState;
use crate::db;
use crate::entity::scenario_file;
use crate::entity::sea_orm_active_enums::ScenarioFormat;
use crate::http::AppError;
use crate::http::handlers::bytes::sha256_hex;

#[derive(Deserialize)]
struct SpecYaml {
    scenario_name: Option<String>,
    map_name: Option<String>,
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

/// Parse a zip entry into `(scenario_folder, relative_path_within_folder)`.
///
/// If `wrapper` is Some, strip that leading component first — this lets the
/// caller handle zips that wrap every scenario in an extra top-level dir
/// (e.g. `00-2/<scenario>/…`). The scenario folder is the first component
/// after the wrapper (if any); the relative path preserves any subdirs
/// inside the scenario folder (e.g. `Catalogs/Vehicles.xosc`).
fn parse_zip_entry(path: &std::path::Path, wrapper: Option<&str>) -> Option<(String, String)> {
    let components: Vec<&str> = path.iter().filter_map(|c| c.to_str()).collect();
    let stripped: &[&str] = match wrapper {
        Some(w) => {
            let first = components.first()?;
            if *first != w {
                return None;
            }
            &components[1..]
        }
        None => &components[..],
    };
    match stripped {
        [folder, rest @ ..] if !rest.is_empty() => Some((folder.to_string(), rest.join("/"))),
        _ => None,
    }
}

/// If every entry in the zip shares a single leading directory component,
/// return it so the caller can strip it. A zip laid out as
/// `foo/scenarioA/…`, `foo/scenarioB/…` gets `Some("foo")`; a flat
/// `scenarioA/…`, `scenarioB/…` gets `None`.
fn detect_wrapper_dir(entry_paths: &[std::path::PathBuf]) -> Option<String> {
    let mut candidate: Option<String> = None;
    for p in entry_paths {
        let comps: Vec<&str> = p.iter().filter_map(|c| c.to_str()).collect();
        if comps.len() < 2 {
            return None;
        }
        match &candidate {
            None => candidate = Some(comps[0].to_string()),
            Some(c) if c == comps[0] => {}
            _ => return None,
        }
    }
    candidate
}

pub async fn upload_scenarios(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResult>, AppError> {
    let mut zip_bytes: Option<Vec<u8>> = None;
    let mut format = ScenarioFormat::OpenScenario1;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                zip_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::bad_request(format!("Failed to read file: {e}")))?
                        .to_vec(),
                );
            }
            "format" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::bad_request(format!("Failed to read format: {e}")))?;
                format = match text.as_str() {
                    "open_scenario1" => ScenarioFormat::OpenScenario1,
                    "open_scenario2" => ScenarioFormat::OpenScenario2,
                    "carla_lb_route" => ScenarioFormat::CarlaLbRoute,
                    _ => {
                        return Err(AppError::bad_request(format!("Unknown format: {text}")));
                    }
                };
            }
            _ => {}
        }
    }

    let zip_bytes = zip_bytes.ok_or_else(|| AppError::bad_request("No file uploaded"))?;

    // Pass 0: enumerate entry paths so we can detect an optional top-level
    // wrapper dir (`00-2/<scenario>/…` vs. `<scenario>/…`).
    let entry_paths: Vec<std::path::PathBuf> = {
        let cursor = std::io::Cursor::new(&zip_bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| AppError::bad_request(format!("Invalid zip file: {e}")))?;
        (0..archive.len())
            .filter_map(|i| {
                let f = archive.by_index(i).ok()?;
                if f.is_dir() {
                    return None;
                }
                f.enclosed_name().map(|p| p.to_owned())
            })
            .collect()
    };
    let wrapper = detect_wrapper_dir(&entry_paths);

    // Collect spec.yaml + every file's bytes per scenario folder.
    let mut specs: HashMap<String, SpecYaml> = HashMap::new();
    let mut scenario_files: HashMap<String, Vec<(String, Vec<u8>)>> = HashMap::new();

    {
        let cursor = std::io::Cursor::new(&zip_bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| AppError::bad_request(format!("Invalid zip file: {e}")))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| AppError::bad_request(format!("Zip read error: {e}")))?;

            if file.is_dir() {
                continue;
            }

            let path = match file.enclosed_name() {
                Some(p) => p.to_owned(),
                None => continue,
            };

            let (folder_name, rel_path) = match parse_zip_entry(&path, wrapper.as_deref()) {
                Some(v) => v,
                None => continue,
            };

            let mut contents = Vec::new();
            file.read_to_end(&mut contents).map_err(|e| {
                AppError::bad_request(format!("Failed to read {rel_path} in {folder_name}: {e}"))
            })?;

            if rel_path == "spec.yaml" {
                let spec: SpecYaml = serde_yaml::from_slice(&contents).map_err(|e| {
                    AppError::bad_request(format!(
                        "Failed to parse spec.yaml in {folder_name}: {e}"
                    ))
                })?;
                specs.insert(folder_name.clone(), spec);
            }

            scenario_files
                .entry(folder_name)
                .or_default()
                .push((rel_path, contents));
        }
    }

    let mut results = Vec::new();

    for (folder_name, spec) in &specs {
        let scenario_name = spec
            .scenario_name
            .as_deref()
            .unwrap_or(folder_name.as_str());

        // Move the (rel_path, contents) Vec out of the map so the
        // per-file loop below can hand ownership of the bytes straight
        // to db::scenario_file::put without cloning. With a 512 MB
        // body limit, cloning each file before insert was a
        // double-memory hazard.
        let files = match scenario_files.remove(folder_name) {
            Some(v) => v,
            None => {
                results.push(ScenarioUploadResult {
                    name: scenario_name.to_string(),
                    status: "skipped".to_string(),
                    message: Some("No files in folder".to_string()),
                });
                continue;
            }
        };

        let has_xosc = files.iter().any(|(p, _)| p.ends_with(".xosc"));
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

        let scenario_id =
            match db::scenario::create(&state.db, format.clone(), Some(scenario_name.to_string()))
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

        let mut file_error: Option<String> = None;
        for (rel_path, contents) in files {
            let sha = sha256_hex(&contents);
            let store = <scenario_file::Model as crate::db::FileStore>::put(
                &state.db,
                scenario_id,
                rel_path,
                contents,
                sha,
            )
            .await
            .map(|_| ());
            if let Err(e) = store {
                file_error = Some(format!("Failed to store file: {e}"));
                break;
            }
        }

        if let Some(msg) = file_error {
            results.push(ScenarioUploadResult {
                name: scenario_name.to_string(),
                status: "error".to_string(),
                message: Some(msg),
            });
            continue;
        }

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

        results.push(ScenarioUploadResult {
            name: scenario_name.to_string(),
            status: "created".to_string(),
            message: None,
        });
    }

    let total = results.len();
    Ok(Json(UploadResult { total, results }))
}
