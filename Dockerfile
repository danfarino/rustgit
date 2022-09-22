FROM rust:1.63.0-slim-buster

WORKDIR /work

COPY . .

RUN export DEBIAN_FRONTEND=noninteractive && \
    apt-get update && \
    apt-get install -y pkg-config

RUN cargo build --release
