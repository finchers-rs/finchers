#!/bin/bash

set -eux

FEATURES=tls

if [[ "${TRAVIS_RUST_VERSION}" == "nightly" ]]; then
    cargo build --all --all-features
    cargo test --all --all-features
else
    cargo build --all --exclude example-todo --features "$FEATURES"
    cargo test --all --exclude example-todo --features "$FEATURES"
fi

if [[ "${TRAVIS_RUST_VERSION}" == "stable" ]]; then
    cd guide/
    mdbook test -L ../target/debug/deps
fi
