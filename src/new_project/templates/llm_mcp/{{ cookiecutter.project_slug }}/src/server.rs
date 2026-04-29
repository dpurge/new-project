use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query, State},
    http::{
        header::{HeaderMap, CONTENT_TYPE},
        HeaderValue, StatusCode,
    },
    response::{
        sse::{Event, KeepAlive, Sse},
        Response,
    },
};
use futures_util::{stream, Stream};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::{broadcast, RwLock};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::info;
use uuid::Uuid;

use crate::{
    error::AppError,
    jsonrpc::{IncomingMessage, JsonRpcRequest, JsonRpcResponse, MCP_PROTOCOL_VERSION, JSONRPC_VERSION},
    prompts, resources,
    tools,
};

#[derive(Clone)]
pub struct AppState {
    allowed_origins: Arc<Vec<String>>,
    auth_token: Arc<Option<String>>,
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
}

impl AppState {
    pub fn new(allowed_origins: Vec<String>, auth_token: Option<String>) -> Self {
        Self {
            allowed_origins: Arc::new(allowed_origins),
            auth_token: Arc::new(auth_token),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn validate_origin(&self, headers: &HeaderMap) -> Result<(), AppError> {
        let Some(origin) = headers.get("origin") else {
            return Ok(());
        };

        let origin = origin
            .to_str()
            .map_err(|_| AppError::InvalidRequest("invalid Origin header".to_owned()))?;

        if self.allowed_origins.iter().any(|value| value == origin) {
            return Ok(());
        }

        Err(AppError::InvalidRequest(format!(
            "Origin `{origin}` is not allowed"
        )))
    }

    pub fn validate_auth(&self, headers: &HeaderMap) -> Result<(), AppError> {
        let Some(expected) = self.auth_token.as_ref().as_ref() else {
            return Ok(());
        };

        let provided = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Auth("missing Authorization header".to_owned()))?;

        let expected_header = format!("Bearer {expected}");
        if provided == expected_header {
            Ok(())
        } else {
            Err(AppError::Auth("invalid bearer token".to_owned()))
        }
    }

    async fn ensure_session(&self, session_id: Option<String>) -> String {
        match session_id {
            Some(session_id) => {
                let mut sessions = self.sessions.write().await;
                sessions.entry(session_id.clone()).or_insert_with(new_session_state);
                session_id
            }
            None => {
                let session_id = Uuid::new_v4().to_string();
                let mut sessions = self.sessions.write().await;
                sessions.insert(session_id.clone(), new_session_state());
                session_id
            }
        }
    }

    async fn session_sender(&self, session_id: &str) -> broadcast::Sender<Value> {
        let mut sessions = self.sessions.write().await;
        sessions
            .entry(session_id.to_owned())
            .or_insert_with(new_session_state)
            .sender
            .clone()
    }
}

#[derive(Clone)]
struct SessionState {
    sender: broadcast::Sender<Value>,
}

fn new_session_state() -> SessionState {
    let (sender, _) = broadcast::channel(64);
    SessionState { sender }
}

pub async fn handle_http_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    maybe_session: Option<String>,
    body: String,
) -> Result<Response, AppError> {
    state.validate_origin(&headers)?;
    state.validate_auth(&headers)?;
    let incoming: IncomingMessage = serde_json::from_str(&body)?;

    let session_id = match &incoming {
        IncomingMessage::Request(request) if request.method == "initialize" => {
            Some(state.ensure_session(maybe_session).await)
        }
        _ => maybe_session,
    };

    let response = dispatch_incoming(&state, incoming, session_id.clone()).await?;
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    if let Some(session_id) = session_id {
        builder = builder.header("Mcp-Session-Id", session_id);
    }

    Ok(builder
        .body(axum::body::Body::from(serde_json::to_vec(&response)?))
        .expect("response builder should be valid"))
}

