# {{ cookiecutter.project_name }}

{{ cookiecutter.description }}

## What this project is

This template gives you a complete starter stack:

- `llm-api` for the backend
- `llm-chat` for the frontend
- one `compose.yaml` that runs both together

Use this template if you want something runnable quickly without wiring the API
and UI together by hand.

## Services

- `llm-api`: Rust `axum` service with provider adapters, example tools, and in-process session memory
- `llm-chat`: SolidJS chat frontend served by Nginx and reverse-proxied to `llm-api`

## Layout

- `compose.yaml`: runs both services together
- `llm-api/`: Rust agent-ready backend
- `llm-chat/`: static frontend and Nginx runtime proxy

## Quick start

Set the provider variables and start both services:

```bash
export LLM_PROVIDER="openai"
export LLM_MODEL="gpt-4.1-mini"
export LLM_API_KEY="replace-me"
docker compose up --build
```

Then open the chat UI at `http://localhost:{{ cookiecutter.chat_port }}`.

That is the simplest way to try the full stack.

## Environment

The stack expects these variables when you run Compose:

- `LLM_PROVIDER`: `openai`, `anthropic`, or `ollama`
- `LLM_MODEL`: model name for the selected provider
- `LLM_API_KEY`: bearer token for OpenAI or Anthropic
- `LLM_BASE_URL`: optional provider override
- `MEMORY_MAX_MESSAGES`: number of recent server-side messages to retain per session
- `AGENT_MAX_STEPS`: maximum tool-calling loop iterations

If you are using Ollama, you usually also want:

```bash
export LLM_PROVIDER="ollama"
export LLM_MODEL="qwen3"
export LLM_BASE_URL="http://host.docker.internal:11434"
```

## Notes

The browser only talks to `llm-chat`. Nginx forwards `/v1/*` to `llm-api`, and
`llm-api` keeps the provider API key on the server side.

The generated backend includes example tools:

- `get_time`
- `echo`
- `remember_fact`
- `recall_fact`
- `list_facts`

It also supports stdio MCP servers through an optional `MCP_SERVER_CONFIG` JSON
file inside `llm-api/`, with remote tools exposed under names like
`mcp__filesystem__read_file`.

Use `session_id` in chat requests if you want the backend to maintain its own
conversation history and memory facts. The default `llm-chat` app continues to
send client-managed history and does not require server-side sessions.

If you are new to the project, start without MCP first. Once the basic chat flow
works, add MCP servers to `llm-api/` if you need external tools.
