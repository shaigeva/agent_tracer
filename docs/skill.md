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

The output is JSON with a `mermaid` field. To create a viewable diagram, extract the
mermaid source and wrap it in a fenced code block in a `.md` file:

```bash
trace diagram "tests/test_auth.py::test_login" --index .trace-index \
  | python3 -c "
import sys, json
m = json.load(sys.stdin)['mermaid']
print('# Diagram\n\n\`\`\`mermaid')
print(m)
print('\`\`\`')
" > diagram.md
```

View on GitHub (renders natively) or in VS Code with the
"Markdown Preview Mermaid Support" extension (`bierner.markdown-mermaid`).

### Generate flame graph / call-chain sequence diagram
```bash
# Interactive SVG flame graph (open in any browser - click to zoom)
trace flamegraph "tests/test_auth.py::test_login" --format svg --index .trace-index > flamegraph.svg

# Folded stacks (load in speedscope)
trace flamegraph "tests/test_auth.py::test_login" --index .trace-index

# Mermaid sequence diagram
trace flamegraph "tests/test_auth.py::test_login" --format mermaid --index .trace-index
```

Requires building the index with `--call-traces` (see rebuild section below).

### Generate gallery of all scenarios
```bash
trace gallery --output .trace-gallery --index .trace-index
# Then: open .trace-gallery/index.html
```

Creates a self-contained HTML directory with flame graphs for all scenarios.
Useful for quickly browsing all traced flows and drilling into specific ones.

### Run a specific scenario
```bash
trace run "tests/test_auth.py::test_login"
```

### Rebuild the index (with call traces)
```bash
uv run pytest --cov=src --cov-context=test
uv run pytest-tracer collect . -o scenarios.json
uv run pytest-tracer trace . -o call_traces.json
trace build --coverage .coverage --scenarios scenarios.json \
  --call-traces call_traces.json --output .trace-index
```

## Workflow

1. Before modifying code, use `trace affected <file>` to find which tests cover that code
2. Use `trace context <scenario_id>` to understand the full scope of a test
3. Use `trace diagram <scenario_id>` to visualize file dependencies
4. After changes, use `trace run <scenario_id>` to verify affected tests still pass
5. Rebuild the index after adding new tests
