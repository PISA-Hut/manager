//! HTTP-level integration tests for the task lifecycle endpoints.

mod common;

use axum::http::StatusCode;
use serde_json::json;

use crate::common::spawn_test_app;

#[tokio::test]
async fn task_failed_rejects_negative_concrete_count() {
    let app = spawn_test_app().await;

    // Validation should fire before any DB lookup, so we don't need
    // to seed a task for this test.
    let resp = app
        .server
        .post("/task/failed")
        .json(&json!({
            "task_id": 1,
            "reason": "smoke",
            "concrete_scenarios_executed": -1,
        }))
        .await;

    resp.assert_status(StatusCode::BAD_REQUEST);
    assert!(
        resp.text().contains("concrete_scenarios_executed"),
        "expected error body to mention the field name; got: {}",
        resp.text()
    );
}

#[tokio::test]
async fn task_claim_with_no_pending_returns_null() {
    let app = spawn_test_app().await;

    // First we need an executor row so the claim endpoint doesn't
    // 404 with "worker not found". Insert one via PostgREST-style
    // raw SQL since we don't have HTTP CRUD for executors directly
    // exposed.
    use sea_orm::{ConnectionTrait, Statement};
    app.db
        .execute(Statement::from_string(
            sea_orm::DbBackend::Postgres,
            "INSERT INTO executor \
                 (slurm_job_id, slurm_array_id, slurm_node_list, hostname) \
             VALUES (1, 0, 'node-0', 'test-host')",
        ))
        .await
        .expect("seed executor");

    let resp = app
        .server
        .post("/task/claim")
        .json(&json!({ "executor_id": 1 }))
        .await;

    resp.assert_status_ok();
    // No pending tasks exist → manager returns JSON null.
    assert_eq!(resp.text(), "null");
}

#[tokio::test]
async fn health_endpoint_returns_200() {
    let app = spawn_test_app().await;
    let resp = app.server.get("/health").await;
    resp.assert_status_ok();
}
