#!/bin/bash

set -e

DIR=$(cd "$(dirname ${BASH_SOURCE[0]})"/../ && pwd)

if [[ -n "${RUSTFMT_VERSION:-}" ]]; then
    cargo fmt -- --write-mode=diff
    exit 0
fi

channel() {
    if [[ "${TRAVIS_RUST_VERSION}" == "${CHANNEL}" ]]; then
        (set -x; cargo "$@")
    fi
}

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
channel doc

cd "$DIR/finchers-urlencoded"
channel build
channel test
channel doc --no-deps

cd "$DIR/examples/form"
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
channel doc

cd "$DIR/finchers-urlencoded"
channel build
channel test
channel doc --no-deps

cd "$DIR/examples/form"
channel build
channel test
channel doc --no-deps


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
channel doc

cd "$DIR/finchers-urlencoded"
channel build
channel test
channel doc --no-deps

cd "$DIR/examples/form"
channel build
channel test
channel doc --no-deps

cd "$DIR/examples/todo"
channel build
channel test
channel doc --no-deps


# ===================================================================

# run doctest
cd "$DIR/doc"
(set -x; cargo build)
(set -x; cargo test)

if [[ ${TRAVIS_RUST_VERSION} == "stable" ]]; then
    bash "${DIR}/doc/generate.sh"
    cp -a "${DIR}/target/doc" "${DIR}/target/doc-upload/api"
    rm -f "${DIR}/target/doc-upload/api/.lock"
fi