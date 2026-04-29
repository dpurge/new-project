FROM rust:1.94.1-alpine3.22 AS builder

RUN apk add --no-cache build-base musl-dev

WORKDIR /app

COPY . .

RUN cargo build --release \
    && strip target/release/llm-api

FROM alpine:3.22

RUN apk add --no-cache ca-certificates \
    && addgroup -S app \
    && adduser -S app -G app

WORKDIR /app

COPY --from=builder /app/target/release/llm-api /usr/local/bin/service

ENV HOST=0.0.0.0 \
    PORT={{ cookiecutter.api_port }}

USER app

EXPOSE {{ cookiecutter.api_port }}

ENTRYPOINT ["/usr/local/bin/service"]
