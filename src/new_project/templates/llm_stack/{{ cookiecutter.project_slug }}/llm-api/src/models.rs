use axum::{
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::Response,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<ApiMessage>,
    pub stream: Option<bool>,
    pub session_id: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

impl ChatCompletionRequest {
    pub fn agent_messages(&self) -> Vec<AgentMessage> {
        self.messages
            .iter()
            .map(|message| AgentMessage {
                role: message.role.clone(),
                content: message.content.clone().unwrap_or_default(),
                tool_calls: message
                    .tool_calls
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(InternalToolCall::from_api_call)
                    .collect(),
                tool_call_id: message.tool_call_id.clone(),
                tool_name: message.name.clone(),
            })
            .collect()
    }

    pub fn user_visible_messages(&self) -> Vec<AgentMessage> {
        self.agent_messages()
            .into_iter()
            .filter(|message| matches!(message.role.as_str(), "system" | "user" | "assistant"))
            .collect()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApiMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_calls: Option<Vec<ApiToolCall>>,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApiToolCall {
    pub id: String,
    #[serde(default = "default_tool_type")]
    pub r#type: String,
    pub function: ApiToolFunction,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApiToolFunction {
    pub name: String,
    pub arguments: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentMessage {
    pub role: String,
    pub content: String,
    pub tool_calls: Vec<InternalToolCall>,
    pub tool_call_id: Option<String>,
    pub tool_name: Option<String>,
}

impl AgentMessage {
    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_owned(),
            content,
            tool_calls: Vec::new(),
            tool_call_id: None,
            tool_name: None,
        }
    }

    pub fn assistant_with_tools(content: String, tool_calls: Vec<InternalToolCall>) -> Self {
        Self {
            role: "assistant".to_owned(),
            content,
            tool_calls,
            tool_call_id: None,
            tool_name: None,
        }
    }

    pub fn tool(result: ToolResultMessage) -> Self {
        Self {
            role: "tool".to_owned(),
            content: result.content,
            tool_calls: Vec::new(),
            tool_call_id: Some(result.tool_call_id),
            tool_name: Some(result.tool_name),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ToolResultMessage {
    pub tool_call_id: String,
    pub tool_name: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InternalToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

impl InternalToolCall {
    pub fn from_api_call(call: ApiToolCall) -> Self {
        let arguments = match call.function.arguments {
            Value::String(raw) => serde_json::from_str(&raw).unwrap_or(Value::String(raw)),
            value => value,
        };

        Self {
            id: call.id,
            name: call.function.name,
            arguments,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProviderRequest {
    pub model_override: Option<String>,
    pub messages: Vec<AgentMessage>,
    pub tools: Vec<ToolDefinition>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct ProviderResponse {
    pub model: String,
    pub content: String,
    pub tool_calls: Vec<InternalToolCall>,
    pub usage: UsageStats,
}

#[derive(Clone, Debug, Default)]
pub struct UsageStats {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl UsageStats {
    pub fn accumulate(&mut self, other: &UsageStats) {
        self.prompt_tokens += other.prompt_tokens;
        self.completion_tokens += other.completion_tokens;
        self.total_tokens += other.total_tokens;
    }
}

#[derive(Clone, Debug)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub source: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct McpServerDescription {
    pub name: String,
    pub tool_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct MemorySnapshot {
    pub session_id: String,
    pub messages: Vec<AgentMessage>,
    pub facts: BTreeMap<String, String>,
}

pub fn json_response(model: String, content: String, usage: UsageStats) -> Response {
    let body = json!({
        "id": format!("chatcmpl-{}", Uuid::new_v4().simple()),
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": model,
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": content,
                    "tool_calls": Value::Null,
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": usage.prompt_tokens,
            "completion_tokens": usage.completion_tokens,
            "total_tokens": usage.total_tokens,
        }
    });

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(axum::body::Body::from(body.to_string()))
        .expect("response builder should be valid")
}

pub fn streaming_response(model: String, content: String) -> Response {
    let completion_id = format!("chatcmpl-{}", Uuid::new_v4().simple());
    let created = chrono::Utc::now().timestamp();

    let mut events = Vec::new();
    for chunk in chunk_text(&content, 48) {
        let payload = json!({
            "id": completion_id,
            "object": "chat.completion.chunk",
            "created": created,
            "model": model,
            "choices": [
                {
                    "index": 0,
                    "delta": {
                        "content": chunk
                    },
                    "finish_reason": Value::Null
                }
            ]
        });
        events.push(format!("data: {payload}\n\n"));
    }

    let final_payload = json!({
        "id": completion_id,
        "object": "chat.completion.chunk",
        "created": created,
        "model": model,
        "choices": [
            {
                "index": 0,
                "delta": {},
                "finish_reason": "stop"
            }
        ]
    });
    events.push(format!("data: {final_payload}\n\n"));
    events.push("data: [DONE]\n\n".to_owned());

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static("text/event-stream"))
        .body(axum::body::Body::from(events.concat()))
        .expect("response builder should be valid")
}

fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if current.len() >= chunk_size {
            chunks.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

fn default_tool_type() -> String {
    "function".to_owned()
}
