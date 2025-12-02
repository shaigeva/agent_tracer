#!/bin/bash
# End-to-end workflow test for pytest-tracer
#
# This script demonstrates the full workflow:
# 1. Run pytest with coverage on a sample project
# 2. Collect scenario metadata
# 3. Build the trace index
# 4. Query the index
#
# Usage: ./tests/e2e_workflow_test.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
SAMPLE_PROJECT="$ROOT_DIR/projects/pytest_tracer_python/tests/fixtures/sample_project"
WORK_DIR=$(mktemp -d)

echo "=== pytest-tracer End-to-End Workflow Test ==="
echo ""
echo "Working directory: $WORK_DIR"
echo "Sample project: $SAMPLE_PROJECT"
echo ""

# Cleanup on exit
cleanup() {
    rm -rf "$WORK_DIR"
}
trap cleanup EXIT

# Step 1: Run pytest with coverage
echo "Step 1: Running pytest with coverage..."
cd "$SAMPLE_PROJECT"
uv run pytest tests/ \
    --cov=src \
    --cov-context=test \
    --cov-report= \
    -q
mv .coverage "$WORK_DIR/.coverage"
echo "  ✓ Coverage data generated"
echo ""

# Step 2: Collect scenario metadata
echo "Step 2: Collecting scenario metadata..."
cd "$ROOT_DIR/projects/pytest_tracer_python"
uv run python -m pytest_tracer_python.cli collect \
    "$SAMPLE_PROJECT" \
    --test-dir tests \
    -o "$WORK_DIR/scenarios.json"
echo "  ✓ Scenario metadata collected"
echo ""

# Step 3: Build the trace index
echo "Step 3: Building trace index..."
cd "$ROOT_DIR/projects/trace_analyzer"
cargo run -q -- build \
    --coverage "$WORK_DIR/.coverage" \
    --scenarios "$WORK_DIR/scenarios.json" \
    --output "$WORK_DIR/.trace-index"
echo ""

# Step 4: Query the index
echo "Step 4: Querying the index..."
echo ""

echo "  4a. List all scenarios:"
cargo run -q -- list --index "$WORK_DIR/.trace-index" | head -20
echo "  ..."
echo ""

echo "  4b. Search for 'login' scenarios:"
cargo run -q -- search "login" --index "$WORK_DIR/.trace-index"
echo ""

echo "  4c. Find scenarios covering auth.py:"
cargo run -q -- affected "auth.py" --index "$WORK_DIR/.trace-index" | head -30
echo "  ..."
echo ""

echo "  4d. Get context for a specific scenario:"
SCENARIO_ID=$(cargo run -q -- list --index "$WORK_DIR/.trace-index" | jq -r '.[0].id')
echo "  Scenario: $SCENARIO_ID"
cargo run -q -- context "$SCENARIO_ID" --index "$WORK_DIR/.trace-index"
echo ""

echo "=== End-to-End Test Completed Successfully ==="
