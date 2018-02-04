#!/bin/bash

set -eux

rm -rf target/doc

# rustdoc
cargo doc --all --no-deps --features tls

# homepage
cp site/index.html target/doc/

# mdbook
cd guide/
mdbook build
