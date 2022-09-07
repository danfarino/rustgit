FROM rust:1.63.0-slim-buster

WORKDIR /work

COPY . .

RUN export DEBIAN_FRONTEND=noninteractive && \
    apt-get update && \
    apt-get install -y libssl-dev pkg-config

RUN cargo install cross
RUN cross build --release --target x86_64-unknown-linux-gnu
RUN cross build --release --target x86_64-apple-darwin
