#!/bin/bash

set -e

DIR=$(cd "$(dirname ${BASH_SOURCE[0]})"/../ && pwd)
DEST="${DIR}/doc-upload"

UPSTREAM_URL="https://${GH_TOKEN}@github.com/finchers-rs/finchers-rs.github.io.git"
REV="$(git rev-parse --short HEAD)"
USERNAME="Yusuke Sasaki"
EMAIL="yusuke.sasaki.nuem@gmail.com"

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

rungit() {
    (set -x; git "$@")
}

echo "[Copying the documentation to the upstream repository...]"
rungit clone "${UPSTREAM_URL}" "${DEST}" --branch master --depth 10

# TODO: run generate.sh only on release tags
bash "${DIR}/doc/generate.sh" "$DEST"

rm -rf "${DEST}/api"
cp -a "${DIR}/target/doc" "${DEST}/api"
rm -f "${DEST}/api/.lock"

echo "[Committing...]"
cd "${DEST}"
echo ">> $(pwd)"
git config user.name "${USERNAME}"
git config user.email "${EMAIL}"
rungit add -A .
rungit commit -qm "Build documentation at ${TRAVIS_REPO_SLUG}@${REV}"

echo "[Pushing to GitHub...]"
rungit remote add upstream "${UPSTREAM_URL}"
rungit push -q upstream HEAD:refs/heads/master --force
