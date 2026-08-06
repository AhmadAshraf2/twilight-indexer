# syntax=docker/dockerfile:1
# Cargo.lock is v4 (cargo >= 1.78); litemap 0.8 / zerofrom 0.1.6 need rustc >= 1.81
FROM rust:1.87-bookworm AS build
# protobuf-compiler: build.rs (prost-build); libpq-dev: diesel links libpq
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler libpq-dev pkg-config libssl-dev git \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
# --locked: git deps (zkos-rust develop) stay pinned to Cargo.lock revisions
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked \
    && cp target/release/twilight_indexer /usr/local/bin/twilight_indexer

FROM debian:bookworm-slim
# libpq5: diesel links libpq even in DECODE_ONLY mode; curl is for the healthcheck
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 libssl3 ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
ENV API_HOST=0.0.0.0 \
    API_PORT=8449 \
    DECODE_ONLY=true
EXPOSE 8449
COPY --from=build /usr/local/bin/twilight_indexer /usr/local/bin/twilight_indexer
CMD ["twilight_indexer"]
