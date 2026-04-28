pub mod health;
pub mod todos;

use axum::{
    routing::get,
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::health_check))
        .route("/api/todos", get(todos::list_todos).post(todos::create_todo))
        .route("/api/todos/{id}", get(todos::get_todo))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
