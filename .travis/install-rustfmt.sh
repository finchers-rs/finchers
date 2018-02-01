#!/bin/bash

set -eu

if [ -z "${RUSTFMT_VERSION:-}" ]; then
    echo "Requires RUSTFMT_VERSION"
    exit 0
fi

INSTALLED="$(cargo install --list | grep rustfmt-nightly | wc -l)"
if [[ "${INSTALLED}" == 0 ]]; then
    cargo install rustfmt-nightly --version "${RUSTFMT_VERSION}" --force
fi
