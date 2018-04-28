#!/bin/bash

set -e

DIR=$(cd "$(dirname ${BASH_SOURCE[0]})"/../ && pwd)
DEST="${1:-"$DIR/doc-upload"}"

cd "${DIR}/doc"
echo ">> $(pwd)"
set -x
bundle install
bundle exec jekyll build -d "$DEST"
# mdbook build -d "$DEST/guide" ./guide/
