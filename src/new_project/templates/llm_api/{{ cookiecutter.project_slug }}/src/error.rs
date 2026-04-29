use std::net::AddrParseError;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("missing required environment variable: {0}")]
    MissingEnv(String),
    #[error("{message}")]
    Config {
        message: String,
        source: AddrParseError,
    },
    #[error("unsupported provider: {0}")]
    InvalidProvider(String),
    #[error("invalid integer for {name}: {value}")]
    InvalidInteger { name: String, value: String },
    #[error("server startup failed: {0}")]
    Startup(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("mcp integration failed: {0}")]
    Mcp(String),
    #[error("tool execution failed: {0}")]
    Tool(String),
    #[error("agent loop exceeded the configured limit of {0} steps")]
    AgentLoopExceeded(usize),
    #[error("failed to reach upstream provider: {0}")]
    Upstream(#[from] reqwest::Error),
    #[error("failed to serialize or parse JSON: {0}")]
    Json(#[from] serde_json::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::InvalidRequest(_) | Self::Tool(_) => StatusCode::BAD_REQUEST,
            Self::Mcp(_) => StatusCode::BAD_GATEWAY,
            Self::Upstream(_) => StatusCode::BAD_GATEWAY,
            Self::MissingEnv(_)
            | Self::Config { .. }
            | Self::InvalidProvider(_)
            | Self::InvalidInteger { .. }
            | Self::Startup(_)
            | Self::AgentLoopExceeded(_)
            | Self::Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, axum::Json(json!({ "error": self.to_string() }))).into_response()
    }
}
