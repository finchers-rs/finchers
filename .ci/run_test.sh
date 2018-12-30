#!/bin/bash

set -ex

rustc --version
cargo --version

if cargo fmt --version >/dev/null 2>&1; then
    cargo fmt -- --check
fi

if cargo clippy --version >/dev/null 2>&1; then
    cargo clippy --all --all-targets
fi

cargo test --all
