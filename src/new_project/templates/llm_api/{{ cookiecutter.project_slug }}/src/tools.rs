use crate::{
    error::AppError,
    memory::MemoryStore,
    mcp::McpRegistry,
    models::{InternalToolCall, McpServerDescription, ToolDefinition, ToolDescription},
};
use chrono::Utc;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct ToolRegistry {
    mcp: McpRegistry,
}

impl ToolRegistry {
    pub fn new(mcp: McpRegistry) -> Self {
        Self { mcp }
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        let mut tools = vec![
            ToolDefinition {
                name: "get_time".to_owned(),
                description: "Get the current UTC date or time.".to_owned(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "format": {
                            "type": "string",
                            "enum": ["rfc3339", "date", "time"],
                            "description": "Output format. Defaults to rfc3339."
                        }
                    }
                }),
            },
            ToolDefinition {
                name: "echo".to_owned(),
                description: "Echo text back verbatim. Useful as a simple example tool.".to_owned(),
                parameters: json!({
                    "type": "object",
                    "required": ["text"],
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "The text to echo."
                        }
                    }
                }),
            },
            ToolDefinition {
                name: "remember_fact".to_owned(),
                description: "Store a fact in the current session memory.".to_owned(),
                parameters: json!({
                    "type": "object",
                    "required": ["key", "value"],
                    "properties": {
                        "key": {
                            "type": "string",
                            "description": "Memory key."
                        },
                        "value": {
                            "type": "string",
                            "description": "Memory value."
                        }
                    }
                }),
            },
            ToolDefinition {
                name: "recall_fact".to_owned(),
                description: "Read a fact from the current session memory.".to_owned(),
                parameters: json!({
                    "type": "object",
                    "required": ["key"],
                    "properties": {
                        "key": {
                            "type": "string",
                            "description": "Memory key to look up."
                        }
                    }
                }),
            },
            ToolDefinition {
                name: "list_facts".to_owned(),
                description: "List all remembered facts for the current session.".to_owned(),
                parameters: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ];
        tools.extend(self.mcp.definitions());
        tools
    }

    pub fn descriptions(&self) -> Vec<ToolDescription> {
        self.definitions()
            .into_iter()
            .map(|tool| {
                let source = if tool.name.starts_with("mcp__") {
                    "mcp".to_owned()
                } else {
                    "local".to_owned()
                };

                ToolDescription {
                    name: tool.name,
                    description: tool.description,
                    parameters: tool.parameters,
                    source,
                }
            })
            .collect()
    }

    pub fn mcp_servers(&self) -> Vec<McpServerDescription> {
        self.mcp.descriptions()
    }

    pub async fn execute(
        &self,
        call: &InternalToolCall,
        context: ToolContext<'_>,
    ) -> Result<String, AppError> {
        match call.name.as_str() {
            "get_time" => Ok(run_get_time(&call.arguments)),
            "echo" => Ok(read_string_arg(&call.arguments, "text")?),
            "remember_fact" => run_remember_fact(&call.arguments, context).await,
            "recall_fact" => run_recall_fact(&call.arguments, context).await,
            "list_facts" => run_list_facts(context).await,
            _ => match self.mcp.call_tool(&call.name, call.arguments.clone()).await? {
                Some(value) => Ok(value),
                None => Err(AppError::Tool(format!("unknown tool: {}", call.name))),
            },
        }
    }
}

#[derive(Clone)]
pub struct ToolContext<'a> {
    pub session_id: Option<String>,
    pub memory: &'a MemoryStore,
}

fn run_get_time(arguments: &Value) -> String {
    let format = arguments["format"].as_str().unwrap_or("rfc3339");
    match format {
        "date" => Utc::now().format("%Y-%m-%d").to_string(),
        "time" => Utc::now().format("%H:%M:%SZ").to_string(),
        _ => Utc::now().to_rfc3339(),
    }
}

async fn run_remember_fact(arguments: &Value, context: ToolContext<'_>) -> Result<String, AppError> {
    let session_id = require_session_id(context.session_id)?;
    let key = read_string_arg(arguments, "key")?;
    let value = read_string_arg(arguments, "value")?;
    context
        .memory
        .remember_fact(&session_id, key.clone(), value.clone())
        .await;
    Ok(format!("Stored fact `{key}` with value `{value}`."))
}

async fn run_recall_fact(arguments: &Value, context: ToolContext<'_>) -> Result<String, AppError> {
    let session_id = require_session_id(context.session_id)?;
    let key = read_string_arg(arguments, "key")?;
    Ok(match context.memory.recall_fact(&session_id, &key).await {
        Some(value) => format!("{key} = {value}"),
        None => format!("No fact stored for key `{key}`."),
    })
}

async fn run_list_facts(context: ToolContext<'_>) -> Result<String, AppError> {
    let session_id = require_session_id(context.session_id)?;
    let facts = context.memory.list_facts(&session_id).await;
    if facts.is_empty() {
        return Ok("No facts stored for this session.".to_owned());
    }

    let body = facts
        .into_iter()
        .map(|(key, value)| format!("- {key}: {value}"))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(body)
}

fn require_session_id(session_id: Option<String>) -> Result<String, AppError> {
    session_id.ok_or_else(|| {
        AppError::Tool("this tool requires a `session_id` in the chat request".to_owned())
    })
}

fn read_string_arg(arguments: &Value, name: &str) -> Result<String, AppError> {
    arguments[name]
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| AppError::Tool(format!("missing string argument: {name}")))
}
