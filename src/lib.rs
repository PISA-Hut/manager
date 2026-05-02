//! Public surface so integration tests under `tests/` can import the
//! manager's modules. The actual binary entrypoint lives in `main.rs`
//! and re-uses these.

pub mod app_state;
pub mod db;
pub mod entity;
pub mod events;
pub mod http;
pub mod migrator;
pub mod reaper;
pub mod service;
