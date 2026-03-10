mod app_state;
mod db;
mod entity;
mod http;
mod migrator;
mod service;

use crate::app_state::AppState;

use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    dotenv::dotenv().ok();

    let db = db::connect(&std::env::var("DATABASE_URL").expect("DATABSE_URL")).await;
    db::migrate(&db).await.unwrap();

    let state = AppState { db };

    let app = http::router::create_router(state);

    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid u16");

    info!("Starting server on {}:{}", bind_address, port);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", bind_address, port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
