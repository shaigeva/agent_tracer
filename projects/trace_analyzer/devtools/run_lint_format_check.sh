#!/usr/bin/env zsh
set -x

cd "$(dirname "$0")/.."
cargo fmt -- --check
cargo clippy -- -D warnings