pub async fn streamable_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, AppError> {
    state.validate_origin(&headers)?;
    state.validate_auth(&headers)?;

    let session_id = headers
        .get("mcp-session-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let sender = state.session_sender(&session_id).await;
    let rx = sender.subscribe();
    let initial = Event::default()
        .event("session")
        .data(json!({ "sessionId": session_id }).to_string());

    let stream = stream::once(async move { Ok(initial) }).chain(
        BroadcastStream::new(rx).filter_map(|message| {
            match message {
                Ok(payload) => Some(Ok(Event::default().event("message").data(payload.to_string()))),
                Err(_) => None,
            }
        }),
    );

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

pub async fn legacy_sse(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, AppError> {
    state.validate_origin(&headers)?;
    state.validate_auth(&headers)?;
    let session_id = state.ensure_session(None).await;
    let sender = state.session_sender(&session_id).await;
    let rx = sender.subscribe();

    let endpoint = Event::default().event("endpoint").data(
        json!({
            "uri": format!("/messages?session_id={session_id}")
        })
        .to_string(),
    );

    let stream = stream::once(async move { Ok(endpoint) }).chain(
        BroadcastStream::new(rx).filter_map(|message| {
            match message {
                Ok(payload) => Some(Ok(Event::default().event("message").data(payload.to_string()))),
                Err(_) => None,
            }
        }),
    );

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[derive(Debug, Deserialize)]
pub struct LegacyQuery {
    pub session_id: Option<String>,
}

pub async fn legacy_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<LegacyQuery>,
    body: String,
) -> Result<Response, AppError> {
    handle_http_message(State(state), headers, query.session_id, body).await
}

pub async fn dispatch_incoming(
    state: &AppState,
    incoming: IncomingMessage,
    session_id: Option<String>,
) -> Result<Value, AppError> {
    match incoming {
        IncomingMessage::Request(request) => dispatch_request(state, request, session_id).await,
        IncomingMessage::Batch(requests) => {
            let mut responses = Vec::new();
            for request in requests {
                if let Some(response) = dispatch_request_value(state, request, session_id.clone()).await? {
                    responses.push(response);
                }
            }
            Ok(Value::Array(responses))
        }
    }
}

pub async fn dispatch_request_value(
    state: &AppState,
    request: JsonRpcRequest,
    session_id: Option<String>,
) -> Result<Option<Value>, AppError> {
    if request.id.is_none() {
        handle_notification(state, &request, session_id).await?;
        return Ok(None);
    }

    let response = handle_request(state, request, session_id).await;
    Ok(Some(serde_json::to_value(response)?))
}

async fn dispatch_request(
    state: &AppState,
    request: JsonRpcRequest,
    session_id: Option<String>,
) -> Result<Value, AppError> {
    match dispatch_request_value(state, request, session_id).await? {
        Some(value) => Ok(value),
        None => Ok(Value::Null),
    }
}

async fn handle_notification(
    state: &AppState,
    request: &JsonRpcRequest,
    session_id: Option<String>,
) -> Result<(), AppError> {
    if request.jsonrpc != JSONRPC_VERSION {
        return Err(AppError::InvalidRequest("jsonrpc must be `2.0`".to_owned()));
    }

    if request.method == "notifications/initialized" {
        if let Some(session_id) = session_id {
            let sender = state.session_sender(&session_id).await;
            let _ = sender.send(json!({
                "jsonrpc": JSONRPC_VERSION,
                "method": "notifications/message",
                "params": {
                    "level": "info",
                    "data": "session initialized"
                }
            }));
        }
        return Ok(());
    }

    info!(method = %request.method, "ignoring notification");
    Ok(())
}

async fn handle_request(
    _state: &AppState,
    request: JsonRpcRequest,
    session_id: Option<String>,
) -> JsonRpcResponse {
    if request.jsonrpc != JSONRPC_VERSION {
        return JsonRpcResponse::error(request.id, -32600, "jsonrpc must be `2.0`");
    }

    let id = request.id.clone();

    match route_method(request, session_id).await {
        Ok(result) => JsonRpcResponse::success(id, result),
        Err(err) => JsonRpcResponse::error(id, map_error_code(&err), err.to_string()),
    }
}

async fn route_method(
    request: JsonRpcRequest,
    _session_id: Option<String>,
) -> Result<Value, AppError> {
    match request.method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {
                "tools": {
                    "listChanged": false
                },
                "resources": {
                    "listChanged": false,
                    "subscribe": false
                },
                "prompts": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "{{ cookiecutter.project_slug }}",
                "version": "0.1.0"
            }
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({
            "tools": tools::tool_definitions()
        })),
        "tools/call" => {
            let params = request
                .params
                .ok_or_else(|| AppError::InvalidRequest("tools/call requires params".to_owned()))?;
            let name = params
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| AppError::InvalidRequest("tools/call requires `name`".to_owned()))?;
            let arguments = params.get("arguments");
            tools::call_tool(name, arguments)
        }
        "resources/list" => Ok(json!({
            "resources": resources::resource_definitions()
        })),
        "resources/read" => {
            let params = request
                .params
                .ok_or_else(|| AppError::InvalidRequest("resources/read requires params".to_owned()))?;
            let uri = params
                .get("uri")
                .and_then(Value::as_str)
                .ok_or_else(|| AppError::InvalidRequest("resources/read requires `uri`".to_owned()))?;
            resources::read_resource(uri)
        }
        "prompts/list" => Ok(json!({
            "prompts": prompts::prompt_definitions()
        })),
        "prompts/get" => {
            let params = request
                .params
                .ok_or_else(|| AppError::InvalidRequest("prompts/get requires params".to_owned()))?;
            let name = params
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| AppError::InvalidRequest("prompts/get requires `name`".to_owned()))?;
            prompts::get_prompt(name, params.get("arguments"))
        }
        "notifications/initialized" => Ok(json!({})),
        other => Err(AppError::InvalidRequest(format!("unsupported method `{other}`"))),
    }
}

fn map_error_code(error: &AppError) -> i32 {
    match error {
        AppError::InvalidRequest(_) => -32600,
        AppError::Json(_) => -32700,
        _ => -32000,
    }
}
