mod config;
mod error;
mod jsonrpc;
mod prompts;
mod resources;
mod server;
mod tools;

use axum::{
    extract::State,
    http::HeaderMap,
    response::Response,
    routing::{get, post},
    Router,
};
use config::{Config, TransportMode};
use error::AppError;
use server::AppState;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    signal,
};
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
    let state = AppState::new(config.allowed_origins.clone(), config.auth_token.clone());

    match config.transport {
        TransportMode::Stdio => run_stdio(state).await,
        TransportMode::StreamableHttp | TransportMode::Http | TransportMode::All => {
            run_http(config, state).await
        }
    }
}

async fn run_http(config: Config, state: AppState) -> Result<(), AppError> {
    let app = Router::new()
        .route("/mcp", get(server::streamable_get).post(streamable_post))
        .route("/sse", get(server::legacy_sse))
        .route("/messages", post(server::legacy_post))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    info!("listening on {}", config.bind_address);

    let listener = tokio::net::TcpListener::bind(config.bind_address)
        .await
        .map_err(|err| AppError::Startup(err.to_string()))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|err| AppError::Startup(err.to_string()))?;
    Ok(())
}

async fn streamable_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<Response, AppError> {
    let session_id = headers
        .get("mcp-session-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);

    server::handle_http_message(State(state), headers, session_id, body).await
}

async fn run_stdio(state: AppState) -> Result<(), AppError> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin).lines();
    let mut writer = stdout;

    while let Some(line) = reader
        .next_line()
        .await
        .map_err(|err| AppError::Transport(err.to_string()))?
    {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let incoming = serde_json::from_str(trimmed)?;
        let response = server::dispatch_incoming(&state, incoming, Some("stdio".to_owned())).await?;

        if !response.is_null() {
            writer
                .write_all(serde_json::to_string(&response)?.as_bytes())
                .await
                .map_err(|err| AppError::Transport(err.to_string()))?;
            writer
                .write_all(b"\n")
                .await
                .map_err(|err| AppError::Transport(err.to_string()))?;
            writer
                .flush()
                .await
                .map_err(|err| AppError::Transport(err.to_string()))?;
        }
    }

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
}
