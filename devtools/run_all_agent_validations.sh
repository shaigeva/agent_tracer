#!/usr/bin/env zsh

# Run all validation steps for all projects by delegating to per-project scripts
# On success, prints a single summary line

set -e  # Exit on first error

# Get the root directory (absolute path)
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Running validations for all projects..."
echo ""

# Run Python project validations
echo "📦 Python project (pytest_tracer_python)..."
if ! "$ROOT_DIR/projects/pytest_tracer_python/devtools/run_all_agent_validations.sh"; then
    exit 1
fi
echo ""

# Run Rust project validations
echo "🦀 Rust project (trace_analyzer)..."
if ! "$ROOT_DIR/projects/trace_analyzer/devtools/run_all_agent_validations.sh"; then
    exit 1
fi
echo ""

# All validations passed
echo "🎉 All validations passed for all projects!"
