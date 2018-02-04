#!/bin/bash

set -eux

DIR=$(cd $(dirname ${BASH_SOURCE[0]}) && pwd)
cd "${DIR}/.."

# remove all contents at destination directory
rm -rf target/doc-upload
mkdir -p target/doc-upload

# homepage
pushd site/ > /dev/null
bundle install
bundle exec jekyll build -d ../target/doc-upload
popd > /dev/null

# API doc
rm -rf target/doc
cargo doc --all --no-deps --features tls
cp -a target/doc target/doc-upload/api

# users guide
pushd guide/ > /dev/null
mdbook build
popd > /dev/null
