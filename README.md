# Agent Tracer

Multi-language project for tracing and analyzing test executions.

## Project Structure

This repository contains multiple projects organized by language:

```
agent_tracer/
├── projects/
│   ├── pytest_tracer_python/    # Python: pytest tracing
│   └── trace_analyzer/          # Rust: CLI & MCP server for trace analysis
├── devtools/                    # Cross-project validation scripts
└── notebooks/                   # Jupyter notebooks
```

## Projects

- **[pytest_tracer_python](projects/pytest_tracer_python/)** - Python project for pytest tracing
- **[trace_analyzer](projects/trace_analyzer/)** - Rust CLI and MCP server for trace analysis

Each project has its own:
- Source code and tests
- `devtools/` directory with validation scripts
- Language-specific configuration files

## Running Validations

### All Projects

Run validations for all projects from the root:
```sh
./devtools/run_all_agent_validations.sh
```

### Python Project (pytest_tracer_python)

```sh
cd projects/pytest_tracer_python

# All validations (lint, format, type check, tests)
./devtools/run_all_agent_validations.sh

# Just tests
uv run pytest -rP

# Watch mode
./devtools/run_tests_watch.sh
```

### Rust Project (trace_analyzer)

```sh
cd projects/trace_analyzer

# All validations (fmt, clippy, test, build)
./devtools/run_all_agent_validations.sh

# Just tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## Setup

### Python Project
```sh
cd projects/pytest_tracer_python
uv sync
```

### Rust Project
```sh
cd projects/trace_analyzer
cargo build
```

## Running

### Python Project
```sh
cd projects/pytest_tracer_python
uv run python -m pytest_tracer_python.main
```

### Rust Project
```sh
cd projects/trace_analyzer

# Run as MCP server
cargo run -- mcp

# Analyze a trace file
cargo run -- analyze <file>
```
