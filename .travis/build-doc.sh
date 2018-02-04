#!/bin/bash

set -eux

DIR=$(cd "$(dirname ${BASH_SOURCE[0]})"/../ && pwd)
REV="$(git rev-parse --short HEAD)"
USERNAME="Yusuke Sasaki"
EMAIL="yusuke.sasaki.nuem@gmail.com"

# remove all contents at destination directory
cd "${DIR}"
rm -rf target/doc-upload
mkdir -p target/doc-upload

# homepage
cd "${DIR}/site"
bundle install
bundle exec jekyll build -d "${DIR}/target/doc-upload"

# API doc
cd "${DIR}"
rm -rf ./target/doc
cargo doc --all --no-deps --features "$STABLE_FEATURES"
cp -a ./target/doc ./target/doc-upload/api

# users guide
cd "${DIR}/guide"
mdbook build

# commit
cd "${DIR}/target/doc-upload"
git init
git config user.name "${USERNAME}"
git config user.email "${EMAIL}"
git add -A .
git commit -qm "Build documentation at ${TRAVIS_REPO_SLUG}@${REV}"
