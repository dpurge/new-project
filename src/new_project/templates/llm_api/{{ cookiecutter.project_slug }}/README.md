# {{ cookiecutter.project_name }}

{{ cookiecutter.description }}

## What this project is

This template gives you a small Rust backend for building LLM-powered features.

It already includes:

- one HTTP API: `POST /v1/chat/completions`
- support for OpenAI, Anthropic, and Ollama
- a simple agent loop that can call tools
- basic in-memory session memory
- optional MCP servers exposed as additional tools

Use this template if you want a backend service that a web app, CLI, or another
service can call.

## What is included

- `Rust` for a small compiled service
- `axum` for HTTP routing and response handling
- direct `reqwest` provider adapters for OpenAI, Anthropic, and Ollama
- an internal tool loop with a few example tools
- in-process session memory with sliding chat history and key/value facts
- optional MCP stdio clients
- `tower-http` for permissive CORS and request tracing
- a multi-stage Docker build with a small Alpine runtime image

## How it works

This template exposes an OpenAI-compatible `POST /v1/chat/completions` endpoint,
routes requests to a configurable upstream provider, and runs a simple agent loop
that can call local tools before returning the final assistant message.

That makes it a good starting point for:

- tool-using agents
- simple session memory
- provider swapping between OpenAI, Anthropic, and Ollama
- future MCP and workflow modules without a large framework
- MCP servers loaded from an optional JSON config file

## Quick start

If you only want to try the project locally, use OpenAI first:

```bash
export LLM_PROVIDER="openai"
export LLM_MODEL="gpt-4.1-mini"
export LLM_API_KEY="replace-me"
cargo run
```

Then send a test request:

```bash
curl -N http://localhost:{{ cookiecutter.http_port }}/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [
      { "role": "user", "content": "Say hello in one short sentence." }
    ],
    "stream": true
  }'
```

If that works, you can start using tools, sessions, and MCP servers.

## Environment

- `LLM_PROVIDER`: `openai`, `anthropic`, or `ollama`
- `LLM_MODEL`: model name to use for the selected provider
- `LLM_API_KEY`: upstream bearer token for OpenAI or Anthropic
- `LLM_BASE_URL`: optional provider override
- `MEMORY_MAX_MESSAGES`: number of recent messages to retain per server-side session
- `AGENT_MAX_STEPS`: maximum tool-calling loop iterations per request
- `MCP_SERVER_CONFIG`: optional path to a JSON file describing stdio MCP servers
- `HOST`: bind host, defaults to `0.0.0.0`
- `PORT`: bind port, defaults to `{{ cookiecutter.http_port }}`

Default base URLs:

- OpenAI: `https://api.openai.com/v1`
- Anthropic: `https://api.anthropic.com`
- Ollama: `http://localhost:11434`

If you are new to this project, the most important variables are:

- `LLM_PROVIDER`
- `LLM_MODEL`
- `LLM_API_KEY` for OpenAI or Anthropic
- `LLM_BASE_URL` only if you need to override the default

## Endpoints

- `GET /healthz`
- `GET /v1/tools`
- `GET /v1/mcp/servers`
- `GET /v1/memory/{session_id}`
- `DELETE /v1/memory/{session_id}`
- `POST /v1/chat/completions`

## Example request

This example stores a fact in the server-side session and then asks the model to
use it:

```bash
curl -N http://localhost:{{ cookiecutter.http_port }}/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "session_id": "demo-session",
    "stream": true,
    "messages": [
      { "role": "user", "content": "Remember that my favorite editor is Helix, then tell me the current UTC time." }
    ]
  }'
```

The generated service includes these local example tools:

- `get_time`
- `echo`
- `remember_fact`
- `recall_fact`
- `list_facts`

## When to use `session_id`

Use `session_id` when you want the backend to remember things across requests.

For example:

- a chat UI where the server should keep short-term history
- a tool-using agent that should remember facts between turns

Do not use `session_id` if your client is already sending the full conversation
every time and you do not want duplicated context.

## Project layout

- `src/main.rs`: application bootstrap and routes
- `src/config.rs`: environment-backed configuration
- `src/models.rs`: OpenAI-style API models and internal message types
- `src/providers.rs`: direct adapters for OpenAI, Anthropic, and Ollama
- `src/tools.rs`: example tools and tool schemas
- `src/mcp.rs`: MCP stdio client integration and remote tool bridging
- `src/memory.rs`: in-process session memory
- `src/agent.rs`: tool-calling loop that ties providers, tools, and memory together
- `src/state.rs`: shared app state

## Local development

OpenAI:

```bash
export LLM_PROVIDER="openai"
export LLM_MODEL="gpt-4.1-mini"
export LLM_API_KEY="replace-me"
# export MCP_SERVER_CONFIG="./mcp_servers.example.json"
cargo run
```

Anthropic:

```bash
export LLM_PROVIDER="anthropic"
export LLM_MODEL="claude-sonnet-4-20250514"
export LLM_API_KEY="replace-me"
cargo run
```

Ollama:

```bash
export LLM_PROVIDER="ollama"
export LLM_MODEL="qwen3"
export LLM_BASE_URL="http://localhost:11434"
unset LLM_API_KEY
cargo run
```

## Container workflow

Build the image:

```bash
docker build -t {{ cookiecutter.project_slug }} .
```

Run the container:

```bash
docker run --rm -p {{ cookiecutter.http_port }}:{{ cookiecutter.http_port }} \
  -e LLM_PROVIDER \
  -e LLM_BASE_URL \
  -e LLM_MODEL \
  -e LLM_API_KEY \
  {{ cookiecutter.project_slug }}
```

## Working with llm-chat

The companion `llm-chat` template is already wired to call `/v1/chat/completions`.
Run this service on port `{{ cookiecutter.http_port }}` and point the chat container's
`LLM_API_ORIGIN` environment variable at it.

## MCP servers

The generated template can treat stdio MCP servers as additional tools. Server
definitions live in a JSON file referenced by `MCP_SERVER_CONFIG`.

Example:

```json
[
  {
    "name": "filesystem",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-filesystem", "."]
  }
]
```

This file is optional. If you do not need MCP yet, ignore it.

Remote MCP tools are exposed to the model with names like:

```text
mcp__filesystem__read_file
```

That prefix keeps local and remote tool names distinct and makes routing explicit.

## Memory model

If you send a `session_id`, the server stores recent user/assistant turns plus any
facts written through the example memory tools. This keeps the template simple, but
it also means memory is in-process and ephemeral. The generated code is structured
so you can replace it with Redis, PostgreSQL, or a vector store later.

In plain terms:

- client-managed conversation history
- server-managed history via `session_id`

Mixing both is possible, but it can duplicate context if you are not careful.
