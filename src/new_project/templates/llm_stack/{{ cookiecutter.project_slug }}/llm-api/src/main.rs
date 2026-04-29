mod agent;
mod config;
mod error;
mod mcp;
mod memory;
mod models;
mod providers;
mod state;
mod tools;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use config::Config;
use error::AppError;
use models::ChatCompletionRequest;
use providers::ProviderClient;
use state::AppState;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=info".to_owned()),
        )
        .init();

    let config = Config::from_env()?;
    let provider = ProviderClient::new(config.clone())?;
    let state = AppState::new(config, provider).await?;

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/tools", get(list_tools))
        .route("/v1/mcp/servers", get(list_mcp_servers))
        .route("/v1/memory/{session_id}", get(get_memory).delete(clear_memory))
        .route("/v1/chat/completions", post(chat_completions))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    info!("listening on {}", state.config.bind_address);

    let listener = tokio::net::TcpListener::bind(state.config.bind_address)
        .await
        .map_err(|err| AppError::Startup(err.to_string()))?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|err| AppError::Startup(err.to_string()))?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "provider": state.config.provider_name(),
            "model": state.config.model,
        })),
    )
}

async fn list_tools(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(state.tools.descriptions()))
}

async fn list_mcp_servers(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(state.tools.mcp_servers()))
}

async fn get_memory(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let snapshot = state.memory.snapshot(&session_id).await;
    (StatusCode::OK, Json(snapshot))
}

async fn clear_memory(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    state.memory.clear(&session_id).await;
    (
        StatusCode::OK,
        Json(serde_json::json!({ "cleared": true, "session_id": session_id })),
    )
}

async fn chat_completions(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Response, AppError> {
    let result = agent::run_agent(&state.provider, &state.memory, &state.tools, &request).await?;

    if request.stream.unwrap_or(false) {
        Ok(models::streaming_response(result.model, result.message))
    } else {
        Ok(models::json_response(result.model, result.message, result.usage))
    }
}
