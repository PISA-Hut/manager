use axum::{
    Router,
    routing::{get, post},
};

use crate::{app_state::AppState, http::handlers};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::health::root))
        .route("/health", get(handlers::health::health))
        .route(
            "/av",
            get(handlers::av::list_avs).post(handlers::av::create_av),
        )
        .route(
            "/map",
            get(handlers::map::list_maps).post(handlers::map::create_map),
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
            "/scenario",
            get(handlers::scenario::list_scenarios).post(handlers::scenario::create_scenario),
        )
        .route(
            "/simulator",
            get(handlers::simulator::list_simulators).post(handlers::simulator::create_simulator),
        )
        .route(
            "/task",
            get(handlers::task::list_tasks).post(handlers::task::create_task),
        )
        .route(
            "/worker",
            get(handlers::worker::list_workers).post(handlers::worker::create_worker),
        )
        .route("/task/claim", post(handlers::task::claim_task))
        .route("/task/failed", post(handlers::task::task_failed))
        .route("/task/succeeded", post(handlers::task::task_succeeded))
        .with_state(state)
}
