# High-Level Architecture

**Related guides**:
- [main_spec.md](../spec/main_spec.md) - Project specification
- [how_to_write_specs.md](../spec/how_to_write_specs.md) - Writing specifications
- [how_to_implement_tasks.md](../how_to_implement_tasks.md) - Task implementation process

## Overview

pytest-tracer is a two-part system for capturing test execution coverage and exposing it for AI agent consumption:

1. **pytest-tracer** (Python): Minimal package providing scenario markers and metadata collection
2. **trace-analyzer** (Rust): CLI tool and MCP server for indexing and querying coverage data

The design philosophy is to leverage existing, battle-tested tools (pytest-cov, coverage.py) rather than building custom tracing infrastructure for the MVP.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          Python Test Suite                               │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │  tests/scenarios/                                                 │    │
│  │  ├── test_auth.py      @pytest.mark.scenario                     │    │
│  │  ├── test_orders.py    @pytest.mark.behavior("...")              │    │
│  │  └── ...               @pytest.mark.error                        │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ pytest --cov --cov-context=test
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         Coverage Collection                              │
│  ┌──────────────────────┐    ┌──────────────────────┐                   │
│  │ pytest-cov           │    │ pytest-tracer        │                   │
│  │ (line coverage)      │    │ (metadata collector) │                   │
│  └──────────┬───────────┘    └──────────┬───────────┘                   │
│             │                           │                                │
│             ▼                           ▼                                │
│  ┌──────────────────────┐    ┌──────────────────────┐                   │
│  │ .coverage (SQLite)   │    │ .scenarios.json      │                   │
│  │ per-test line data   │    │ scenario metadata    │                   │
│  └──────────────────────┘    └──────────────────────┘                   │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ trace build
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         trace-analyzer (Rust)                            │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     Index Builder                                  │   │
│  │  • Parse .coverage SQLite                                         │   │
│  │  • Import .scenarios.json                                         │   │
│  │  • Map lines → functions (AST)                                    │   │
│  │  • Build .trace-index/ (SQLite)                                   │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                    │                                     │
│                                    ▼                                     │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    .trace-index/ (SQLite)                         │   │
│  │  scenarios │ coverage_files │ coverage_lines │ functions          │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                    │                                     │
│                 ┌──────────────────┴──────────────────┐                 │
│                 ▼                                      ▼                 │
│  ┌──────────────────────────┐          ┌──────────────────────────┐    │
│  │        CLI               │          │      MCP Server          │    │
│  │  trace list              │          │  scenario_search         │    │
│  │  trace search            │          │  scenario_context        │    │
│  │  trace context           │          │  coverage_affected       │    │
│  │  trace affected          │          │  scenario_run            │    │
│  │  trace run               │          │                          │    │
│  └──────────────────────────┘          └──────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           AI Agents / Users                              │
│  • Query scenario coverage via CLI or MCP                               │
│  • Understand code paths for specific behaviors                         │
│  • Find affected tests before making changes                            │
└─────────────────────────────────────────────────────────────────────────┘
```

## System Components

### Directory Structure

```
agent_tracer/
├── docs/
│   ├── spec/                      # Functional specifications
│   │   ├── main_spec.md
│   │   ├── how_to_write_specs.md
│   │   └── detailed/              # Detailed feature specs
│   ├── tech_spec/                 # Technical specifications
│   │   └── high_level_architecture.md
│   └── tasks/                     # Task tracking
│       ├── current_task_list.md
│       └── archive/
├── projects/
│   ├── pytest_tracer_python/      # Python package (minimal)
│   │   ├── pytest_tracer_python/
│   │   │   ├── __init__.py
│   │   │   ├── markers.py         # Re-exports pytest markers
│   │   │   └── collector.py       # Scenario metadata collector
│   │   ├── tests/
│   │   │   └── scenarios/         # Example scenario tests
│   │   ├── pyproject.toml
│   │   └── devtools/
│   │       └── run_all_agent_validations.sh
│   └── trace_analyzer/            # Rust CLI + MCP server
│       ├── src/
│       │   ├── main.rs            # CLI entry point
│       │   ├── cli/
│       │   │   ├── mod.rs
│       │   │   ├── build.rs       # trace build command
│       │   │   ├── list.rs        # trace list command
│       │   │   ├── search.rs      # trace search command
│       │   │   ├── context.rs     # trace context command
│       │   │   ├── affected.rs    # trace affected command
│       │   │   └── run.rs         # trace run command
│       │   ├── index/
│       │   │   ├── mod.rs
│       │   │   ├── builder.rs     # Index construction
│       │   │   ├── coverage.rs    # .coverage file parser
│       │   │   ├── metadata.rs    # .scenarios.json parser
│       │   │   └── ast.rs         # Python AST for line→function
│       │   ├── query/
│       │   │   ├── mod.rs
│       │   │   ├── scenarios.rs   # Scenario queries
│       │   │   ├── coverage.rs    # Coverage queries
│       │   │   └── affected.rs    # Affected test queries
│       │   └── mcp/
│       │       ├── mod.rs
│       │       ├── server.rs      # MCP server implementation
│       │       └── tools.rs       # MCP tool definitions
│       ├── tests/
│       ├── Cargo.toml
│       └── devtools/
│           └── run_all_agent_validations.sh
└── devtools/
    └── run_all_agent_validations.sh  # Cross-project validation
