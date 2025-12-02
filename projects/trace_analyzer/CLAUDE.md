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
│   ├── main.rs            # CLI entry point (thin layer)
│   ├── lib.rs             # Library root
│   ├── error.rs           # Error types
│   ├── models.rs          # Core data models
│   ├── coverage.rs        # .coverage SQLite parser
│   ├── scenarios.rs       # scenarios.json parser
│   ├── run.rs             # Scenario execution with coverage
│   ├── index/             # Index building
│   │   ├── mod.rs
│   │   ├── schema.rs      # SQLite schema and Index handle
│   │   └── builder.rs     # IndexBuilder implementation
│   ├── query/             # Query implementations
│   │   └── mod.rs         # list, search, context, affected queries
│   └── mcp/               # MCP server
│       └── mod.rs         # TraceServer with all tools
├── tests/
│   └── cli_tests.rs       # CLI integration tests
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
- `schemars`: JSON schema generation for MCP tool parameters
- `rmcp`: Rust MCP SDK for MCP server implementation
- `tokio`: Async runtime for MCP server
- `thiserror` / `anyhow`: Error handling

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

All CLI commands output JSON to stdout.

## MCP Tools

The MCP server (`trace mcp`) exposes these tools:

| Tool | Description |
|------|-------------|
| `scenario_list` | List all test scenarios |
| `scenario_list_errors` | List only error scenarios |
| `scenario_search` | Search scenarios by description |
| `scenario_context` | Get coverage context for a scenario |
| `coverage_affected_file` | Find scenarios covering a file |
| `coverage_affected_line` | Find scenarios covering a specific line |
| `scenario_run` | Run a scenario with coverage collection |

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
