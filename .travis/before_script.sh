#!/bin/bash

set -eux

ensure_installed() {
    local installed="$(cargo install --list | grep \"$1\" | wc -l)"
    if [[ "${installed}" == 0 ]]; then
        cargo install "$1" --version "$2" --force
    fi
}

if [[ -n "${RUSTFMT:-}" ]]; then
    ensure_installed rustfmt-nightly "${RUSTFMT_VERSION}"
else
    ensure_installed mdbook "${MDBOOK_VERSION}"
fi

