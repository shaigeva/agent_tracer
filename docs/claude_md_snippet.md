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

# Build trace index
trace build --coverage .coverage --scenarios scenarios.json --output .trace-index
\```

### Generating Diagrams

Generate mermaid diagrams showing file dependencies for scenarios:

\```bash
# Diagram for a specific scenario
trace diagram "tests/test_auth.py::test_login" --index .trace-index

# Diagram for all scenarios covering a file
trace diagram --file src/auth.py --index .trace-index
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
