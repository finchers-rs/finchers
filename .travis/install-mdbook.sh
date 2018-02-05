#!/bin/bash

set -eux

URL="https://github.com/rust-lang-nursery/mdBook/releases/download/v${MDBOOK_VERSION}/mdbook-v${MDBOOK_VERSION}-x86_64-unknown-linux-gnu.tar.gz"

mkdir -p "$HOME/.local/bin"
cd "$HOME/.local/bin"

curl -sL "$URL" | tar xzf -
