use sea_orm::DatabaseConnection;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    /// Fan-out channel for `pisa_events` NOTIFY payloads. Each SSE
    /// subscriber holds its own receiver; the single PgListener task
    /// (see `events::spawn_listener`) is the sole producer.
    pub events_tx: broadcast::Sender<String>,
    /// Number of consecutive useless task_runs (those that finished
    /// zero concrete scenarios) that permanently fail a task.
    /// Sourced from `USELESS_STREAK_LIMIT` at startup; defaults to 10.
    pub useless_streak_limit: usize,
}
