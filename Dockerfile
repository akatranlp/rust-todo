################################
# Tools
################################
FROM minio/mc:RELEASE.2025-07-21T05-28-08Z AS tools-mc


FROM rust:slim-bookworm AS base
RUN cargo install cargo-chef --version ^0.1

################################
# Planner
################################
FROM base AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

################################
# Builder
################################
FROM base AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    # For some reason GCC fails to compile valhalla so we use clang instead
    clang \
    # Valhalla build dependencies
    build-essential \
    cmake \
    libboost-dev \
    liblz4-dev \
    libprotobuf-dev \
    protobuf-compiler \
    zlib1g-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

ENV CC=clang CXX=clang++

WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin server

################################
# Runner
################################
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    libprotobuf-lite32 \
    && apt clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=tools-mc /usr/bin/mc /usr/bin/mc

COPY ./entrypoint.sh /entrypoint.sh
RUN useradd -ms /bin/bash runner
USER runner
WORKDIR /app
COPY --from=builder /app/target/release/server /app/

EXPOSE 8000

ENTRYPOINT ["/entrypoint.sh", "/app/server"]

