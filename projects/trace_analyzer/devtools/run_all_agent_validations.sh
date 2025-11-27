#!/usr/bin/env zsh

# Run all Rust validation steps and only print output on failure
# On success, prints a single summary line

set -e  # Exit on first error

TEMP_OUTPUT=$(mktemp)
trap "rm -f $TEMP_OUTPUT" EXIT

SUCCESS=true
FAILED_STEP=""

# Function to run a command and capture output
run_step() {
    local step_name="$1"
    shift

    if "$@" > "$TEMP_OUTPUT" 2>&1; then
        return 0
    else
        SUCCESS=false
        FAILED_STEP="$step_name"
        return 1
    fi
}

# Change to project directory
cd "$(dirname "$0")/.."

# Run each validation step
run_step "cargo-fmt" cargo fmt -- --check || {
    echo "❌ Cargo formatting check failed:"
    cat "$TEMP_OUTPUT"
    echo ""
    echo "Run 'cargo fmt' to fix formatting issues"
    exit 1
}

run_step "cargo-clippy" cargo clippy -- -D warnings || {
    echo "❌ Cargo clippy failed:"
    cat "$TEMP_OUTPUT"
    exit 1
}

run_step "cargo-test" cargo test || {
    echo "❌ Tests failed:"
    cat "$TEMP_OUTPUT"
    exit 1
}

run_step "cargo-build" cargo build || {
    echo "❌ Build failed:"
    cat "$TEMP_OUTPUT"
    exit 1
}

# All validations passed
echo "✅ All Rust validations passed (fmt, clippy, test, build)"
