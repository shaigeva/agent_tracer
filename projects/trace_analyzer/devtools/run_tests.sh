#!/usr/bin/env zsh
set -x

cd "$(dirname "$0")/.."
cargo test
