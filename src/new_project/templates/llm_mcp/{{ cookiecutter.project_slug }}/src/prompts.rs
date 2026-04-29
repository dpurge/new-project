use serde_json::{json, Value};

use crate::error::AppError;

pub fn prompt_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "summarize-notes",
            "description": "Summarize notes into a concise action-oriented update.",
            "arguments": [
                {
                    "name": "notes",
                    "description": "The notes to summarize.",
                    "required": true
                }
            ]
        }),
        json!({
            "name": "triage-issue",
            "description": "Produce an issue triage summary with severity and next step.",
            "arguments": [
                {
                    "name": "title",
                    "description": "Issue title.",
                    "required": true
                },
                {
                    "name": "details",
                    "description": "Issue details.",
                    "required": true
                }
            ]
        }),
    ]
}

pub fn get_prompt(name: &str, arguments: Option<&Value>) -> Result<Value, AppError> {
    match name {
        "summarize-notes" => {
            let notes = required_arg(arguments, "notes")?;
            Ok(json!({
                "description": "Summarize notes into a concise status update.",
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!("Summarize these notes into a short update with next actions:\n\n{notes}")
                        }
                    }
                ]
            }))
        }
        "triage-issue" => {
            let title = required_arg(arguments, "title")?;
            let details = required_arg(arguments, "details")?;
            Ok(json!({
                "description": "Triage an issue and recommend a next step.",
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": format!(
                                "Triage this issue. Return severity, likely owner, and the next action.\n\nTitle: {title}\n\nDetails:\n{details}"
                            )
                        }
                    }
                ]
            }))
        }
        other => Err(AppError::InvalidRequest(format!("unknown prompt `{other}`"))),
    }
}

fn required_arg(arguments: Option<&Value>, name: &str) -> Result<String, AppError> {
    arguments
        .and_then(|value| value.get(name))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| AppError::InvalidRequest(format!("prompt requires `{name}` argument")))
}
