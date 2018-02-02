#!/bin/bash

set -eux

if [[ "${TRAVIS_RUST_VERSION}" == "nightly" ]]; then
    cargo build --all
    cargo test --all
else
    cargo build --all --exclude example-todo
    cargo test --all --exclude example-todo
fi

if [[ "${TRAVIS_RUST_VERSION}" == "stable" ]]; then
    cargo doc --no-deps
fi
