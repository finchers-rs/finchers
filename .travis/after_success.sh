#!/bin/bash

set -eux

if [[ -n "${RUSTFMT:-}" ]]; then
    exit 1
fi

DIR=$(cd "$(dirname ${BASH_SOURCE[0]})"/../ && pwd)

bash "${DIR}/doc/generate.sh"
bash "${DIR}/.travis/deploy-doc.sh"
#  - travis-cargo --only stable coveralls --no-sudo
