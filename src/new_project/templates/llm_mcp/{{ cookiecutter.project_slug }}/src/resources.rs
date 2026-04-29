use serde_json::{json, Value};

use crate::error::AppError;

pub fn resource_definitions() -> Vec<Value> {
    vec![
        json!({
            "uri": "memo://welcome",
            "name": "Welcome Memo",
            "description": "Introductory memo for clients connecting to this MCP server.",
            "mimeType": "text/markdown"
        }),
        json!({
            "uri": "memo://architecture",
            "name": "Architecture Notes",
            "description": "Short architecture notes for this sample server.",
            "mimeType": "text/markdown"
        }),
    ]
}

pub fn read_resource(uri: &str) -> Result<Value, AppError> {
    let text = match uri {
        "memo://welcome" => "# Welcome\n\nThis MCP server demonstrates tools, resources, prompts, and multiple transports.",
        "memo://architecture" => "# Architecture\n\n- Shared JSON-RPC router\n- Stdio transport\n- Streamable HTTP transport\n- Legacy HTTP+SSE compatibility",
        other => {
            return Err(AppError::InvalidRequest(format!("unknown resource `{other}`")));
        }
    };

    Ok(json!({
        "contents": [
            {
                "uri": uri,
                "mimeType": "text/markdown",
                "text": text
            }
        ]
    }))
}
