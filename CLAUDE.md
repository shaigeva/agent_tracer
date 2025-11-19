# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Python 3.14 project for building a markdown search agent using LangGraph. The project uses `uv` for dependency management and includes FastAPI for potential API endpoints.

## Development Commands

### Main feedback loop tool: run All Validations
Run all validations (silent on success): `./devtools/run_all_agent_validations.sh` - This script auto-fixes issues, formats code, then validates linting, formatting, type checking, and tests. Only prints output on failure.

### Testing specific files
- Run all tests: `uv run pytest -rP`
- Run a single test: `uv run pytest tests/test_file.py::test_function_name`
- Run a single test file: `uv run pytest tests/test_file.py`

### Linting and Formatting
- Auto-fix linting and formatting: `./devtools/run_lint_format_auto_fix.sh` (runs `ruff check --fix` and `ruff format`)
