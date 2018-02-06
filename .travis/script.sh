#!/bin/bash

set -e

DIR=$(cd "$(dirname ${BASH_SOURCE[0]})"/../ && pwd)

if [[ -n "${RUSTFMT_VERSION:-}" ]]; then
    cargo fmt -- --write-mode=diff
    exit 0
fi

# imported from serde-rs/serde/travis.sh
channel() {
    if [[ -n "${TRAVIS}" ]]; then
        if [[ "${TRAVIS_RUST_VERSION}" == "${CHANNEL}" ]]; then
            pwd
            (set -x; cargo "$@")
        fi
    else
        pwd
        (set -x; cargo "+${CHANNEL}" "$@")
    fi
}

rm -rf "$DIR/target/doc"

# ===================================================================
CHANNEL="stable"
cd "$DIR/finchers"
channel build
channel test
channel build --features "tls"
channel test  --features "tls"
channel doc   --features "tls" --no-deps

cd "$DIR/finchers-derive"
channel build
channel test
channel doc --no-deps

cd "$DIR/finchers-json"
channel build
channel test
channel doc --no-deps

cd "$DIR/finchers-urlencoded"
channel build
channel test
channel doc --no-deps

cd "$DIR/examples/form"
channel build
channel test

cd "$DIR/examples/todo"
channel build
channel test


# ===================================================================
CHANNEL="beta"
cd "$DIR/finchers"
channel build
channel test
channel build --features "tls"
channel test  --features "tls"
channel doc   --features "tls" --no-deps

cd "$DIR/finchers-derive"
channel build
channel test
channel doc --no-deps

cd "$DIR/finchers-json"
channel build
channel test
channel doc --no-deps

cd "$DIR/finchers-urlencoded"
channel build
channel test
channel doc --no-deps

cd "$DIR/examples/form"
channel build
channel test

cd "$DIR/examples/todo"
channel build
channel test


# ===================================================================
CHANNEL="nightly"

cd "$DIR/finchers"
channel build
channel test
channel build --all-features
channel test --all-features
channel doc --all-features --no-deps

cd "$DIR/finchers-derive"
channel build
channel test
channel doc --no-deps

cd "$DIR/finchers-json"
channel build
channel test
channel doc --no-deps

cd "$DIR/finchers-urlencoded"
channel build
channel test
channel doc --no-deps

cd "$DIR/examples/form"
channel build
channel test

cd "$DIR/examples/todo"
channel build
channel test

# ===================================================================

# run doctest
cd "$DIR/doc"
(set -x; cargo build)
(set -x; cargo test)
