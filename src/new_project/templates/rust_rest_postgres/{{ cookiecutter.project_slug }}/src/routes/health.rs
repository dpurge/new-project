use axum::{extract::State, Json};

use crate::{
    models::HealthResponse,
    state::AppState,
};

pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        service: state.config.app_name,
        status: "ok",
    })
}
