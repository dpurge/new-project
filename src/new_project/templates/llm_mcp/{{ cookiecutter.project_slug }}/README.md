# {{ cookiecutter.project_name }}

{{ cookiecutter.description }}

## What this project is

This template gives you a small Rust MCP server.

It is useful when you want to expose tools, resources, and prompts to an MCP
client such as an agent host, IDE, or another AI application.

## What is included

- `Rust` for a small compiled server
- `axum` for HTTP transports
- `tokio` for async runtime, stdio loops, and session fan-out
- a shared JSON-RPC handler used by all transports
- a multi-stage Docker build with a small Alpine runtime image

## Supported transports

- `stdio`
- `streamable-http`
- `http`

Important note:

- `streamable-http` follows the current MCP transport shape with `GET` and `POST` on `/mcp`
- `http` in this template means the legacy `HTTP+SSE` compatibility transport from the
  November 5, 2024 MCP revision, exposed as `GET /sse` and `POST /messages`

If you are not sure which one to choose, start with:

- `stdio` for local subprocess integrations
- `streamable-http` for new networked integrations
- `http` only when you need legacy compatibility

## Example features

The template includes a small MCP surface with:

- `initialize`
- `ping`
- `tools/list`
- `tools/call`
- `resources/list`
- `resources/read`
- `prompts/list`
- `prompts/get`

Example tools:

- `echo`
- `get_time`

Example resources:

- `memo://welcome`
- `memo://architecture`

Example prompts:

- `summarize-notes`
- `triage-issue`

## Quick start

The easiest first run is stdio mode:

```bash
MCP_TRANSPORT=stdio cargo run
```

If you want to test it over HTTP instead:

```bash
MCP_TRANSPORT=all HOST=127.0.0.1 PORT={{ cookiecutter.http_port }} cargo run
```

## Environment

- `MCP_TRANSPORT`: `stdio`, `streamable-http`, `http`, or `all`
- `HOST`: bind host for HTTP transports, defaults to `127.0.0.1`
- `PORT`: bind port for HTTP transports, defaults to `{{ cookiecutter.http_port }}`
- `ALLOWED_ORIGINS`: comma-separated allowed origins for HTTP transports
- `MCP_AUTH_TOKEN`: optional bearer token required on HTTP transports

`all` serves both HTTP transports from the same process:

- current MCP endpoint: `/mcp`
- legacy compatibility endpoints: `/sse` and `/messages`

If `MCP_AUTH_TOKEN` is set, HTTP clients must send:

```text
Authorization: Bearer <token>
```

## Local development

Run stdio mode:

```bash
MCP_TRANSPORT=stdio cargo run
```

Run HTTP mode:

```bash
MCP_TRANSPORT=all HOST=127.0.0.1 PORT={{ cookiecutter.http_port }} cargo run
```

Require bearer auth on HTTP endpoints:

```bash
MCP_TRANSPORT=all \
HOST=127.0.0.1 \
PORT={{ cookiecutter.http_port }} \
MCP_AUTH_TOKEN=replace-me \
cargo run
```

If you are new to MCP, test `stdio` first because it removes network and auth
variables from the setup.

## Container workflow

Build the image:

```bash
docker build -t {{ cookiecutter.project_slug }} .
```

Run the HTTP server:

```bash
docker run --rm -p {{ cookiecutter.http_port }}:{{ cookiecutter.http_port }} \
  -e MCP_TRANSPORT=all \
  -e HOST=0.0.0.0 \
  {{ cookiecutter.project_slug }}
```

The container is mainly useful for HTTP transports. `stdio` is usually easiest to
run directly from the host machine.

## Layout

- `src/main.rs`: bootstrap and transport selection
- `src/config.rs`: environment-backed configuration
- `src/error.rs`: application and JSON-RPC errors
- `src/jsonrpc.rs`: shared JSON-RPC request and response types
- `src/server.rs`: MCP method handling and session management
- `src/tools.rs`: example tools
- `src/resources.rs`: example resources
- `src/prompts.rs`: example prompts
