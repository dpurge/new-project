use crate::{
    config::Config,
    error::AppError,
    memory::MemoryStore,
    mcp,
    providers::ProviderClient,
    tools::ToolRegistry,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub provider: ProviderClient,
    pub memory: MemoryStore,
    pub tools: ToolRegistry,
}

impl AppState {
    pub async fn new(config: Config, provider: ProviderClient) -> Result<Self, AppError> {
        let mcp_registry = mcp::load_registry(&config).await?;
        Ok(Self {
            memory: MemoryStore::new(config.memory_max_messages),
            tools: ToolRegistry::new(mcp_registry),
            provider,
            config,
        })
    }
}
