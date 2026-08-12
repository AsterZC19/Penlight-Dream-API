# syntax=docker/dockerfile:1

FROM rust:1.97-alpine AS builder
WORKDIR /app
RUN apk add --no-cache gcc musl-dev
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked \
    && cp target/release/penlight-dream-api /penlight-dream-api

FROM alpine:3.21
RUN adduser -D -H -u 10001 penlight
COPY --from=builder /penlight-dream-api /usr/local/bin/penlight-dream-api
USER penlight
EXPOSE 8080
ENV HOST=0.0.0.0
ENTRYPOINT ["penlight-dream-api"]
