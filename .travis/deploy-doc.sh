#!/bin/sh

set -euo pipefail

DIR=$(cd "$(dirname ${BASH_SOURCE[0]})"/../ && pwd)
UPSTREAM_URL="https://${GH_TOKEN}@github.com/finchers-rs/finchers-rs.github.io.git"

if [[ -z "${TRAVIS_BRANCH:-}" ]]; then
    echo "[This script may only be running from Travis CI]"
    exit 1
fi

if [[ "$TRAVIS_RUST_VERSION" != "stable" ]]; then
    echo "[This script should be running only on stable channel]"
    exit 1
fi

if [[ "${TRAVIS_PULL_REQUEST:-}" == false ]]; then
    BRANCH="${TRAVIS_BRANCH:-}"
else
    BRANCH="${TRAVIS_PULL_REQUEST_BRANCH:-}"
fi
if [[ "${BRANCH}" != "master" ]]; then
    echo "[The deployment should be from 'master', not '${BRANCH}']"
    exit 1
fi

echo "[Pushing to GitHub...]"
cd "${DIR}/target/doc-upload"
git remote add upstream "${UPSTREAM_URL}"
git push -q upstream HEAD:refs/heads/master --force
