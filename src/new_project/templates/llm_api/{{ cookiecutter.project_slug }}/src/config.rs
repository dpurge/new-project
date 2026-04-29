use std::{env, net::SocketAddr};

use crate::error::AppError;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_address: SocketAddr,
    pub provider: ProviderKind,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: String,
    pub memory_max_messages: usize,
    pub agent_max_steps: usize,
    pub mcp_server_config: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderKind {
    OpenAi,
    Anthropic,
    Ollama,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_owned());
        let bind_address = format!("{host}:{port}")
            .parse()
            .map_err(|source| AppError::Config {
                message: format!("invalid HOST/PORT combination: {host}:{port}"),
                source,
            })?;

        let provider = match env::var("LLM_PROVIDER")
            .unwrap_or_else(|_| "openai".to_owned())
            .to_lowercase()
            .as_str()
        {
            "openai" => ProviderKind::OpenAi,
            "anthropic" => ProviderKind::Anthropic,
            "ollama" => ProviderKind::Ollama,
            value => {
                return Err(AppError::InvalidProvider(value.to_owned()));
            }
        };

        let model = required_env("LLM_MODEL")?;
        let api_key = env::var("LLM_API_KEY").ok();

        if provider != ProviderKind::Ollama && api_key.is_none() {
            return Err(AppError::MissingEnv("LLM_API_KEY".to_owned()));
        }

        let base_url = env::var("LLM_BASE_URL").unwrap_or_else(|_| match provider {
            ProviderKind::OpenAi => "https://api.openai.com/v1".to_owned(),
            ProviderKind::Anthropic => "https://api.anthropic.com".to_owned(),
            ProviderKind::Ollama => "http://localhost:11434".to_owned(),
        });

        let memory_max_messages = parse_usize("MEMORY_MAX_MESSAGES", 12)?;
        let agent_max_steps = parse_usize("AGENT_MAX_STEPS", 6)?;

        Ok(Self {
            bind_address,
            provider,
            model,
            api_key,
            base_url: base_url.trim_end_matches('/').to_owned(),
            memory_max_messages,
            agent_max_steps,
            mcp_server_config: env::var("MCP_SERVER_CONFIG").ok(),
        })
    }

    pub fn provider_name(&self) -> &'static str {
        match self.provider {
            ProviderKind::OpenAi => "openai",
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::Ollama => "ollama",
        }
    }
}

fn required_env(name: &str) -> Result<String, AppError> {
    env::var(name).map_err(|_| AppError::MissingEnv(name.to_owned()))
}

fn parse_usize(name: &str, default: usize) -> Result<usize, AppError> {
    match env::var(name) {
        Ok(value) => value.parse::<usize>().map_err(|_| AppError::InvalidInteger {
            name: name.to_owned(),
            value,
        }),
        Err(_) => Ok(default),
    }
}
