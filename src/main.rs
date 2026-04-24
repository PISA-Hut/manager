mod app_state;
mod db;
mod entity;
mod events;
mod http;
mod migrator;
mod reaper;
mod service;

use crate::app_state::AppState;

use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = db::connect(&database_url).await;
    db::migrate(&db).await.unwrap();

    let (events_tx, _events_rx) = events::channel();
    events::spawn_listener(database_url, events_tx.clone());
    reaper::spawn(db.clone());

    let useless_streak_limit: usize = std::env::var("USELESS_STREAK_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|n| *n >= 1)
        .unwrap_or(10);
    info!(
        "Permanent-fail after {} consecutive useless task_runs",
        useless_streak_limit
    );

    let state = AppState {
        db,
        events_tx,
        useless_streak_limit,
    };

    let app = http::router::create_router(state);

    let bind_address = std::env::var("MANAGER_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0".to_string());

    let port: u16 = std::env::var("MANAGER_PORT")
        .unwrap_or_else(|_| "9000".to_string())
        .parse()
        .expect("PORT must be a valid u16");

    info!("Starting server on {}:{}", bind_address, port);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", bind_address, port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
