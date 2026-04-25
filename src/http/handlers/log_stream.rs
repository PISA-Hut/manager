use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
};
use sea_orm::EntityTrait;
use serde_json::json;

use crate::{
    app_state::AppState,
    db,
    entity::{sea_orm_active_enums::TaskRunStatus, task_run::Entity as TaskRun},
};

/// Append a stdout/stderr chunk to a task_run row. Called by the executor
/// every ~1 s (or whenever its log buffer has fresh bytes) while the run
/// is in progress.
///
/// Returns 410 Gone once the task_run has been finalised (e.g. user hit
/// Stop in the web UI). The executor treats that as its cue to abort and
/// exit cleanly, since the log-stream tick is the only out-of-band signal
/// it ever asks the manager for.
///
/// Two side effects on the happy path: update `task_run.log` via an
/// append-only SQL (no racing with lifecycle calls), and broadcast a
/// `log` SSE envelope so the Log Drawer can stream chunks into the UI.
pub async fn append_log(
    State(state): State<AppState>,
    Path(run_id): Path<i32>,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    if body.is_empty() {
        return Ok(StatusCode::NO_CONTENT);
    }

    let row = TaskRun::find_by_id(run_id)
        .one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "task_run not found".to_string()))?;
    if row.task_run_status != TaskRunStatus::Running {
        return Err((
            StatusCode::GONE,
            format!("task_run {run_id} is no longer running"),
        ));
    }

    let chunk = std::str::from_utf8(&body)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("utf-8 required: {e}")))?;
    let end_offset = db::task_run::append_log(&state.db, run_id, chunk)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // `end_offset` = octet_length(task_run.log) after the append. Each
    // chunk's [start, end] is therefore (end_offset - utf8_bytes(chunk),
    // end_offset]. The Log Drawer uses this to dedupe chunks that
    // arrive over SSE while it's still fetching the initial snapshot.
    let envelope = json!({
        "kind": "log",
        "task_run_id": run_id,
        "chunk": chunk,
        "end_offset": end_offset,
    });
    let _ = state.events_tx.send(envelope.to_string());

    Ok(StatusCode::NO_CONTENT)
}
