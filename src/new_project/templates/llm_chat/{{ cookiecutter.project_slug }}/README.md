# {{ cookiecutter.project_name }}

{{ cookiecutter.description }}

## What this project is

This template gives you a small chat frontend that talks to an LLM backend over
`/v1/chat/completions`.

It is meant to pair with the `llm-api` template, but it can work with any backend
that offers a compatible endpoint.

## What is included

- `TypeScript` for browser-facing API and UI safety
- `SolidJS` for fine-grained updates while streaming token-by-token output
- `Vite` for a fast static build
- `nginx:alpine` for a small runtime image that can reverse proxy `/v1/*` to `llm-api`

## How it works

The app sends OpenAI-style `POST /v1/chat/completions` requests with `stream: true`.
At runtime, Nginx forwards `/v1/*` to `LLM_API_ORIGIN`, so the browser never needs
the upstream LLM key. Point `LLM_API_ORIGIN` at the companion `llm-api` service
and the two templates work together without changing application code.

## Quick start

1. Start an API backend such as `llm-api` on port `8080`.
2. Install dependencies.
3. Start the frontend dev server.

```bash
npm install
npm run dev
```

Then open `http://localhost:3000`.

If you already have a working backend, this is usually the fastest way to verify
the UI.

## Layout

- `src/app.tsx`: chat UI and SSE stream handling
- `src/main.tsx`: app bootstrap
- `src/styles.css`: page styling
- `nginx.conf.template`: runtime reverse-proxy configuration
- `Dockerfile`: Node build stage plus small Nginx runtime image

## Local development

Install dependencies and start the Vite dev server:

```bash
npm install
npm run dev
```

The Vite dev server runs on `http://localhost:3000`.

Note: in local dev, Vite serves the frontend. In containers, Nginx serves the
built static files and proxies `/v1/*`.

## Build

```bash
npm run build
```

## Container workflow

Build the image:

```bash
docker build -t {{ cookiecutter.project_slug }} .
```

Run it against a local `llm-api` on port `8080`:

```bash
docker run --rm -p 3000:8080 \
  -e LLM_API_ORIGIN=http://host.docker.internal:8080 \
  {{ cookiecutter.project_slug }}
```

If both apps share the same Docker Compose network, use `http://llm-api:8080`
instead.
