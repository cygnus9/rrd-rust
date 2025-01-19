#!/bin/bash

# Quick local development check

set -exu

cargo fmt --check
cargo build --all-targets -q
cargo clippy --all-targets -- -Dwarnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo test -q
