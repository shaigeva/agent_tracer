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

## Quick Start

### Prerequisites

- Python 3.11+ with [uv](https://github.com/astral-sh/uv)
- Rust toolchain (cargo) - for building the trace analyzer once

### 1. Build the Rust CLI (one-time setup)

```bash
# Clone pytest-tracer somewhere on your machine
git clone <pytest-tracer-repo> ~/tools/pytest-tracer

# Build the Rust CLI
cd ~/tools/pytest-tracer/projects/trace_analyzer
cargo build --release

# The binary is now at:
# ~/tools/pytest-tracer/projects/trace_analyzer/target/release/trace
```

### 2. Add pytest-tracer to Your Project

In your project directory:

```bash
cd your_project

# Add pytest-tracer as a dev dependency (from local path)
uv add --dev ~/tools/pytest-tracer/projects/pytest_tracer_python

# This also adds pytest and pytest-cov as dependencies
```

### 3. Mark Your Tests as Scenarios

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

### 4. Run Tests with Coverage

```bash
# Run pytest with per-test coverage context
uv run pytest tests/ --cov=src --cov-context=test
```

### 5. Collect Scenario Metadata

```bash
# Collect scenario metadata from your tests
uv run pytest-tracer collect . -o scenarios.json
```

### 6. Collect Call Traces (optional, for flame graphs)

```bash
# Collect call traces using sys.monitoring (Python 3.12+)
uv run pytest-tracer trace . -o call_traces.json
```

### 7. Build the Trace Index

```bash
# Use the Rust CLI to build the index
~/tools/pytest-tracer/projects/trace_analyzer/target/release/trace build \
    --coverage .coverage \
    --scenarios scenarios.json \
    --call-traces call_traces.json \
    --output .trace-index
```

### 8. Query the Index

```bash
# Set up an alias for convenience (add to your shell config)
alias trace="~/tools/pytest-tracer/projects/trace_analyzer/target/release/trace"

# List all scenarios
trace list --index .trace-index

# Search for scenarios by description
trace search "login" --index .trace-index

# Find scenarios covering a file
trace affected src/auth.py --index .trace-index

# Find scenarios covering a specific line
trace affected src/auth.py:25 --index .trace-index

# Get full coverage context for a scenario
trace context "tests/test_auth.py::test_login" --index .trace-index

# Run a specific scenario with coverage
trace run "tests/test_auth.py::test_login"
```

### 8. Start MCP Server (for AI Agent Integration)

```bash
trace mcp --index .trace-index
```

To configure with Claude Desktop, add to `~/.config/claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "trace-analyzer": {
      "command": "/Users/you/tools/pytest-tracer/projects/trace_analyzer/target/release/trace",
      "args": ["mcp", "--index", "/path/to/your/project/.trace-index"]
    }
  }
}
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

### trace flamegraph

Generate flame graph or call-chain sequence diagram from call traces. Requires building the index with `--call-traces`.

```bash
# Interactive SVG flame graph (open the .svg file in any browser)
trace flamegraph "tests/test_auth.py::test_login" --format svg > flamegraph.svg
open flamegraph.svg  # opens in default browser

# Folded stacks format (for speedscope, flamegraph.pl)
trace flamegraph "tests/test_auth.py::test_login"

# Mermaid sequence diagram showing call chain between files
trace flamegraph "tests/test_auth.py::test_login" --format mermaid
```

The **SVG format** is fully self-contained and interactive — click bars to zoom, hover for details. No external tools required.

Folded stacks can also be loaded into [speedscope](https://www.speedscope.app/) for a richer interactive view.

### trace gallery

Generate a self-contained HTML gallery of **all** scenarios with flame graphs, coverage tables, and call sequence diagrams:

```bash
trace gallery --output .trace-gallery --index .trace-index
open .trace-gallery/index.html
```

The gallery contains:
- **Index page** — grid of all scenarios with flame graph thumbnails, tags, and stats
- **Per-scenario detail pages** — embedded interactive flame graph, call sequence diagram, coverage table
- **No external dependencies** — everything is local HTML/SVG, works offline

Great for reviewing all traced scenarios at a glance and drilling into individual ones.

### trace diagram

Generate mermaid diagrams from coverage data:

```bash
# Show all files covered by a scenario
trace diagram "tests/test_auth.py::test_login"

# Show all scenarios covering a file
trace diagram --file src/auth.py

# Show scenarios covering a specific line
trace diagram --file src/auth.py:25
```

Returns JSON with a `mermaid` field containing the diagram source code.

**To extract and save as a viewable diagram:**

```bash
# Save mermaid source to a markdown file
trace diagram "tests/test_auth.py::test_login" --index .trace-index \
  | python3 -c "
import sys, json
mermaid = json.load(sys.stdin)['mermaid']
print('# Diagram\n\n\`\`\`mermaid')
print(mermaid)
print('\`\`\`')
" > diagram.md
```

**Where to view the rendered diagram:**

- **GitHub** — push the `.md` file; GitHub renders mermaid blocks natively
- **VS Code** — install the [Markdown Preview Mermaid Support](https://marketplace.visualstudio.com/items?itemName=bierner.markdown-mermaid) extension, then `Cmd+Shift+V` to preview
- **Mermaid Live Editor** — paste the mermaid source at https://mermaid.live

See `projects/pytest_tracer_python/tests/fixtures/sample_project_3/example_diagram.md` for a complete example.

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
| `diagram_scenario` | Generate mermaid diagram for a scenario |
| `diagram_file` | Generate mermaid diagram for a file |
| `flamegraph` | Generate flame graph or sequence diagram from call traces |

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

## Installing on PATH

To make `trace` available globally, symlink the release binary:

```bash
# Build release binary
cd projects/trace_analyzer
cargo build --release

# Symlink to a directory on your PATH
ln -s "$(pwd)/target/release/trace" ~/.local/bin/trace
```

## Claude Code Integration

### MCP Server

Add to your project's `.claude/settings.json`:

```json
{
  "mcpServers": {
    "trace-analyzer": {
      "command": "trace",
      "args": ["mcp", "--index", ".trace-index"]
    }
  }
}
```

If `trace` is not on your PATH, use the full path to the binary.

### Skill File

See `docs/skill.md` for a skill description that can be used with Claude Code.

### CLAUDE.md Snippet

See `docs/claude_md_snippet.md` for a ready-to-paste CLAUDE.md section that teaches agents how to write scenario tests and use the tracer.

## Current Limitations

- **Line-to-function mapping**: Not yet implemented (future: Python AST analysis)
- **Incremental index updates**: Not yet implemented (must rebuild full index)
- Coverage data must be from pytest-cov with `--cov-context=test`
