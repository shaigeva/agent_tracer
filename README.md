# pytest-tracer

A tool for helping AI coding agents understand codebases by capturing test execution traces and making them available for context gathering.

## Overview

pytest-tracer is a two-part system:

1. **pytest-tracer** (Python): A pytest plugin that captures test execution coverage with per-test granularity
2. **trace-analyzer** (Rust): A CLI tool and MCP server that indexes and queries the coverage data

### What It Does

AI coding agents struggle to understand codebases because they lack execution context. pytest-tracer helps by:

- Capturing which code paths execute for specific test scenarios
- Making it easy to find tests that exercise particular code
- Enabling agents to focus on relevant files/functions rather than searching randomly

## Quick Start (Local Development)

### Prerequisites

- Python 3.11+ with [uv](https://github.com/astral-sh/uv)
- Rust toolchain (cargo)
- jq (for JSON parsing in examples)

### 1. Install Dependencies

```bash
# Python project
cd projects/pytest_tracer_python
uv sync

# Rust project
cd projects/trace_analyzer
cargo build --release
```

### 2. Mark Your Tests as Scenarios

Add markers to tests you want to track:

```python
import pytest

@pytest.mark.scenario
@pytest.mark.behavior("authentication")
def test_successful_login():
    """
    User logs in with valid credentials

    GIVEN a registered user with email and password
    WHEN they submit valid credentials
    THEN they receive an auth token
    """
    result = login("user@example.com", "password123")
    assert result["status"] == 200


@pytest.mark.scenario
@pytest.mark.behavior("authentication")
@pytest.mark.error
def test_login_invalid_password():
    """Login fails with wrong password"""
    result = login("user@example.com", "wrong")
    assert result["status"] == 401
```

### 3. Run Tests with Coverage

```bash
cd your_project
pytest tests/ --cov=src --cov-context=test
```

### 4. Collect Scenario Metadata

```bash
cd /path/to/pytest-tracer/projects/pytest_tracer_python
uv run python -m pytest_tracer_python.cli collect \
    /path/to/your_project \
    --test-dir tests \
    -o /path/to/your_project/scenarios.json
```

### 5. Build the Trace Index

```bash
cd /path/to/pytest-tracer/projects/trace_analyzer
cargo run --release -- build \
    --coverage /path/to/your_project/.coverage \
    --scenarios /path/to/your_project/scenarios.json \
    --output /path/to/your_project/.trace-index
```

### 6. Query the Index

```bash
# List all scenarios
cargo run --release -- list --index /path/to/your_project/.trace-index

# Search for scenarios by description
cargo run --release -- search "login" --index /path/to/your_project/.trace-index

# Find scenarios covering a file
cargo run --release -- affected src/auth.py --index /path/to/your_project/.trace-index

# Find scenarios covering a specific line
cargo run --release -- affected src/auth.py:25 --index /path/to/your_project/.trace-index

# Get full coverage context for a scenario
cargo run --release -- context "tests/test_auth.py::test_login" --index /path/to/your_project/.trace-index

# Run a specific scenario with coverage
cargo run --release -- run "tests/test_auth.py::test_login"
```

### 7. Start MCP Server (for AI Agent Integration)

```bash
cargo run --release -- mcp --index /path/to/your_project/.trace-index
```

## CLI Reference

### trace build

Build an index from coverage data and scenario metadata:

```bash
trace build --coverage .coverage --scenarios scenarios.json --output .trace-index
```

### trace list

List all scenarios, optionally filtered:

```bash
trace list                           # All scenarios
trace list --errors                  # Only error scenarios
trace list --behavior authentication # Filter by behavior tag
```

### trace search

Search scenarios by description/documentation:

```bash
trace search "login"
trace search "user authentication"
```

### trace context

Get full coverage context for a scenario:

```bash
trace context "tests/test_auth.py::test_login"
```

Returns the scenario metadata plus all files and lines covered by that test.

### trace affected

Find scenarios that cover specific code:

```bash
trace affected src/auth.py           # All scenarios covering the file
trace affected src/auth.py:25        # Scenarios covering line 25
```

### trace run

Run a specific test with coverage collection:

```bash
trace run "tests/test_auth.py::test_login"
```

### trace mcp

Start the MCP server for AI agent integration:

```bash
trace mcp --index .trace-index
```

## MCP Tools

When running as an MCP server, these tools are exposed:

| Tool | Description |
|------|-------------|
| `scenario_list` | List all test scenarios |
| `scenario_list_errors` | List only error scenarios |
| `scenario_search` | Search scenarios by description |
| `scenario_context` | Get coverage context for a scenario |
| `coverage_affected_file` | Find scenarios covering a file |
| `coverage_affected_line` | Find scenarios covering a specific line |
| `scenario_run` | Run a scenario with coverage collection |

## Example Output

### scenario context

```json
{
  "scenario": {
    "id": "tests/test_auth.py::test_successful_login",
    "description": "User logs in with valid credentials",
    "behaviors": ["authentication", "session-management"],
    "outcome": "success"
  },
  "coverage": [
    {
      "path": "src/auth.py",
      "lines": [12, 13, 14, 23, 24, 25]
    },
    {
      "path": "src/models/user.py",
      "lines": [45, 46, 47]
    }
  ]
}
```

### trace affected

```json
[
  {
    "scenario": {
      "id": "tests/test_auth.py::test_login",
      "description": "User logs in with valid credentials",
      "behaviors": ["authentication"],
      "outcome": "success"
    },
    "matching_lines": [12, 13, 14]
  }
]
```

## Project Structure

```
pytest-tracer/
├── projects/
│   ├── pytest_tracer_python/    # Python: pytest markers and scenario collector
│   └── trace_analyzer/          # Rust: CLI and MCP server
├── tests/
│   └── e2e_workflow_test.sh     # End-to-end workflow test
└── devtools/                    # Cross-project validation scripts
```

## Development

### Running Validations

```bash
# All projects
./devtools/run_all_agent_validations.sh

# Python only
cd projects/pytest_tracer_python && ./devtools/run_all_agent_validations.sh

# Rust only
cd projects/trace_analyzer && ./devtools/run_all_agent_validations.sh
```

### End-to-End Test

```bash
./tests/e2e_workflow_test.sh
```

## Current Limitations

- **Line-to-function mapping**: Not yet implemented (future: Python AST analysis)
- **Incremental index updates**: Not yet implemented (must rebuild full index)
- Coverage data must be from pytest-cov with `--cov-context=test`
