use std::{collections::BTreeMap, process::Stdio, sync::Arc};

use serde::Deserialize;
use serde_json::{json, Value};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::Mutex,
};
use tracing::{info, warn};

use crate::{
    config::Config,
    error::AppError,
    models::{McpServerDescription, ToolDefinition},
};

pub async fn load_registry(config: &Config) -> Result<McpRegistry, AppError> {
    let Some(path) = &config.mcp_server_config else {
        return Ok(McpRegistry::default());
    };

    let raw = std::fs::read_to_string(path)
        .map_err(|err| AppError::Mcp(format!("failed to read MCP config `{path}`: {err}")))?;
    let configs: Vec<McpServerConfig> = serde_json::from_str(&raw)
        .map_err(|err| AppError::Mcp(format!("failed to parse MCP config `{path}`: {err}")))?;

    let mut servers = Vec::new();
    for server in configs {
        servers.push(McpServerConnection::connect(server).await?);
    }

    Ok(McpRegistry { servers })
}

#[derive(Clone, Default)]
pub struct McpRegistry {
    servers: Vec<McpServerConnection>,
}

impl McpRegistry {
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.servers
            .iter()
            .flat_map(|server| {
                server.tools.iter().cloned().map(|tool| ToolDefinition {
                    name: tool.exposed_name,
                    description: tool.description,
                    parameters: tool.parameters,
                })
            })
            .collect()
    }

    pub fn descriptions(&self) -> Vec<McpServerDescription> {
        self.servers
            .iter()
            .map(|server| McpServerDescription {
                name: server.name.clone(),
                tool_count: server.tools.len(),
            })
            .collect()
    }

    pub async fn call_tool(
        &self,
        exposed_name: &str,
        arguments: Value,
    ) -> Result<Option<String>, AppError> {
        for server in &self.servers {
            if let Some(tool) = server.lookup(exposed_name) {
                return Ok(Some(server.call_tool(tool, arguments).await?));
            }
        }

        Ok(None)
    }
}

#[derive(Clone)]
struct McpServerConnection {
    name: String,
    tools: Vec<McpTool>,
    process: Arc<Mutex<McpProcess>>,
}

impl McpServerConnection {
    async fn connect(config: McpServerConfig) -> Result<Self, AppError> {
        let mut command = Command::new(&config.command);
        command
            .args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(env) = &config.env {
            command.envs(env);
        }

        let mut child = command
            .spawn()
            .map_err(|err| AppError::Mcp(format!("failed to spawn MCP server `{}`: {err}", config.name)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::Mcp(format!("failed to capture stdin for `{}`", config.name)))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::Mcp(format!("failed to capture stdout for `{}`", config.name)))?;
        if let Some(stderr) = child.stderr.take() {
            spawn_stderr_logger(config.name.clone(), stderr);
        }

        let process = Arc::new(Mutex::new(McpProcess {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        }));

        {
            let mut process_guard = process.lock().await;
            process_guard
                .initialize()
                .await
                .map_err(|err| AppError::Mcp(format!("failed to initialize MCP server `{}`: {err}", config.name)))?;
        }

        let discovered = {
            let mut process_guard = process.lock().await;
            process_guard
                .list_tools()
                .await
                .map_err(|err| AppError::Mcp(format!("failed to list MCP tools for `{}`: {err}", config.name)))?
        };

        let tools = discovered
            .into_iter()
            .map(|tool| McpTool {
                exposed_name: format!("mcp__{}__{}", config.name, tool.name),
                remote_name: tool.name,
                description: tool.description.unwrap_or_else(|| "MCP tool".to_owned()),
                parameters: tool.input_schema,
            })
            .collect();

        Ok(Self {
            name: config.name,
            tools,
            process,
        })
    }

    fn lookup(&self, exposed_name: &str) -> Option<&McpTool> {
        self.tools.iter().find(|tool| tool.exposed_name == exposed_name)
    }

    async fn call_tool(&self, tool: &McpTool, arguments: Value) -> Result<String, AppError> {
        let arguments = match arguments {
            Value::Object(map) => Value::Object(map),
            _ => {
                return Err(AppError::Tool(
                    "MCP tool arguments must be a JSON object".to_owned(),
                ));
            }
        };

        let result = self
            .process
            .lock()
            .await
            .call_tool(&tool.remote_name, arguments)
            .await?;

        if let Some(structured) = result.structured_content {
            return Ok(serde_json::to_string_pretty(&structured)?);
        }

        let mut parts = Vec::new();
        for item in result.content {
            if item["type"].as_str() == Some("text") {
                if let Some(text) = item["text"].as_str() {
                    parts.push(text.to_owned());
                    continue;
                }
            }
            parts.push(serde_json::to_string_pretty(&item)?);
        }

        if parts.is_empty() {
            Ok("MCP tool returned no content.".to_owned())
        } else {
            Ok(parts.join("\n"))
        }
    }
}

struct McpProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpProcess {
    async fn initialize(&mut self) -> Result<(), AppError> {
        let result = self
            .request(
                "initialize",
                json!({
                    "protocolVersion": "2025-06-18",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "llm-api",
                        "version": "0.1.0"
                    }
                }),
            )
            .await?;

