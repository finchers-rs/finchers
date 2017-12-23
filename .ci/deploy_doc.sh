#!/bin/sh

set -euo pipefail

GH_TOKEN=$1
TRAVIS_BRANCH=$2
TRAVIS_RUST_VERSION=$3
TRAVIS_REPO_SLUG=$4
TRAVIS_PULL_REQUEST=${5-""}

HUGO_URL=https://github.com/gohugoio/hugo/releases/download/v0.26/hugo_0.26_Linux-64bit.tar.gz

if [[ "${TRAVIS_BRANCH:-}" = "master" ]] && \
   [[ "${TRAVIS_RUST_VERSION}" = "stable" ]] && \
   [[ "${TRAVIS_PULL_REQUEST}" = "false" ]]
then
    cargo doc --all-features --no-deps

    # mkdir -p ./hugo
    # curl -sL "${HUGO_URL}" | tar xzvf - -C ./hugo
    # mv ./hugo/hugo $HOME/.local/bin/

    # hugo -s site/
    # rsync -av site/public/ target/doc/

    ghp-import -n target/doc
    git push -qf "https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git" gh-pages
fi
