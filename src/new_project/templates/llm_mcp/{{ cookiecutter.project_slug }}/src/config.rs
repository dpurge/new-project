use std::{env, net::SocketAddr};

use crate::error::AppError;

#[derive(Clone, Debug)]
pub struct Config {
    pub transport: TransportMode,
    pub bind_address: SocketAddr,
    pub allowed_origins: Vec<String>,
    pub auth_token: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportMode {
    Stdio,
    StreamableHttp,
    Http,
    All,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        let transport = match env::var("MCP_TRANSPORT")
            .unwrap_or_else(|_| "streamable-http".to_owned())
            .to_lowercase()
            .as_str()
        {
            "stdio" => TransportMode::Stdio,
            "streamable-http" => TransportMode::StreamableHttp,
            "http" => TransportMode::Http,
            "all" => TransportMode::All,
            value => return Err(AppError::InvalidConfig(format!("unsupported MCP_TRANSPORT `{value}`"))),
        };

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
        let port = env::var("PORT").unwrap_or_else(|_| "{{ cookiecutter.http_port }}".to_owned());
        let bind_address = format!("{host}:{port}")
            .parse()
            .map_err(|source| AppError::BindAddress {
                message: format!("invalid HOST/PORT combination: {host}:{port}"),
                source,
            })?;

        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost,http://127.0.0.1".to_owned())
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        Ok(Self {
            transport,
            bind_address,
            allowed_origins,
            auth_token: env::var("MCP_AUTH_TOKEN").ok(),
        })
    }
}