        let protocol_version = result["protocolVersion"].as_str().unwrap_or("unknown");
        info!(%protocol_version, "connected to MCP server");

        self.notification("notifications/initialized", None).await
    }

    async fn list_tools(&mut self) -> Result<Vec<RemoteTool>, AppError> {
        let mut all_tools = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let params = match &cursor {
                Some(cursor) => json!({ "cursor": cursor }),
                None => json!({}),
            };
            let result = self.request("tools/list", params).await?;
            let tools = result["tools"]
                .as_array()
                .ok_or_else(|| AppError::Mcp("MCP server returned invalid tools/list response".to_owned()))?;

            for tool in tools {
                all_tools.push(RemoteTool {
                    name: tool["name"].as_str().unwrap_or_default().to_owned(),
                    description: tool["description"].as_str().map(ToOwned::to_owned),
                    input_schema: tool["inputSchema"].clone(),
                });
            }

            cursor = result["nextCursor"].as_str().map(ToOwned::to_owned);
            if cursor.is_none() {
                break;
            }
        }

        Ok(all_tools)
    }

    async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<RemoteToolResult, AppError> {
        let result = self
            .request(
                "tools/call",
                json!({
                    "name": name,
                    "arguments": arguments,
                }),
            )
            .await?;

        Ok(RemoteToolResult {
            content: result["content"].as_array().cloned().unwrap_or_default(),
            structured_content: result.get("structuredContent").cloned().filter(|value| !value.is_null()),
        })
    }

    async fn request(&mut self, method: &str, params: Value) -> Result<Value, AppError> {
        let id = self.next_id;
        self.next_id += 1;
        let payload = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        self.send_line(&payload).await?;

        loop {
            let message = self.read_message().await?;
            if message.get("id").and_then(Value::as_u64) == Some(id) {
                if let Some(error) = message.get("error") {
                    return Err(AppError::Mcp(format!(
                        "MCP request `{method}` failed: {}",
                        error["message"].as_str().unwrap_or("unknown error")
                    )));
                }

                return Ok(message.get("result").cloned().unwrap_or(Value::Null));
            }

            handle_out_of_band_message(&message);
        }
    }

    async fn notification(&mut self, method: &str, params: Option<Value>) -> Result<(), AppError> {
        let payload = match params {
            Some(params) => json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
            }),
            None => json!({
                "jsonrpc": "2.0",
                "method": method,
            }),
        };

        self.send_line(&payload).await
    }

    async fn send_line(&mut self, value: &Value) -> Result<(), AppError> {
        let encoded = serde_json::to_string(value)?;
        self.stdin
            .write_all(encoded.as_bytes())
            .await
            .map_err(|err| AppError::Mcp(format!("failed to write to MCP server stdin: {err}")))?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|err| AppError::Mcp(format!("failed to write newline to MCP server stdin: {err}")))?;
        self.stdin
            .flush()
            .await
            .map_err(|err| AppError::Mcp(format!("failed to flush MCP server stdin: {err}")))?;
        Ok(())
    }

    async fn read_message(&mut self) -> Result<Value, AppError> {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = self
                .stdout
                .read_line(&mut line)
                .await
                .map_err(|err| AppError::Mcp(format!("failed reading MCP stdout: {err}")))?;

            if bytes == 0 {
                let status = self
                    .child
                    .try_wait()
                    .ok()
                    .flatten()
                    .map(|status| status.to_string())
                    .unwrap_or_else(|| "still running".to_owned());
                return Err(AppError::Mcp(format!(
                    "MCP server closed stdout unexpectedly (status: {status})"
                )));
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            return serde_json::from_str(trimmed).map_err(AppError::from);
        }
    }
}

fn handle_out_of_band_message(message: &Value) {
    if let Some(method) = message["method"].as_str() {
        match method {
            "notifications/message" => {
                warn!(payload = %message, "MCP server log message");
            }
            "notifications/tools/list_changed" => {
                info!("MCP server reported that its tool list changed");
            }
            _ => {
                info!(payload = %message, "ignoring MCP out-of-band message");
            }
        }
    }
}

fn spawn_stderr_logger(name: String, stderr: tokio::process::ChildStderr) {
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        loop {
            match reader.next_line().await {
                Ok(Some(line)) => info!(server = %name, stderr = %line, "MCP stderr"),
                Ok(None) => break,
                Err(err) => {
                    warn!(server = %name, error = %err, "failed reading MCP stderr");
                    break;
                }
            }
        }
    });
}

#[derive(Clone)]
struct McpTool {
    exposed_name: String,
    remote_name: String,
    description: String,
    parameters: Value,
}

#[derive(Debug, Deserialize)]
struct McpServerConfig {
    name: String,
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: Option<BTreeMap<String, String>>,
}

struct RemoteTool {
    name: String,
    description: Option<String>,
    input_schema: Value,
}

struct RemoteToolResult {
    content: Vec<Value>,
    structured_content: Option<Value>,
}
