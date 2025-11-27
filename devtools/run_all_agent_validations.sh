#!/usr/bin/env zsh

# Run all validation steps for all projects and only print output on failure
# On success, prints a single summary line

set -e  # Exit on first error

TEMP_OUTPUT=$(mktemp)
trap "rm -f $TEMP_OUTPUT" EXIT

# Get the root directory (absolute path)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Running validations for all projects..."
echo ""

# Run Python project validations
echo "📦 Python project (pytest_tracer_python)..."
cd "$ROOT_DIR/projects/pytest_tracer_python"

if ! uv run ruff check --fix . > "$TEMP_OUTPUT" 2>&1; then
    echo "❌ Python: Ruff auto-fix failed:"
    cat "$TEMP_OUTPUT"
    exit 1
fi

if ! uv run ruff format . > "$TEMP_OUTPUT" 2>&1; then
    echo "❌ Python: Ruff formatting failed:"
    cat "$TEMP_OUTPUT"
    exit 1
fi

if ! uv run ruff check . > "$TEMP_OUTPUT" 2>&1; then
    echo "❌ Python: Ruff linting failed:"
    cat "$TEMP_OUTPUT"
    exit 1
fi

if ! uv run ruff format --diff . > "$TEMP_OUTPUT" 2>&1; then
    echo "❌ Python: Code formatting check failed:"
    cat "$TEMP_OUTPUT"
    exit 1
fi

if ! uv run ty check > "$TEMP_OUTPUT" 2>&1; then
    echo "❌ Python: Type checking failed:"
    cat "$TEMP_OUTPUT"
    exit 1
fi

if ! uv run pytest > "$TEMP_OUTPUT" 2>&1; then
    echo "❌ Python: Tests failed:"
    cat "$TEMP_OUTPUT"
    exit 1
fi

echo "✅ Python validations passed"
echo ""

# Run Rust project validations
echo "🦀 Rust project (trace_analyzer)..."
cd "$ROOT_DIR/projects/trace_analyzer"

if ! cargo fmt -- --check > "$TEMP_OUTPUT" 2>&1; then
    echo "❌ Rust: Formatting check failed:"
    cat "$TEMP_OUTPUT"
    echo ""
    echo "Run 'cargo fmt' to fix formatting issues"
    exit 1
fi

if ! cargo clippy -- -D warnings > "$TEMP_OUTPUT" 2>&1; then
    echo "❌ Rust: Clippy failed:"
    cat "$TEMP_OUTPUT"
    exit 1
fi

if ! cargo test > "$TEMP_OUTPUT" 2>&1; then
    echo "❌ Rust: Tests failed:"
    cat "$TEMP_OUTPUT"
    exit 1
fi

if ! cargo build > "$TEMP_OUTPUT" 2>&1; then
    echo "❌ Rust: Build failed:"
    cat "$TEMP_OUTPUT"
    exit 1
fi

echo "✅ Rust validations passed"
echo ""

# All validations passed
echo "🎉 All validations passed for all projects!"
