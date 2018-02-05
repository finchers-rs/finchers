#!/bin/bash

set -eux

if [[ -z "${RUSTFMT:-}" ]]; then
    if [[ "${TRAVIS_RUST_VERSION}" == "nightly" ]]; then
        cargo build --all --all-features
        cargo test --all --all-features
    else
        cargo build --all --exclude example-todo --features "$STABLE_FEATURES"
        cargo test --all --exclude example-todo --features "$STABLE_FEATURES"
    fi
else
    cargo fmt -- --write-mode=diff
fi