```

### Python Package: pytest-tracer

**Purpose**: Minimal package providing:
1. Marker re-exports for convenience
2. Scenario metadata collection script

**Why minimal**: We leverage pytest-cov for coverage collection rather than building our own. The Python package only needs to:
- Provide marker convenience imports
- Walk the test tree and extract scenario metadata

```python
# pytest_tracer_python/markers.py
import pytest

# Re-export markers for convenience
scenario = pytest.mark.scenario
behavior = pytest.mark.behavior
error = pytest.mark.error
```

```python
# pytest_tracer_python/collector.py
"""Collect scenario metadata from test files."""

def collect_scenarios(test_dir: str) -> list[ScenarioMetadata]:
    """Walk test directory, find @scenario tests, extract metadata."""
    ...

def export_scenarios(scenarios: list[ScenarioMetadata], output: str) -> None:
    """Export scenarios to JSON file."""
    ...
```

### Rust Application: trace-analyzer

**Purpose**: High-performance CLI and MCP server for:
1. Parsing coverage data from pytest-cov
2. Building queryable index
3. Serving queries via CLI and MCP

**Why Rust**:
- Fast startup time for CLI usage
- Efficient SQLite handling
- Easy MCP server implementation (mcp-rust ecosystem)
- Single binary distribution

## Data Flow

### Phase 1: Test Execution with Coverage

```
1. User runs: pytest --cov --cov-context=test tests/scenarios/
   │
2. pytest-cov instruments Python code
   │
3. For each test, coverage.py records:
   - Context name: "test_file.py::test_function"
   - Lines executed per file
   │
4. Results stored in .coverage (SQLite database)
```

### Phase 2: Metadata Collection

```
1. User runs: python -m pytest_tracer_python.collector
   │
2. Collector walks tests/scenarios/ directory
   │
3. For each file with @scenario markers:
   - Parse Python AST
   - Extract test function metadata
   - Parse docstring structure
   │
4. Output: .scenarios.json
   {
     "scenarios": [
       {
         "id": "tests/scenarios/test_auth.py::test_login",
         "description": "User logs in with valid credentials",
         "documentation": "Full docstring...",
         "behaviors": ["authentication"],
         "outcome": "success"
       }
     ]
   }
```

### Phase 3: Index Building

```
1. User runs: trace build
   │
2. trace-analyzer reads:
   - .coverage (SQLite) - per-test line coverage
   - .scenarios.json - scenario metadata
   │
3. For each covered file:
   - Parse Python AST (using tree-sitter or minimal parser)
   - Map line numbers to function definitions
   │
