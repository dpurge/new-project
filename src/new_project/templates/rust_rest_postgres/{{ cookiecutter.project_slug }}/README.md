# {{ cookiecutter.project_name }}

{{ cookiecutter.description }}

## Stack

- `axum` for routing, extractors, JSON responses, and server bootstrap
- `tokio` for the async runtime
- `serde` for request and response serialization
- `sqlx` for PostgreSQL access and embedded migrations
- `tower-http` for CORS and request tracing middleware

## Layout

- `src/main.rs`: bootstraps config, database pool, migrations, and HTTP server
- `src/config.rs`: environment-backed application configuration
- `src/state.rs`: shared application state
- `src/models.rs`: request and response models
- `src/error.rs`: HTTP-facing application errors
- `src/routes/`: route handlers
- `migrations/`: SQL migrations embedded into the binary
- `Dockerfile`: multi-stage container build with a small Alpine runtime image
- `compose.yaml`: local app + PostgreSQL stack

## Endpoints

- `GET /healthz`
- `GET /api/todos`
- `GET /api/todos/{id}`
- `POST /api/todos`

Create a todo with:

```bash
curl -X POST http://localhost:{{ cookiecutter.http_port }}/api/todos \
  -H 'content-type: application/json' \
  -d '{"title":"Write the first handler"}'
```

## Local development

Copy the example environment file:

```bash
cp .env.example .env
```

Start only PostgreSQL:

```bash
docker compose up -d postgres
```

Run the service:

```bash
cargo run
```

The service runs migrations on startup using `sqlx::migrate!()`.

## Container workflow

Build the image:

```bash
docker build -t {{ cookiecutter.project_slug }} .
```

Run the full stack:

```bash
docker compose up --build
```
