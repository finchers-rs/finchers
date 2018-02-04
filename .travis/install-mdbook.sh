#!/bin/bash

set -eux

INSTALLED="$(cargo install --list | grep mdbook | wc -l)"
if [[ "${INSTALLED}" == 0 ]]; then
    cargo install mdbook --version "${MDBOOK_VERSION}" --force
fi