4. Build .trace-index/index.db (SQLite):
   - scenarios table: id, description, behaviors, outcome
   - coverage table: scenario_id, file_path, lines
   - functions table: file_path, name, start_line, end_line, docstring
```

### Phase 4: Queries

```
CLI:
  trace search "authentication"
  → Query scenarios table by description/behaviors
  → Return JSON list of matching scenarios

  trace context tests/scenarios/test_auth.py::test_login
  → Join scenarios + coverage + functions
  → Return structured JSON with files, lines, functions

  trace affected src/auth/login.py:25
  → Query coverage for line containment
  → Return list of scenarios covering that line

MCP:
  Same queries exposed as MCP tools
  Agent calls: scenario_search(query="authentication")
  Server returns: { scenarios: [...] }
```

## Storage Design

### .coverage (Input - SQLite from coverage.py)

```sql
-- Simplified schema (actual coverage.py schema)
CREATE TABLE file (
    id INTEGER PRIMARY KEY,
    path TEXT
);

CREATE TABLE context (
    id INTEGER PRIMARY KEY,
    context TEXT  -- e.g., "test_auth.py::test_login"
);

CREATE TABLE line_bits (
    file_id INTEGER,
    context_id INTEGER,
    numbits BLOB  -- Bitmap of covered lines
);
```

### .scenarios.json (Input - from collector)

```json
{
  "version": "1.0",
  "collected_at": "2024-01-15T10:30:00Z",
  "scenarios": [
    {
      "id": "tests/scenarios/test_auth.py::test_successful_login",
      "file": "tests/scenarios/test_auth.py",
      "function": "test_successful_login",
      "description": "User logs in with valid credentials",
      "documentation": "User logs in with valid credentials\n\nGIVEN...",
      "behaviors": ["authentication", "session-management"],
      "outcome": "success"
    }
  ]
}
```

### .trace-index/index.db (Output - SQLite)

```sql
-- Scenarios with their metadata
CREATE TABLE scenarios (
    id TEXT PRIMARY KEY,           -- tests/scenarios/test_auth.py::test_login
    file TEXT NOT NULL,
    function TEXT NOT NULL,
    description TEXT NOT NULL,
    documentation TEXT,
    outcome TEXT NOT NULL          -- "success" or "error"
);

-- Behavior tags (many-to-many)
CREATE TABLE scenario_behaviors (
    scenario_id TEXT NOT NULL REFERENCES scenarios(id),
    behavior TEXT NOT NULL,
    PRIMARY KEY (scenario_id, behavior)
);

-- Coverage per scenario
CREATE TABLE coverage (
    scenario_id TEXT NOT NULL REFERENCES scenarios(id),
    file_path TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    PRIMARY KEY (scenario_id, file_path, line_number)
);

-- Function definitions (from AST)
CREATE TABLE functions (
    file_path TEXT NOT NULL,
    name TEXT NOT NULL,
    start_line INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    docstring TEXT,
    PRIMARY KEY (file_path, name, start_line)
);

