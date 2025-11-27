# Main Specification - pytest-tracer

This document contains high-level feature descriptions and rationale. Each feature links to a detailed specification.

**Related guides**:
- [how_to_write_specs.md](how_to_write_specs.md) - Writing specifications
- [how_to_implement_tasks.md](../how_to_implement_tasks.md) - Task implementation process
- [high_level_architecture.md](../tech_spec/high_level_architecture.md) - Technical architecture

---

## Project Overview

**pytest-tracer** is a tool for helping AI coding agents understand codebases by capturing test execution traces and making them available for context gathering.

### Problem Statement

AI coding agents struggle to understand codebases because they lack execution context. When debugging or implementing features, agents need to know:
- Which code paths execute for specific behaviors
- What tests exercise particular code
- How to focus on relevant files/functions rather than searching randomly

### Solution

A two-part system:

1. **pytest-tracer** (Python): A pytest plugin that captures test execution coverage with per-test granularity
2. **trace-analyzer** (Rust): A CLI tool that indexes and queries the coverage data, exposing it via CLI and MCP for agent consumption

### Target Users

- **Primary**: AI coding agents (Claude, etc.) that need execution context for code understanding
- **Secondary**: Human developers who want to understand test coverage and code relationships

### Core Concept: Scenario Tests

Tests marked with `@pytest.mark.scenario` serve as documented entry points into the codebase. Each scenario test:
- Has a descriptive docstring explaining the behavior it tests
- Is tagged with behaviors via `@pytest.mark.behavior("behavior-name")`
- Can be marked as error case via `@pytest.mark.error`
- When run with coverage, reveals which code executes for that behavior

---

## MVP Capabilities (Phase 1: Coverage-Based)

Phase 1 uses pytest-cov's `--cov-context=test` feature to get per-test line coverage. This enables:

**Queries agents can make:**
- "Which tests cover this file/function/line?"
- "What code does test X execute?"
- "What tests are affected by changes to file Y?"
- "Find scenarios related to behavior Z"
- "Compare coverage between success and error test cases"

**What coverage provides:**
- Files involved in a scenario
- Functions called (via AST mapping of lines to functions)
- Exact lines executed

**What coverage does NOT provide (future tracing work):**
- Execution order
- Call graphs / stack traces
- Parameter values
- Loop iteration counts

---

## Feature: Scenario Test Markers
**Status**: 0/4 requirements implemented (0%)
**Detail Spec**: [detailed/scenario_markers_detailed_spec.md](detailed/scenario_markers_detailed_spec.md)
**Purpose**: Define test conventions for scenario tests that AI agents can query
**Version**: V1

### Rationale

Scenario tests serve as documented entry points into the codebase. By using consistent markers, we enable agents to discover and understand tests programmatically.

### High-Level Requirements
- REQ-MARKER-001: `@pytest.mark.scenario` marker identifies scenario tests
- REQ-MARKER-002: `@pytest.mark.behavior("name")` tags tests with behaviors (multiple allowed)
- REQ-MARKER-003: `@pytest.mark.error` marks tests as error/failure scenarios
- REQ-MARKER-004: Scenario docstrings follow structured format (description + optional GIVEN/WHEN/THEN)

---

## Feature: Scenario Metadata Collection
**Status**: 0/5 requirements implemented (0%)
**Detail Spec**: [detailed/metadata_collection_detailed_spec.md](detailed/metadata_collection_detailed_spec.md)
**Purpose**: Collect and export scenario metadata for indexing
**Version**: V1

### Rationale

Before analyzing coverage, we need to collect all scenario test metadata (names, docstrings, behaviors, error flags) into a format the Rust analyzer can consume.

### High-Level Requirements
- REQ-META-001: Collect scenario tests from test directory
- REQ-META-002: Extract docstring description and structured sections
- REQ-META-003: Extract behavior markers
- REQ-META-004: Identify error scenarios
- REQ-META-005: Export metadata as JSON for analyzer consumption

---

## Feature: Index Building
**Status**: 0/5 requirements implemented (0%)
**Detail Spec**: [detailed/index_building_detailed_spec.md](detailed/index_building_detailed_spec.md)
**Purpose**: Build queryable index from coverage data and scenario metadata
**Version**: V1

### Rationale

The Rust analyzer needs to parse pytest-cov output (SQLite .coverage file) and scenario metadata to build an efficient index for queries.

### High-Level Requirements
- REQ-INDEX-001: Parse .coverage SQLite file from pytest-cov
- REQ-INDEX-002: Import scenario metadata from JSON
- REQ-INDEX-003: Map coverage lines to functions via AST analysis
- REQ-INDEX-004: Build SQLite index with scenarios, coverage, and mappings
- REQ-INDEX-005: Support incremental index updates

---

## Feature: CLI Queries
**Status**: 0/8 requirements implemented (0%)
**Detail Spec**: [detailed/cli_queries_detailed_spec.md](detailed/cli_queries_detailed_spec.md)
**Purpose**: Command-line interface for querying scenario and coverage data
**Version**: V1

### Rationale

AI agents interact with tools via CLI. The trace-analyzer CLI provides commands for discovering scenarios, getting coverage context, and finding affected tests.

