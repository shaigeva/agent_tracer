# trace-analyzer skill

Use this skill to query test coverage traces and understand which tests exercise which code.
Invoke with `/trace` or when the user asks about test coverage, affected tests, or scenario tracing.

## For agents: use text outputs

You cannot read PNGs or interactive SVGs. For extraction and reasoning, use:

- `trace flamegraph <id> --format summary` — compact JSON list of unique frames touched (use first)
- `trace flamegraph <id> --format folded-compact` — call tree with collapsed prefixes
- `trace flamegraph <id> --format mermaid` — sequence diagram as text
- `trace affected <file> --with-snippets --functions-only` — scenarios + source + function names
- `trace context <id>` — JSON coverage data
- `trace list / search` — JSON scenario lists

The `png`, `html`, and `svg` formats are for humans. Only produce them when the user
explicitly asks to view/share a flame graph.

### Default: auto-anchor at the scenario's test function

Output is trimmed to start at the scenario's own test function; everything above
it (pytest fixture graph, framework setup) is dropped. Override with:

- `--include-fixtures` — show the full tree including fixture setup/teardown
- `--from <pattern>` — anchor at a different frame (e.g. `--from OrderService.create_order`)

### Scoping and pruning

- `--include 'auth_api,password'` or `--include auth_api --include password` — keep only stacks containing a matching frame
- `--exclude 'conftest,bootstrap'` or `--exclude conftest --exclude bootstrap` — drop stacks containing a matching frame
- `--max-depth 3` — truncate deeply-nested stacks

If filtering produces empty output, a stderr hint suggests what to try.

## Prerequisites

- A `.trace-index/` directory must exist in the project (built via `trace build`)
- The `trace` CLI must be on PATH or available at a known path
- If the index doesn't have call traces (flamegraph commands fail), rebuild with `--call-traces`

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

Output formats: `png` | `html` | `svg` | `folded` | `mermaid`

```bash
# PNG - static image, loads anywhere (VS Code, any browser, email, docs)
trace flamegraph "tests/test_auth.py::test_login" --format png --index .trace-index > flame.png

# HTML - interactive flame graph wrapped in a page (recommended for viewing)
trace flamegraph "tests/test_auth.py::test_login" --format html --index .trace-index > flame.html

# Folded stacks (for speedscope.app or flamegraph.pl)
trace flamegraph "tests/test_auth.py::test_login" --index .trace-index

# Mermaid sequence diagram
trace flamegraph "tests/test_auth.py::test_login" --format mermaid --index .trace-index
```

Recommend PNG or HTML over raw SVG — browsers block scripts in file:// SVGs
and VS Code shows SVGs as XML text by default. Requires building the index
with `--call-traces` (see rebuild section below).

### Generate gallery of all scenarios
```bash
trace gallery --output .trace-gallery --index .trace-index
cd .trace-gallery && python3 -m http.server
# Then open http://localhost:8000/gallery.html
```

Creates a lazy-loading HTML viewer:
- `gallery.html` + `flamegraph.js` (reusable standalone renderer)
- `data/index.json` (scenario metadata, loads eagerly)
- `data/traces/<id>.json` (per-scenario events, lazy-loaded)

Features: grid view with search, filter by success/error/traced, click-to-drill-in
flame graph (zoom, hover, text search), call sequence diagram, raw events view.
Scales to thousands of scenarios since traces are only loaded when viewed.

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

## Typical agent workflow

1. **Before editing code**: `trace affected <file>` to find which scenarios cover it
2. **To understand a scenario**: `trace context <scenario_id>` for coverage, `trace flamegraph <id> --format folded` for call chain
3. **To map dependencies**: `trace flamegraph <id> --format mermaid` for a textual sequence diagram
4. **After changes**: `trace run <scenario_id>` to re-execute and verify
5. **After adding tests**: rebuild the index (see rebuild section above)

## Understanding flame graph folded output

Each line is one call stack:

```
conftest.order_repo 1
conftest.order_repo;order_repository.OrderRepository.__init__ 1
tests_test_order_flow.test_add_item;routes.OrderRoutes.post_order 1
tests_test_order_flow.test_add_item;routes.OrderRoutes.post_order;middleware.AuthMiddleware.create_order 1
```

Semicolons separate frames (outermost → innermost). The trailing number is sample count. Frame format is `module.qualname` where `module` is the file stem. This makes the call chain directly readable — each line tells you "this call stack happened during this test."
