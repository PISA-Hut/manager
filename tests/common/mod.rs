//! Shared test harness: spin a Postgres container, run migrations,
//! return an `axum_test::TestServer` wired to the manager router.
//!
//! Each test owns its own container so they can run in parallel and
//! never share schema state.

use axum_test::TestServer;
use manager::{app_state::AppState, db::connect, db::migrate, events, http::router::create_router};
use sea_orm::DatabaseConnection;
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

pub struct TestApp {
    pub server: TestServer,
    pub db: DatabaseConnection,
    /// Holding the container handle keeps Postgres alive for the
    /// duration of the test. Drop it and the container is removed.
    _container: ContainerAsync<Postgres>,
}

/// Bring up a fresh Postgres + migrations + AppState + axum server.
/// Returns a TestApp the test can `.post(...)` / `.get(...)` against.
pub async fn spawn_test_app() -> TestApp {
    // The PostgREST-permission migration reads `AUTHENTICATOR_PASSWORD`
    // from env at run-time. Stub it for tests so each `migrate()` call
    // can create the `authenticator` role. Safety: the value is the
    // same across all tests so the racy set is benign.
    unsafe {
        std::env::set_var("AUTHENTICATOR_PASSWORD", "test-authenticator-password");
    }

    let container = Postgres::default()
        .start()
        .await
        .expect("start postgres container");
    let host = container.get_host().await.expect("container host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("container port");
    let database_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let db = connect(&database_url).await;
    migrate(&db).await.expect("run migrations");

    let (events_tx, _events_rx) = events::channel();

    let state = AppState {
        db: db.clone(),
        events_tx,
        useless_streak_limit: 10,
    };

    let app = create_router(state);
    let server = TestServer::new(app).expect("build TestServer");

    TestApp {
        server,
        db,
        _container: container,
    }
}
