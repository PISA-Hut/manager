use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
};
use sea_orm::{DbErr, EntityTrait};
use serde_json::json;

use crate::{
    app_state::AppState,
    db,
    entity::{sea_orm_active_enums::TaskRunStatus, task_run::Entity as TaskRun},
    http::AppError,
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
) -> Result<StatusCode, AppError> {
    if body.is_empty() {
        return Ok(StatusCode::NO_CONTENT);
    }

    let row = TaskRun::find_by_id(run_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("task_run not found"))?;
    if row.task_run_status != TaskRunStatus::Running {
        return Err(AppError::gone(format!(
            "task_run {run_id} is no longer running"
        )));
    }

    let chunk = std::str::from_utf8(&body)
        .map_err(|e| AppError::bad_request(format!("utf-8 required: {e}")))?;
    // The pre-flight status check above is racy: between the SELECT and
    // the UPDATE, the run can be finalised by Stop/abort/reaper. The
    // append_log SQL is gated on `task_run_status = 'running'`, so a
    // racing finalisation surfaces as RecordNotFound here — map it to
    // the same 410 Gone the explicit pre-check returns.
    let end_offset = db::task_run::append_log(&state.db, run_id, chunk)
        .await
        .map_err(|e| match e {
            DbErr::RecordNotFound(_) => {
                AppError::gone(format!("task_run {run_id} is no longer running"))
            }
            other => AppError::from(other),
        })?;

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
