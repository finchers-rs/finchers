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

# users guide
cd "${DIR}/doc/guide"
mdbook build
