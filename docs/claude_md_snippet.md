# pytest-tracer Integration Guide

Add the relevant sections below to your project's CLAUDE.md to help AI agents use trace-analyzer.

---

## CLAUDE.md Snippet

```markdown
## pytest-tracer Integration

This project uses pytest-tracer to capture test execution traces. The `trace-analyzer` MCP server provides tools to query which tests cover which code.

**Use the `trace-analyzer` MCP to find relevant tests before making code changes.**

### Writing Scenario Tests

Mark tests with `@pytest.mark.scenario` so they're indexed:

\```python
import pytest

@pytest.mark.scenario  # Required - marks this as a traceable scenario
@pytest.mark.behavior("feature-name")  # Optional - categorizes the test
@pytest.mark.error  # Optional - indicates this tests an error case
def test_something():
    """First line is the searchable description"""
    ...
\```

### Docstring Format

The first line becomes the scenario description (shown in search results). Use GIVEN/WHEN/THEN for complex scenarios:

\```python
@pytest.mark.scenario
@pytest.mark.behavior("authentication")
def test_login_success():
    """
    User logs in with valid credentials

    GIVEN a registered user
    WHEN they submit correct email and password
    THEN they receive an auth token
    """
    ...

@pytest.mark.scenario
@pytest.mark.behavior("authentication")
@pytest.mark.error
def test_login_wrong_password():
    """Login fails with incorrect password"""
    ...
\```

### Behavior Tags

Use `@pytest.mark.behavior("tag")` to categorize tests. Multiple behaviors allowed:

\```python
@pytest.mark.scenario
@pytest.mark.behavior("user-management")
@pytest.mark.behavior("validation")
def test_create_user_validates_email():
    """User creation rejects invalid email format"""
    ...
\```

### Rebuilding the Trace Index

After adding new tests, rebuild the index:

\```bash
# Run tests with per-test coverage
uv run pytest --cov=src --cov-context=test

# Collect scenario metadata
uv run pytest-tracer collect . -o scenarios.json

# Collect call traces (required for flame graphs)
uv run pytest-tracer trace . -o call_traces.json

# Build trace index
trace build --coverage .coverage --scenarios scenarios.json \
  --call-traces call_traces.json --output .trace-index
\```

### Text outputs for understanding a test

You cannot read PNG/SVG. Use these text formats (ranked by token cost):

\```bash
# Compact JSON list of unique frames - recommended first
trace flamegraph "tests/test_auth.py::test_login" --format summary --index .trace-index

# Call tree with prefix compaction (...(N) collapses shared prefixes)
trace flamegraph "tests/test_auth.py::test_login" --format folded-compact --index .trace-index

# Full folded stacks (one line per nested call)
trace flamegraph "tests/test_auth.py::test_login" --format folded --index .trace-index

# Sequence diagram showing cross-file calls (mermaid text)
trace flamegraph "tests/test_auth.py::test_login" --format mermaid --index .trace-index

# Scenarios covering a file, with source snippets AND function names
trace affected src/auth.py --with-snippets --functions-only --index .trace-index

# Files + line numbers covered by a test (JSON)
trace context "tests/test_auth.py::test_login" --index .trace-index
\```

By default, pytest fixture (conftest.py) frames are dropped from flamegraph output
(they're mostly noise). Use `--include-fixtures` to see them. Further scoping:
`--include 'auth_api,password'`, `--exclude 'bootstrap'`, `--max-depth 5`.

### Generating Diagrams (for humans to view)

\```bash
# Mermaid coverage diagram (files grouped by directory)
trace diagram "tests/test_auth.py::test_login" --index .trace-index

# PNG flame graph (renders anywhere - email, docs, any viewer)
trace flamegraph "tests/test_auth.py::test_login" --format png --index .trace-index > flame.png

# Interactive HTML flame graph
trace flamegraph "tests/test_auth.py::test_login" --format html --index .trace-index > flame.html
\```
```

---

## Claude Code MCP Server Configuration

Add this to your project's `.claude/settings.json` (or `~/.claude/settings.json` for global):

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

If `trace` is not on your PATH, use the full path to the binary:

```json
{
  "mcpServers": {
    "trace-analyzer": {
      "command": "/path/to/pytest-tracer/projects/trace_analyzer/target/release/trace",
      "args": ["mcp", "--index", "/absolute/path/to/your/project/.trace-index"]
    }
  }
}
```

### Claude Desktop Configuration

Add to `~/.config/claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "trace-analyzer": {
      "command": "trace",
      "args": ["mcp", "--index", "/absolute/path/to/your/project/.trace-index"]
    }
  }
}
```

---

## Installation

### 1. Build the Rust CLI (one-time)

```bash
cd /path/to/pytest-tracer/projects/trace_analyzer
cargo build --release

# Optionally symlink to a directory on your PATH:
ln -s "$(pwd)/target/release/trace" ~/.local/bin/trace
```

### 2. Add pytest-tracer to your Python project

```bash
cd your_project
uv add --dev /path/to/pytest-tracer/projects/pytest_tracer_python
```

This installs the scenario markers and the `pytest-tracer` CLI for metadata collection.

### 3. Add to .gitignore

```
.trace-index/
scenarios.json
```
