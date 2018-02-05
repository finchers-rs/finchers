#!/bin/bash

set -eux

DIR=$(cd "$(dirname ${BASH_SOURCE[0]})"/../ && pwd)

if [[ -n "${RUSTFMT:-}" ]]; then
    cargo fmt -- --write-mode=diff
    exit 0
fi

case "${TRAVIS_RUST_VERSION:-}" in
"nightly")
    cargo build --all --all-features
    cargo test --all --all-features
    cargo doc --all --all-features --no-deps
    ;;
*)
    cargo build --all --exclude example-todo --features "$STABLE_FEATURES"
    cargo test --all --exclude example-todo --features "$STABLE_FEATURES"
    cargo doc --all --features "$STABLE_FEATURES" --no-deps
    ;;
esac

# doctest
cargo test --manifest-path "$DIR/doc/Cargo.toml"

bash "${DIR}/doc/generate.sh"
cp -a "${DIR}/target/doc" "${DIR}/target/doc-upload/api"
rm -f "${DIR}/target/doc-upload/api/.lock"
