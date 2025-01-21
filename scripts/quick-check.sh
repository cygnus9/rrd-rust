#!/bin/bash

# Quick local development check

set -exu

cd $(dirname $0)/..

cargo fmt --check
cargo build --all-targets -q
cargo clippy --all-targets -- -Dwarnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo test -q

for ex in rrd-create rrd-fetch rrd-update; do
  cargo run --example $ex > /dev/null
  rm db.rrd
done