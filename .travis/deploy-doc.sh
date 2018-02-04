#!/bin/sh

set -euo pipefail

if [[ "$TRAVIS_RUST_VERSION" != "stable" ]]; then
    echo "This script should be running only on stable channel"
    exit 1
fi

get-branch() {
    if [[ "${TRAVIS_PULL_REQUEST:-}" == false ]]; then
        echo "${TRAVIS_BRANCH}"
    else
        echo "${TRAVIS_PULL_REQUEST_BRANCH}"
    fi
}

if [[ -z "${TRAVIS_BRANCH:-}" ]]; then
    echo "This script may only be running from Travis CI."
    exit 1
fi

BRANCH="$(get-branch)"
if [[ "${BRANCH}" != "master" ]]; then
    echo "The deployment should be from 'master', not '${BRANCH}'."
    exit 1
fi

# =============================================================================
DIR=$(cd $(dirname ${BASH_SOURCE[0]}) && pwd)
bash $DIR/build-doc.sh

REV="$(git rev-parse --short HEAD)"
UPSTREAM_URL="https://${GH_TOKEN}@github.com/finchers-rs/finchers.git"
USERNAME="Yusuke Sasaki"
EMAIL="yusuke.sasaki.nuem@gmail.com"

echo "Commiting docs directory to gh-pages branch"
cd target/doc-upload
git init
git remote add upstream "${UPSTREAM_URL}"
git config user.name "${USERNAME}"
git config user.email "${EMAIL}"
git add -A .
git commit -qm "Build documentation at ${TRAVIS_REPO_SLUG}@${REV}"

echo "Pushing gh-pages to GitHub"
git push -q upstream HEAD:refs/heads/gh-pages --force
