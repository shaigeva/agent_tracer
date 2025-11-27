# CLAUDE.md - pytest_tracer_python

This file provides guidance to Claude Code when working with the Python project.

## Project Purpose

Minimal Python package providing:
1. Pytest marker re-exports for scenario tests
2. Scenario metadata collection script for the Rust analyzer

## Directory Structure

```
pytest_tracer_python/
├── pytest_tracer_python/
│   ├── __init__.py
│   ├── markers.py       # Re-exports pytest markers
│   └── collector.py     # Scenario metadata collector
├── tests/
│   ├── scenarios/       # Example scenario tests
│   └── test_*.py        # Unit tests
├── pyproject.toml
└── devtools/
    └── run_all_agent_validations.sh
```

## Development Commands

```bash
# Install dependencies
uv sync

# Run all validations (lint, format, type check, tests)
./devtools/run_all_agent_validations.sh

# Run tests only
uv run pytest -rP

# Run tests in watch mode
./devtools/run_tests_watch.sh

# Lint and format
uv run ruff check .
uv run ruff format .

# Type check
uv run ty check .
```

## Key Files

- `markers.py`: Re-exports `@pytest.mark.scenario`, `@pytest.mark.behavior`, `@pytest.mark.error`
- `collector.py`: Walks test directories, extracts scenario metadata, outputs JSON

## Testing Guidelines

- Tests live in `tests/`
- Scenario examples (for integration testing) live in `tests/scenarios/`
- Follow spec-driven development (see `docs/spec/how_to_write_specs.md`)

## Dependencies

- `pytest`: Required for markers
- `pytest-cov`: For coverage collection (dev dependency)
- `pydantic`: For data models

## Output Format

The collector outputs `.scenarios.json` with structure:

```json
{
  "version": "1.0",
  "collected_at": "2024-01-15T10:30:00Z",
  "scenarios": [
    {
      "id": "tests/scenarios/test_auth.py::test_login",
      "file": "tests/scenarios/test_auth.py",
      "function": "test_login",
      "description": "User logs in with valid credentials",
      "documentation": "Full docstring...",
      "behaviors": ["authentication"],
      "outcome": "success"
    }
  ]
}
```
