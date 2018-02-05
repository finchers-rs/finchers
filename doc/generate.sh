#!/bin/bash

set -eux

DIR=$(cd "$(dirname ${BASH_SOURCE[0]})"/../ && pwd)
STABLE_FEATURES="${STABLE_FEATURES:-}"

# remove all contents at destination directory
cd "${DIR}"
rm -rf target/doc-upload
mkdir -p target/doc-upload

# homepage
cd "${DIR}/doc/site"
bundle install
bundle exec jekyll build

# API doc
cd "${DIR}"
rm -rf ./target/doc
cargo doc --all --no-deps --features "$STABLE_FEATURES"
cp -a ./target/doc ./target/doc-upload/api
rm -f ./target/doc-upload/api/.lock

# users guide
cd "${DIR}/doc/guide"
mdbook build
