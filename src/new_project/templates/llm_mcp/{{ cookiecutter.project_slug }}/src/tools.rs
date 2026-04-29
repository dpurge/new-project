use chrono::Utc;
use serde_json::{json, Value};

use crate::error::AppError;

pub fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "echo",
            "description": "Echo text back verbatim.",
            "inputSchema": {
                "type": "object",
                "required": ["text"],
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to echo."
                    }
                }
            }
        }),
        json!({
            "name": "get_time",
            "description": "Return the current UTC time.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "format": {
                        "type": "string",
                        "enum": ["rfc3339", "date", "time"],
                        "description": "Output format."
                    }
                }
            }
        }),
    ]
}

pub fn call_tool(name: &str, arguments: Option<&Value>) -> Result<Value, AppError> {
    match name {
        "echo" => {
            let text = arguments
                .and_then(|value| value.get("text"))
                .and_then(Value::as_str)
                .ok_or_else(|| AppError::InvalidRequest("echo requires a `text` argument".to_owned()))?;

            Ok(tool_text_result(text.to_owned()))
        }
        "get_time" => {
            let format = arguments
                .and_then(|value| value.get("format"))
                .and_then(Value::as_str)
                .unwrap_or("rfc3339");

            let rendered = match format {
                "date" => Utc::now().format("%Y-%m-%d").to_string(),
                "time" => Utc::now().format("%H:%M:%SZ").to_string(),
                _ => Utc::now().to_rfc3339(),
            };

            Ok(tool_text_result(rendered))
        }
        other => Err(AppError::InvalidRequest(format!("unknown tool `{other}`"))),
    }
}

fn tool_text_result(text: String) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": text,
            }
        ]
    })
}
