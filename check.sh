#!/usr/bin/env bash
# This scripts runs various CI-like checks in a convenient way.
set -eux

cargo check --all-targets --workspace
cargo check --all-features --lib --target wasm32-unknown-unknown --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --  -D warnings -W clippy::all
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
trunk build