### High-Level Requirements
- REQ-CLI-001: `trace build` - Build/rebuild index from coverage and metadata
- REQ-CLI-002: `trace list` - List all scenarios with optional filters
- REQ-CLI-003: `trace search "query"` - Search scenario descriptions
- REQ-CLI-004: `trace context <scenario>` - Get full coverage context for a scenario
- REQ-CLI-005: `trace affected <file>` - Find tests covering a file
- REQ-CLI-006: `trace affected <file:line>` - Find tests covering specific line
- REQ-CLI-007: `trace run <scenario>` - Run scenario with coverage
- REQ-CLI-008: JSON output format for all commands

---

## Feature: MCP Server
**Status**: 0/5 requirements implemented (0%)
**Detail Spec**: [detailed/mcp_server_detailed_spec.md](detailed/mcp_server_detailed_spec.md)
**Purpose**: Model Context Protocol server for AI agent integration
**Version**: V1

### Rationale

MCP provides standardized tool interfaces for AI agents. Exposing scenario queries via MCP allows agents like Claude to directly query coverage information.

### High-Level Requirements
- REQ-MCP-001: MCP server exposes scenario_search tool
- REQ-MCP-002: MCP server exposes scenario_context tool
- REQ-MCP-003: MCP server exposes coverage_affected tool
- REQ-MCP-004: MCP server exposes scenario_run tool
- REQ-MCP-005: Server follows MCP specification for tool definitions

---

## Future Features (Not Yet Planned)

### Full Execution Tracing (V2)
- Capture execution order via sys.monitoring (Python 3.12+)
- Call graph construction
- Parameter value capture
- Loop iteration counts

### Time-Travel Debugging UI (V3)
- Visual exploration of execution traces
- Step through code execution
- Compare traces between test runs

---

## Status Legend

**Feature Status:**
- (0% requirements implemented) - Not Started
- (1-99% requirements implemented) - Partial
- (100% requirements implemented) - Fully Implemented

**Requirement Status:**
- Not marked - Not Implemented
- Implemented - Implemented and tested
- Needs Fix - Implemented incorrectly

---

## Example Scenario Test

```python
import pytest

@pytest.mark.scenario
@pytest.mark.behavior("authentication")
@pytest.mark.behavior("session-management")
def test_successful_login():
    """
    User logs in with valid credentials

    GIVEN a registered user with email and password
    WHEN they submit valid credentials
    THEN they receive an auth token and session is created
    """
    user = create_user(email="test@example.com", password="secret")
    response = login(email=user.email, password="secret")

    assert response.status == 200
    assert response.token is not None


@pytest.mark.scenario
@pytest.mark.behavior("authentication")
@pytest.mark.error
def test_login_invalid_credentials():
    """
    Login fails with invalid credentials

    GIVEN a registered user
    WHEN they submit wrong password
    THEN they receive 401 error
    """
    user = create_user(email="test@example.com", password="secret")
    response = login(email=user.email, password="wrong")

    assert response.status == 401
```

---

## Example Context Output

```json
{
  "scenario": {
    "id": "tests/scenarios/test_auth.py::test_successful_login",
    "description": "User logs in with valid credentials",
    "documentation": "User logs in with valid credentials\n\nGIVEN a registered user...",
    "behaviors": ["authentication", "session-management"],
    "outcome": "success"
  },
  "coverage": {
    "files": {
      "src/auth/login.py": {
        "lines": [12, 13, 14, 23, 24, 25],
        "functions": [
          {"name": "authenticate", "lines": [12, 13, 14], "docstring": "Validate credentials"},
          {"name": "create_session", "lines": [23, 24, 25], "docstring": "Create user session"}
        ]
      },
      "src/models/user.py": {
        "lines": [45, 46, 47],
        "functions": [
          {"name": "check_password", "lines": [45, 46, 47], "docstring": "Verify password hash"}
        ]
      }
    }
  }
}
```

---

## Agent Instructions

This section can be provided to AI agents to explain how to use pytest-tracer effectively.

### Quick Start for Agents

1. **Find relevant scenarios**: Use `trace search "behavior"` to find tests related to what you're working on
2. **Get context**: Use `trace context <scenario_id>` to see what code a scenario exercises
3. **Find affected tests**: Use `trace affected <file:line>` when modifying code to know what tests to run
4. **Understand patterns**: Read scenario docstrings for GIVEN/WHEN/THEN structure

### Query Workflow

```
# 1. Start with behavior search
trace search "user authentication"

# 2. Get details on relevant scenario
trace context tests/scenarios/test_auth.py::test_successful_login

# 3. Before modifying code, find affected tests
trace affected src/auth/login.py:25

# 4. Run affected tests
trace run tests/scenarios/test_auth.py::test_successful_login
```

### Understanding Coverage Context

The `trace context` output tells you:
- **files**: Which source files were executed
- **lines**: Specific line numbers that ran
- **functions**: Which functions were called, with their docstrings

Use this to:
- Understand code paths for a behavior
- Find related code when implementing similar features
- Identify what code to modify for bug fixes

### Best Practices

1. **Search by behavior first**: Don't search by implementation details
2. **Read docstrings**: They explain the business intent
3. **Compare success/error cases**: Use `--filter error` to find error handling scenarios
4. **Check coverage before changes**: Run `trace affected` before modifying code
