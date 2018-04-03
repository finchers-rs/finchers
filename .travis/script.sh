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
channel test --all
channel test --all --all-features

# ===================================================================
CHANNEL="beta"
channel test --all
channel test --all --all-features

# ===================================================================
CHANNEL="nightly"
channel test --all
channel test --all --all-features

# ===================================================================
CHANNEL="stable"
channel doc --all-features --no-deps
