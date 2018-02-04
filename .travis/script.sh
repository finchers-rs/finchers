#!/bin/bash

set -eux

if [[ "${TRAVIS_RUST_VERSION}" == "nightly" ]]; then
    cargo build --all --all-features
    cargo test --all --all-features
else
    cargo build --all --exclude example-todo --features "$STABLE_FEATURES"
    cargo test --all --exclude example-todo --features "$STABLE_FEATURES"
fi

cd guide/
mdbook test -L ../target/debug/deps
