# trace-analyzer skill

Use this skill to query test coverage traces and understand which tests exercise which code.
Invoke with `/trace` or when the user asks about test coverage, affected tests, or scenario tracing.

## Prerequisites

- A `.trace-index/` directory must exist in the project (built via `trace build`)
- The `trace` CLI must be on PATH or available at a known path

## Available Commands

### List scenarios
```bash
trace list --index .trace-index
trace list --behavior authentication --index .trace-index
trace list --errors --index .trace-index
```

### Search scenarios by description
```bash
trace search "login" --index .trace-index
```

### Find tests covering a file or line
```bash
trace affected src/auth.py --index .trace-index
trace affected src/auth.py:25 --index .trace-index
```

### Get full coverage context for a scenario
```bash
trace context "tests/test_auth.py::test_login" --index .trace-index
```

### Generate mermaid diagram
```bash
trace diagram "tests/test_auth.py::test_login" --index .trace-index
trace diagram --file src/auth.py --index .trace-index
```

### Run a specific scenario
```bash
trace run "tests/test_auth.py::test_login"
```

### Rebuild the index
```bash
uv run pytest --cov=src --cov-context=test
uv run pytest-tracer collect . -o scenarios.json
trace build --coverage .coverage --scenarios scenarios.json --output .trace-index
```

## Workflow

1. Before modifying code, use `trace affected <file>` to find which tests cover that code
2. Use `trace context <scenario_id>` to understand the full scope of a test
3. Use `trace diagram <scenario_id>` to visualize file dependencies
4. After changes, use `trace run <scenario_id>` to verify affected tests still pass
5. Rebuild the index after adding new tests
