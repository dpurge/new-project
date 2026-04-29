use reqwest::Client;
use serde_json::{json, Value};

use crate::{
    config::{Config, ProviderKind},
    error::AppError,
    models::{AgentMessage, InternalToolCall, ProviderRequest, ProviderResponse, UsageStats},
};

#[derive(Clone)]
pub struct ProviderClient {
    client: Client,
    config: Config,
}

impl ProviderClient {
    pub fn new(config: Config) -> Result<Self, AppError> {
        Ok(Self {
            client: Client::builder().build()?,
            config,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn generate(&self, request: ProviderRequest) -> Result<ProviderResponse, AppError> {
        match self.config.provider {
            ProviderKind::OpenAi => self.generate_openai(request).await,
            ProviderKind::Anthropic => self.generate_anthropic(request).await,
            ProviderKind::Ollama => self.generate_ollama(request).await,
        }
    }

    async fn generate_openai(&self, request: ProviderRequest) -> Result<ProviderResponse, AppError> {
        let model = request
            .model_override
            .unwrap_or_else(|| self.config.model.clone());
        let body = json!({
            "model": model,
            "messages": request.messages.iter().map(openai_message).collect::<Vec<_>>(),
            "tools": openai_tools(&request.tools),
            "tool_choice": "auto",
            "stream": false,
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
        });

        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.base_url))
            .bearer_auth(self.required_api_key()?)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let payload: Value = response.json().await?;
        let choice = payload["choices"]
            .get(0)
            .ok_or_else(|| AppError::InvalidRequest("provider returned no choices".to_owned()))?;
        let message = &choice["message"];

        Ok(ProviderResponse {
            model: payload["model"]
                .as_str()
                .unwrap_or(self.config.model.as_str())
                .to_owned(),
            content: message["content"].as_str().unwrap_or_default().to_owned(),
            tool_calls: parse_openai_tool_calls(message["tool_calls"].as_array()),
            usage: UsageStats {
                prompt_tokens: read_u32(&payload["usage"]["prompt_tokens"]),
                completion_tokens: read_u32(&payload["usage"]["completion_tokens"]),
                total_tokens: read_u32(&payload["usage"]["total_tokens"]),
            },
        })
    }

    async fn generate_anthropic(
        &self,
        request: ProviderRequest,
    ) -> Result<ProviderResponse, AppError> {
        let model = request
            .model_override
            .unwrap_or_else(|| self.config.model.clone());
        let system = request
            .messages
            .iter()
            .filter(|message| message.role == "system")
            .map(|message| message.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        let body = json!({
            "model": model,
            "max_tokens": request.max_tokens.unwrap_or(1024),
            "temperature": request.temperature,
            "system": if system.is_empty() { Value::Null } else { Value::String(system) },
            "messages": anthropic_messages(&request.messages)?,
            "tools": anthropic_tools(&request.tools),
        });

        let response = self
            .client
            .post(format!("{}/v1/messages", self.config.base_url))
            .header("x-api-key", self.required_api_key()?)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let payload: Value = response.json().await?;
        let blocks = payload["content"]
            .as_array()
            .ok_or_else(|| AppError::InvalidRequest("anthropic response had no content blocks".to_owned()))?;

        let mut text = String::new();
        let mut tool_calls = Vec::new();
        for block in blocks {
            match block["type"].as_str().unwrap_or_default() {
                "text" => {
                    if let Some(value) = block["text"].as_str() {
                        text.push_str(value);
                    }
                }
                "tool_use" => {
                    tool_calls.push(InternalToolCall {
                        id: block["id"].as_str().unwrap_or_default().to_owned(),
                        name: block["name"].as_str().unwrap_or_default().to_owned(),
                        arguments: block["input"].clone(),
                    });
                }
                _ => {}
            }
        }

        Ok(ProviderResponse {
            model: payload["model"]
                .as_str()
                .unwrap_or(self.config.model.as_str())
                .to_owned(),
            content: text,
            tool_calls,
            usage: UsageStats {
                prompt_tokens: read_u32(&payload["usage"]["input_tokens"]),
                completion_tokens: read_u32(&payload["usage"]["output_tokens"]),
                total_tokens: read_u32(&payload["usage"]["input_tokens"])
                    + read_u32(&payload["usage"]["output_tokens"]),
            },
        })
    }

    async fn generate_ollama(&self, request: ProviderRequest) -> Result<ProviderResponse, AppError> {
        let model = request
            .model_override
            .unwrap_or_else(|| self.config.model.clone());
        let body = json!({
            "model": model,
            "messages": request.messages.iter().map(ollama_message).collect::<Vec<_>>(),
            "tools": openai_tools(&request.tools),
            "stream": false,
            "options": {
                "temperature": request.temperature,
            }
        });

        let response = self
            .client
            .post(format!("{}/api/chat", self.config.base_url))
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let payload: Value = response.json().await?;
        let message = &payload["message"];

        Ok(ProviderResponse {
            model: payload["model"]
                .as_str()
                .unwrap_or(self.config.model.as_str())
                .to_owned(),
            content: message["content"].as_str().unwrap_or_default().to_owned(),
            tool_calls: parse_ollama_tool_calls(message["tool_calls"].as_array()),
            usage: UsageStats {
                prompt_tokens: read_u32(&payload["prompt_eval_count"]),
                completion_tokens: read_u32(&payload["eval_count"]),
                total_tokens: read_u32(&payload["prompt_eval_count"])
                    + read_u32(&payload["eval_count"]),
            },
        })
    }

    fn required_api_key(&self) -> Result<&str, AppError> {
        self.config
            .api_key
            .as_deref()
            .ok_or_else(|| AppError::MissingEnv("LLM_API_KEY".to_owned()))
    }
}

fn openai_message(message: &AgentMessage) -> Value {
    match message.role.as_str() {
        "assistant" => {
            let tool_calls = if message.tool_calls.is_empty() {
                Value::Null
            } else {
                Value::Array(
                    message
                        .tool_calls
                        .iter()
                        .map(|call| {
                            json!({
                                "id": call.id,
                                "type": "function",
                                "function": {
                                    "name": call.name,
                                    "arguments": call.arguments.to_string(),
                                }
                            })
                        })
                        .collect(),
                )
            };

            json!({
                "role": "assistant",
                "content": if message.content.is_empty() { Value::Null } else { Value::String(message.content.clone()) },
                "tool_calls": tool_calls,
            })
        }
        "tool" => json!({
            "role": "tool",
            "tool_call_id": message.tool_call_id,
            "content": message.content,
        }),
        _ => json!({
            "role": message.role,
            "content": message.content,
        }),
    }
}

fn anthropic_messages(messages: &[AgentMessage]) -> Result<Vec<Value>, AppError> {
    let mut result = Vec::new();

    for message in messages.iter().filter(|message| message.role != "system") {
        match message.role.as_str() {
            "user" => {
                result.push(json!({
                    "role": "user",
                    "content": [{ "type": "text", "text": message.content }],
                }));
            }
            "assistant" => {
                let mut content = Vec::new();
                if !message.content.is_empty() {
                    content.push(json!({ "type": "text", "text": message.content }));
                }
                for call in &message.tool_calls {
                    content.push(json!({
                        "type": "tool_use",
                        "id": call.id,
                        "name": call.name,
                        "input": call.arguments,
                    }));
                }
                result.push(json!({
                    "role": "assistant",
                    "content": content,
                }));
            }
            "tool" => {
                let tool_call_id = message
                    .tool_call_id
                    .clone()
                    .ok_or_else(|| AppError::InvalidRequest("anthropic tool messages require tool_call_id".to_owned()))?;
                result.push(json!({
                    "role": "user",
                    "content": [
                        {
                            "type": "tool_result",
                            "tool_use_id": tool_call_id,
                            "content": message.content,
                        }
                    ],
                }));
            }
            _ => {}
        }
    }

    Ok(result)
}

fn ollama_message(message: &AgentMessage) -> Value {
    match message.role.as_str() {
        "assistant" => {
            let tool_calls = if message.tool_calls.is_empty() {
                Value::Null
            } else {
                Value::Array(
                    message
                        .tool_calls
                        .iter()
                        .enumerate()
                        .map(|(index, call)| {
                            json!({
                                "type": "function",
                                "function": {
                                    "index": index,
                                    "name": call.name,
                                    "arguments": call.arguments,
                                }
                            })
                        })
                        .collect(),
                )
            };

            json!({
                "role": "assistant",
                "content": message.content,
                "tool_calls": tool_calls,
            })
        }
        "tool" => json!({
            "role": "tool",
            "tool_name": message.tool_name,
            "content": message.content,
        }),
        _ => json!({
            "role": message.role,
            "content": message.content,
        }),
    }
}

fn openai_tools(tools: &[crate::models::ToolDefinition]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters,
                }
            })
        })
        .collect()
}

fn anthropic_tools(tools: &[crate::models::ToolDefinition]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "input_schema": tool.parameters,
            })
        })
        .collect()
}

fn parse_openai_tool_calls(tool_calls: Option<&Vec<Value>>) -> Vec<InternalToolCall> {
    tool_calls
        .into_iter()
        .flatten()
        .map(|call| {
            let raw = call["function"]["arguments"].as_str().unwrap_or("{}");
            let arguments = serde_json::from_str(raw).unwrap_or(Value::String(raw.to_owned()));
            InternalToolCall {
                id: call["id"].as_str().unwrap_or_default().to_owned(),
                name: call["function"]["name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                arguments,
            }
        })
        .collect()
}

fn parse_ollama_tool_calls(tool_calls: Option<&Vec<Value>>) -> Vec<InternalToolCall> {
    tool_calls
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, call)| InternalToolCall {
            id: format!("ollama-tool-{index}"),
            name: call["function"]["name"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            arguments: call["function"]["arguments"].clone(),
        })
        .collect()
}

fn read_u32(value: &Value) -> u32 {
    value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(0)
}
