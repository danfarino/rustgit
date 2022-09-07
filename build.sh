#!/bin/bash

set -xeuo pipefail

image=rustgit-output
DOCKER_BUILDKIT=1 docker build . -t $image --volume /var/run/docker.sock:/var/run/docker.sock
# cid="$(docker create "$image" --)"
# cleanup() {
#     docker rm "$cid"
# }
# trap cleanup EXIT
# docker cp "$cid:/work/target/release/rustgit-x86_64-unknown-linux-gnu" .
# docker cp "$cid:/work/target/release/rustgit-x86_64-apple-darwin" .
