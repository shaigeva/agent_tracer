# Current Task List

## Task: Python Package - Scenario Metadata Collection
**Status**: ✅ Completed
**Requirements**: REQ-MARKER-001, REQ-MARKER-002, REQ-MARKER-003, REQ-MARKER-004, REQ-META-001, REQ-META-002, REQ-META-003, REQ-META-004, REQ-META-005

---

## Task: Rust trace_analyzer - Core Infrastructure
**Status**: ✅ Completed
**Requirements**: REQ-INDEX-001, REQ-INDEX-002

### Description
Implement core data structures and parsers for coverage data and scenario metadata.

### Subtasks
1. ✅ Coverage parser - Read `.coverage` SQLite database from pytest-cov
2. ✅ Scenario metadata parser - Read `scenarios.json` from Python collector
3. ✅ Core data models - Rust structs for scenarios, coverage, functions

---

## Task: Rust trace_analyzer - Index Building
**Status**: ✅ Completed
**Requirements**: REQ-INDEX-003, REQ-INDEX-004, REQ-INDEX-005, REQ-CLI-001

### Description
Build queryable SQLite index from coverage and scenario data.

### Subtasks
1. ✅ Index schema and storage
2. ⏸️ Line-to-function mapping (Python AST analysis) - deferred
3. ✅ `trace build` CLI command
4. ⏸️ Incremental index updates - deferred

---

## Task: Rust trace_analyzer - Query Commands
**Status**: ✅ Completed
**Requirements**: REQ-CLI-002, REQ-CLI-003, REQ-CLI-004, REQ-CLI-005, REQ-CLI-006, REQ-CLI-007, REQ-CLI-008

### Description
Implement CLI query commands with JSON output.

### Subtasks
1. ✅ `trace list` - List scenarios with filters
2. ✅ `trace search` - Search scenario descriptions
3. ✅ `trace context` - Get coverage context for scenario
4. ✅ `trace affected` - Find scenarios covering file/line
5. ⏸️ `trace run` - Run scenario with coverage - deferred

---

## Task: Rust trace_analyzer - MCP Server
**Status**: ✅ Completed
**Requirements**: REQ-MCP-001, REQ-MCP-002, REQ-MCP-003, REQ-MCP-004, REQ-MCP-005

### Description
Expose query capabilities via Model Context Protocol.

### Subtasks
1. ✅ MCP server infrastructure
2. ✅ scenario_list tool
3. ✅ scenario_search tool
4. ✅ scenario_context tool
5. ✅ coverage_affected_file tool
6. ✅ coverage_affected_line tool
