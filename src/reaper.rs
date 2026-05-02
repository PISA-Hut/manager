//! Periodically mark task_run rows that have gone silent as `aborted`.
//!
//! Executors send a log-append PUT every ~1 s while alive; the handler
//! bumps `last_heartbeat_at`. A run that hasn't heart-beat in
//! `STALE_AFTER` seconds is presumed dead (node crash, SIGKILL, network
//! partition) and the reaper flips the row. Parent task is pulled back to
//! `queued` so the scheduler can redispatch on the next poll.

use std::time::Duration;

use sea_orm::DatabaseConnection;
use tracing::{info, warn};

/// Runs older than this without a log-append heartbeat get reaped. Needs
/// to be comfortably larger than the executor's streamer interval (~1 s)
/// plus any stop-the-world pauses a live run might have (CARLA cold
/// start, gRPC deadline, the PUT's own network round-trip).
///
/// Tunable at deploy time via `REAPER_STALE_SECS`. Default 300 s —
/// generous enough to survive the worst legitimate silence we've
/// observed while still catching a real crash within a few minutes.
const DEFAULT_STALE_AFTER_SECS: i64 = 300;
const DEFAULT_POLL_INTERVAL_SECS: u64 = 30;

fn read_env_secs(name: &str, default: i64) -> i64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default)
}

pub fn spawn(db: DatabaseConnection) {
    let stale_after = read_env_secs("REAPER_STALE_SECS", DEFAULT_STALE_AFTER_SECS);
    let poll_interval = Duration::from_secs(read_env_secs(
        "REAPER_INTERVAL_SECS",
        DEFAULT_POLL_INTERVAL_SECS as i64,
    ) as u64);
    info!(
        "reaper: spawning with stale_after={}s, poll_interval={:?}",
        stale_after, poll_interval
    );
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(poll_interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        // Skip the first immediate tick — no point reaping before any
        // executor has had a chance to register.
        ticker.tick().await;

        loop {
            ticker.tick().await;
            match crate::db::task_run::reap_stale_runs(&db, stale_after).await {
                Ok(ids) if ids.is_empty() => {}
                Ok(ids) => {
                    info!(
                        "reaper: aborted {} stale task_run(s) with no heartbeat in {}s: {:?}",
                        ids.len(),
                        stale_after,
                        ids
                    );
                }
                Err(e) => {
                    warn!("reaper: scan failed: {e}");
                }
            }
        }
    });
}
