FROM rust:1.62.1-slim-buster

WORKDIR /work

COPY . .

RUN export DEBIAN_FRONTEND=noninteractive && \
    apt-get update && \
    apt-get install -y libssl-dev pkg-config

RUN cargo build --release
