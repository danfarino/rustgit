FROM rust:1.81-slim-bullseye

RUN export DEBIAN_FRONTEND=noninteractive && \
    apt-get update && \
    apt-get install -y pkg-config

WORKDIR /work

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release
