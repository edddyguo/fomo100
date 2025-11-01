#!/usr/bin/env bash
set -e

# Format Cargo.toml
cargo sort --workspace ./

# Format sql files

# Format rust files
cargo fix --allow-dirty --allow-staged --tests
## cargo clippy --workspace --fix --allow-staged --allow-dirty --tests --all-targets --all-features -- -D warnings
cargo fmt