-- Indexes for query performance
CREATE INDEX idx_coverage_file_line ON coverage(file_path, line_number);
CREATE INDEX idx_behaviors_behavior ON scenario_behaviors(behavior);
CREATE INDEX idx_scenarios_description ON scenarios(description);
```

## Key Design Decisions

### 1. Minimal Python Package

**Decision**: The Python package only provides markers and metadata collection, not coverage instrumentation.

**Rationale**:
- pytest-cov is mature and handles all coverage edge cases
- `--cov-context=test` gives us per-test granularity for free
- Reduces maintenance burden and potential bugs
- Users can continue using familiar pytest-cov workflow

### 2. Rust for Analyzer

**Decision**: Build the analyzer and MCP server in Rust.

**Rationale**:
- Fast CLI startup (important for frequent agent queries)
- Excellent SQLite support (rusqlite)
- Growing MCP ecosystem in Rust
- Single binary distribution (no Python runtime needed for queries)
- Memory safety without garbage collection overhead

### 3. SQLite for Everything

**Decision**: Use SQLite for both input (.coverage) and output (.trace-index).

**Rationale**:
- Single file, no server process
- Fast queries with proper indexing
- Standard tool support (can inspect with sqlite3 CLI)
- coverage.py already uses SQLite, so we're reading native format

### 4. JSON Output Format

**Decision**: All CLI commands output JSON by default.

**Rationale**:
- Machine-readable for AI agents
- Easy to parse programmatically
- Can be piped to jq for human inspection
- Consistent format across all commands

### 5. Scenario Convention Over Configuration

**Decision**: Use pytest markers and docstring conventions rather than config files.

**Rationale**:
- Tests are self-documenting
- No separate config to maintain
- Familiar to pytest users
- Docstrings provide natural language descriptions for agents

### 6. Line-Level Coverage (MVP)

**Decision**: Start with line coverage, not full tracing.

**Rationale**:
- pytest-cov provides this out of the box
- Sufficient for "which tests cover this code?" queries
- Much simpler than sys.monitoring tracing
- Can add full tracing in V2 after proving value

## Dependencies

### Python Project

```toml
[project]
dependencies = [
    "pytest>=7.0",
]

[dependency-groups]
dev = [
    "pytest-cov>=4.0",
    "ruff",
]
```

### Rust Project

```toml
[dependencies]
# CLI
clap = { version = "4.5", features = ["derive"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
rusqlite = { version = "0.31", features = ["bundled"] }

# Python AST (for line→function mapping)
tree-sitter = "0.22"
tree-sitter-python = "0.21"

# MCP Server (when available, or implement manually)
# mcp-server = "0.1"  # TBD - may need to implement MCP protocol directly

[dev-dependencies]
tempfile = "3.10"
```

## CLI Interface

### Commands

```
trace-analyzer 0.1.0
CLI for querying test scenario coverage

USAGE:
    trace <COMMAND>

COMMANDS:
    build      Build or rebuild the index from coverage data
    list       List scenarios with optional filtering
    search     Search scenario descriptions and docstrings
    context    Get full coverage context for a scenario
    affected   Find scenarios covering a file or line
    run        Run a scenario with coverage collection
    mcp        Start MCP server mode
    help       Print this message or help for a command

OPTIONS:
    -h, --help       Print help
    -V, --version    Print version
```

### Command Details

```bash
# Build index from .coverage and .scenarios.json
trace build [--coverage .coverage] [--scenarios .scenarios.json] [--output .trace-index]

# List all scenarios
trace list
trace list --behavior authentication
trace list --outcome error
trace list --file tests/scenarios/test_auth.py

# Search scenarios by text
trace search "login"
trace search "authentication" --limit 10

# Get full context for a scenario
trace context tests/scenarios/test_auth.py::test_successful_login

# Find scenarios covering code
trace affected src/auth/login.py
trace affected src/auth/login.py:25
trace affected src/auth/login.py:20-30

# Run scenario with coverage
trace run tests/scenarios/test_auth.py::test_successful_login
trace run --all --behavior authentication

# Start MCP server
trace mcp
```

## MCP Interface

### Tool Definitions

```json
{
  "tools": [
    {
      "name": "scenario_search",
      "description": "Search for test scenarios by description or behavior",
      "inputSchema": {
        "type": "object",
        "properties": {
          "query": {
            "type": "string",
            "description": "Search query for scenario descriptions"
          },
          "behavior": {
            "type": "string",
            "description": "Filter by behavior tag"
          },
          "outcome": {
            "type": "string",
            "enum": ["success", "error"],
            "description": "Filter by expected outcome"
          },
          "limit": {
            "type": "integer",
            "default": 10,
            "description": "Maximum results to return"
          }
        }
      }
    },
    {
      "name": "scenario_context",
      "description": "Get full coverage context for a scenario",
      "inputSchema": {
        "type": "object",
        "properties": {
          "scenario_id": {
            "type": "string",
            "description": "Full scenario ID (e.g., tests/scenarios/test_auth.py::test_login)"
          }
        },
        "required": ["scenario_id"]
      }
    },
    {
      "name": "coverage_affected",
      "description": "Find scenarios that cover specific code",
      "inputSchema": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "description": "Source file path"
          },
          "line": {
            "type": "integer",
            "description": "Specific line number (optional)"
          },
          "line_range": {
            "type": "object",
            "properties": {
              "start": { "type": "integer" },
              "end": { "type": "integer" }
            },
            "description": "Line range (optional)"
          }
        },
        "required": ["file"]
      }
    },
    {
      "name": "scenario_run",
      "description": "Run a scenario with coverage collection",
      "inputSchema": {
        "type": "object",
        "properties": {
          "scenario_id": {
            "type": "string",
            "description": "Full scenario ID to run"
          },
          "rebuild_index": {
            "type": "boolean",
            "default": true,
            "description": "Rebuild index after run"
          }
        },
        "required": ["scenario_id"]
      }
    }
  ]
}
```

## Development Phases

### Phase 1: Core Index & Basic Queries
**Focus**: Get end-to-end working with minimal features

1. Python metadata collector (collect scenarios → JSON)
2. Rust coverage parser (.coverage → internal model)
3. Rust index builder (combine coverage + metadata → SQLite)
4. CLI: `trace build`, `trace list`, `trace context`

### Phase 2: Search & Affected Queries
**Focus**: Make queries useful for agents

1. CLI: `trace search` with text matching
2. CLI: `trace affected` with file/line queries
3. Add behavior filtering to `trace list`

### Phase 3: Run Integration
**Focus**: Complete the workflow loop

1. CLI: `trace run` to execute scenarios
2. Automatic index rebuild after run
3. Integration tests for full workflow

### Phase 4: MCP Server
**Focus**: Enable AI agent integration

1. Implement MCP protocol (or use library)
2. Expose tools: scenario_search, scenario_context, coverage_affected, scenario_run
3. Integration tests with MCP client

## Testing Strategy

### Python Project

```
tests/
├── test_markers.py          # Verify marker exports
├── test_collector.py        # Unit tests for metadata collection
└── scenarios/               # Example scenarios (also used for integration)
    └── test_example.py
