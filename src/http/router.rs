use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

use crate::{app_state::AppState, http::handlers};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::health::root))
        .route("/health", get(handlers::health::health))
        .route("/events", get(handlers::events::sse_events))
        .route(
            "/av",
            get(handlers::av::list_avs).post(handlers::av::create_av),
        )
        .route(
            "/av/{id}/config",
            get(handlers::config::get_av_config)
                .put(handlers::config::put_av_config)
                .delete(handlers::config::delete_av_config)
                .layer(DefaultBodyLimit::max(16 * 1024 * 1024)),
        )
        .route(
            "/map",
            get(handlers::map::list_maps).post(handlers::map::create_map),
        )
        .route("/map/{id}/file", get(handlers::map_file::list_files))
        .route(
            "/map/{id}/file/{*relative_path}",
            get(handlers::map_file::get_file)
                .put(handlers::map_file::put_file)
                .delete(handlers::map_file::delete_file)
                .layer(DefaultBodyLimit::max(256 * 1024 * 1024)),
        )
        .route(
            "/plan",
            get(handlers::plan::list_plans).post(handlers::plan::create_plan),
        )
        .route(
            "/sampler",
            get(handlers::sampler::list_samplers).post(handlers::sampler::create_sampler),
        )
        .route(
            "/sampler/{id}/config",
            get(handlers::config::get_sampler_config)
                .put(handlers::config::put_sampler_config)
                .delete(handlers::config::delete_sampler_config)
                .layer(DefaultBodyLimit::max(16 * 1024 * 1024)),
        )
        .route(
            "/scenario",
            get(handlers::scenario::list_scenarios).post(handlers::scenario::create_scenario),
        )
        .route(
            "/scenario/{id}/file",
            get(handlers::scenario_file::list_files),
        )
        .route(
            "/scenario/{id}/file/{*relative_path}",
            get(handlers::scenario_file::get_file)
                .put(handlers::scenario_file::put_file)
                .delete(handlers::scenario_file::delete_file)
                .layer(DefaultBodyLimit::max(256 * 1024 * 1024)),
        )
        .route(
            "/simulator",
            get(handlers::simulator::list_simulators).post(handlers::simulator::create_simulator),
        )
        .route(
            "/simulator/{id}/config",
            get(handlers::config::get_simulator_config)
                .put(handlers::config::put_simulator_config)
                .delete(handlers::config::delete_simulator_config)
                .layer(DefaultBodyLimit::max(16 * 1024 * 1024)),
        )
        .route(
            "/task",
            get(handlers::task::list_tasks).post(handlers::task::create_task),
        )
        .route(
            "/executor",
            get(handlers::executor::list_executors).post(handlers::executor::create_executor),
        )
        .route("/task/claim", post(handlers::task::claim_task))
        .route("/task/failed", post(handlers::task::task_failed))
        .route("/task/succeeded", post(handlers::task::task_completed))
        .route("/task/aborted", post(handlers::task::task_aborted))
        .route(
            "/task_run/{id}/log/append",
            post(handlers::log_stream::append_log).layer(DefaultBodyLimit::max(4 * 1024 * 1024)),
        )
        .route(
            "/scenario/upload",
            post(handlers::upload::upload_scenarios)
                .layer(DefaultBodyLimit::max(512 * 1024 * 1024)),
        )
        .with_state(state)
}
