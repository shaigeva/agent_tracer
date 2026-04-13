#!/usr/bin/env zsh
# Build the production (release) Rust binary.
# This is separate from run_all_agent_validations.sh so the validation loop stays fast.
# Run this before committing or shipping a feature so target/release/trace is fresh.

set -e

cd "$(dirname "$0")/.."

echo "Building release binary..."
cd projects/trace_analyzer
cargo build --release

BINARY="$(pwd)/target/release/trace"
if [[ ! -x "$BINARY" ]]; then
    echo "❌ Release binary missing at $BINARY"
    exit 1
fi

SIZE=$(ls -lh "$BINARY" | awk '{print $5}')
echo "✅ Release binary built: $BINARY ($SIZE)"