```

### Rust Project

```
tests/
├── cli/
│   ├── test_build.rs        # Index building
│   ├── test_list.rs         # List command
│   ├── test_search.rs       # Search command
│   ├── test_context.rs      # Context command
│   └── test_affected.rs     # Affected command
├── index/
│   ├── test_coverage_parser.rs
│   ├── test_metadata_parser.rs
│   └── test_ast_mapper.rs
└── integration/
    └── test_full_workflow.rs
```

### Integration Testing

Full workflow test:
1. Run Python test suite with coverage
2. Run metadata collector
3. Build index
4. Query via CLI
5. Verify results match expected coverage

## Error Handling

### CLI Errors

```json
{
  "error": {
    "code": "INDEX_NOT_FOUND",
    "message": "Index not found. Run 'trace build' first.",
    "hint": "trace build --coverage .coverage --scenarios .scenarios.json"
  }
}
```

### Error Codes

| Code | Description |
|------|-------------|
| INDEX_NOT_FOUND | No .trace-index found |
| COVERAGE_NOT_FOUND | No .coverage file found |
| SCENARIOS_NOT_FOUND | No .scenarios.json found |
| SCENARIO_NOT_FOUND | Requested scenario doesn't exist |
| INVALID_FILE_PATH | File path doesn't exist |
| PARSE_ERROR | Failed to parse coverage/metadata |

## Future Extensions

### V2: Full Execution Tracing

- Use sys.monitoring (Python 3.12+) for call tracing
- Capture function entry/exit with parameters
- Build call graphs per scenario
- Enable "what called this function?" queries

### V3: Time-Travel Debugging

- Record full execution traces
- Build UI for stepping through execution
- Compare traces between test runs
- Identify behavioral changes

### Multi-Language Support

- Add JavaScript/TypeScript coverage parsing
- Support Jest, Vitest coverage formats
- Language-agnostic index schema
