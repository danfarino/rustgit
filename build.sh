#!/bin/bash

set -xeuo pipefail

image=rustgit-output
DOCKER_BUILDKIT=1 docker build . -t $image
cid="$(docker create "$image" --)"
cleanup() {
    docker rm "$cid"
}
trap cleanup EXIT
docker cp "$cid:/work/target/release/rustgit" .
