# CLAUDE.md - trace_analyzer

This file provides guidance to Claude Code when working with the Rust project.

## Project Purpose

Rust CLI and MCP server for:
1. Parsing pytest-cov coverage data
2. Building queryable index from coverage + scenario metadata
3. Serving queries via CLI commands
4. Exposing tools via MCP protocol for AI agents

## Directory Structure

```
trace_analyzer/
├── src/
│   ├── main.rs            # CLI entry point
│   ├── cli/               # CLI command implementations
│   │   ├── mod.rs
│   │   ├── build.rs       # trace build
│   │   ├── list.rs        # trace list
│   │   ├── search.rs      # trace search
│   │   ├── context.rs     # trace context
│   │   ├── affected.rs    # trace affected
│   │   └── run.rs         # trace run
│   ├── index/             # Index building
│   │   ├── mod.rs
│   │   ├── builder.rs     # Index construction
│   │   ├── coverage.rs    # .coverage parser
│   │   ├── metadata.rs    # .scenarios.json parser
│   │   └── ast.rs         # Python AST for line→function
│   ├── query/             # Query implementations
│   │   ├── mod.rs
│   │   ├── scenarios.rs
│   │   ├── coverage.rs
│   │   └── affected.rs
│   └── mcp/               # MCP server
│       ├── mod.rs
│       ├── server.rs
│       └── tools.rs
├── tests/
│   ├── integration_test.rs
│   └── ...
├── Cargo.toml
└── devtools/
    └── run_all_agent_validations.sh
```

## Development Commands

```bash
# Build
cargo build

# Run all validations (fmt, clippy, test, build)
./devtools/run_all_agent_validations.sh

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Run CLI
cargo run -- <command>

# Examples:
cargo run -- build --coverage .coverage --scenarios .scenarios.json
cargo run -- list
cargo run -- search "authentication"
cargo run -- context tests/scenarios/test_auth.py::test_login
cargo run -- affected src/auth/login.py:25
cargo run -- mcp
```

## Key Dependencies

- `clap`: CLI argument parsing
- `rusqlite`: SQLite database (for .coverage parsing and index storage)
- `serde` / `serde_json`: JSON serialization
- `tree-sitter` / `tree-sitter-python`: Python AST parsing (for line→function mapping)

## CLI Commands

| Command | Description |
|---------|-------------|
| `build` | Build index from .coverage and .scenarios.json |
| `list` | List scenarios with optional filters |
| `search` | Search scenario descriptions |
| `context` | Get full coverage context for a scenario |
| `affected` | Find scenarios covering a file/line |
| `run` | Run scenario with coverage collection |
| `mcp` | Start MCP server mode |

## Output Format

All commands output JSON to stdout. Errors output JSON with error structure:

```json
{
  "error": {
    "code": "INDEX_NOT_FOUND",
    "message": "Index not found. Run 'trace build' first.",
    "hint": "trace build --coverage .coverage --scenarios .scenarios.json"
  }
}
```

## Testing Guidelines

- Integration tests in `tests/`
- Test via CLI subprocess execution, not internal function calls
- Verify JSON output structure
- See `docs/spec/how_to_write_specs.md` for testing philosophy

## Index Schema

The `.trace-index/index.db` SQLite database contains:

- `scenarios`: Scenario metadata
- `scenario_behaviors`: Many-to-many behavior tags
- `coverage`: Per-scenario line coverage
- `functions`: Function definitions from AST

See `docs/tech_spec/high_level_architecture.md` for full schema.
