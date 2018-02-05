#!/bin/bash

set -eux

DIR=$(cd "$(dirname ${BASH_SOURCE[0]})"/../ && pwd)
UPSTREAM_URL="https://${GH_TOKEN}@github.com/finchers-rs/finchers-rs.github.io.git"
REV="$(git rev-parse --short HEAD)"
USERNAME="Yusuke Sasaki"
EMAIL="yusuke.sasaki.nuem@gmail.com"

if [[ -n "${RUSTFMT:-}" ]]; then
    exit 1
fi

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

cd "${DIR}/target/doc-upload"

echo "[Committing...]"
git init
git config user.name "${USERNAME}"
git config user.email "${EMAIL}"
git add -A .
git commit -qm "Build documentation at ${TRAVIS_REPO_SLUG}@${REV}"

echo "[Pushing to GitHub...]"
git remote add upstream "${UPSTREAM_URL}"
git push -q upstream HEAD:refs/heads/master --force
