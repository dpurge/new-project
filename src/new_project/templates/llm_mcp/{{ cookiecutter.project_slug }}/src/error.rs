use std::net::AddrParseError;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{message}")]
    BindAddress {
        message: String,
        source: AddrParseError,
    },
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("server startup failed: {0}")]
    Startup(String),
    #[error("transport failure: {0}")]
    Transport(String),
    #[error("authentication failed: {0}")]
    Auth(String),
    #[error("invalid JSON-RPC request: {0}")]
    InvalidRequest(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            Self::Auth(_) => StatusCode::UNAUTHORIZED,
            Self::BindAddress { .. }
            | Self::InvalidConfig(_)
            | Self::Startup(_)
            | Self::Transport(_)
            | Self::Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, axum::Json(json!({ "error": self.to_string() }))).into_response()
    }
}
