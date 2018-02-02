#!/bin/bash

set -eux

INSTALLED="$(cargo install --list | grep rustfmt-nightly | wc -l)"
if [[ "${INSTALLED}" == 0 ]]; then
    cargo install rustfmt-nightly --version "${RUSTFMT_VERSION}" --force
fi
