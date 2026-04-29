use crate::{
    error::AppError,
    memory::MemoryStore,
    models::{
        AgentMessage, ChatCompletionRequest, ProviderRequest, ToolResultMessage, UsageStats,
    },
    providers::ProviderClient,
    tools::{ToolContext, ToolRegistry},
};

pub async fn run_agent(
    provider: &ProviderClient,
    memory: &MemoryStore,
    tools: &ToolRegistry,
    request: &ChatCompletionRequest,
) -> Result<AgentRunResult, AppError> {
    let session_id = request.session_id.clone();
    let mut messages = Vec::new();

    if let Some(session_id) = &session_id {
        messages.extend(memory.recent_messages(session_id).await);
    }
    messages.extend(request.agent_messages());

    let tool_definitions = tools.definitions();
    let model_override = request.model.clone();
    let mut final_usage = UsageStats::default();

    for _ in 0..provider.config().agent_max_steps {
        let provider_request = ProviderRequest {
            model_override: model_override.clone(),
            messages: messages.clone(),
            tools: tool_definitions.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };

        let response = provider.generate(provider_request).await?;
        final_usage.accumulate(&response.usage);

        if response.tool_calls.is_empty() {
            let assistant_message = AgentMessage::assistant(response.content.clone());
            messages.push(assistant_message.clone());

            if let Some(session_id) = &session_id {
                memory
                    .append_exchange(
                        session_id,
                        request.user_visible_messages(),
                        assistant_message,
                    )
                    .await;
            }

            return Ok(AgentRunResult {
                model: response.model,
                message: response.content,
                usage: final_usage,
            });
        }

        messages.push(AgentMessage::assistant_with_tools(
            response.content,
            response.tool_calls.clone(),
        ));

        for call in response.tool_calls {
            let tool_result = tools
                .execute(
                    &call,
                    ToolContext {
                        session_id: session_id.clone(),
                        memory,
                    },
                )
                .await?;

            messages.push(AgentMessage::tool(ToolResultMessage {
                tool_call_id: call.id.clone(),
                tool_name: call.name.clone(),
                content: tool_result,
            }));
        }
    }

    Err(AppError::AgentLoopExceeded(provider.config().agent_max_steps))
}

#[derive(Debug)]
pub struct AgentRunResult {
    pub model: String,
    pub message: String,
    pub usage: UsageStats,
}
