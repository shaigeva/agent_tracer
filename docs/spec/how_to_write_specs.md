# How to Write Specifications and Tests

This guide defines how to write behavioral specifications and corresponding tests in this project.

**Related guides**:
- [how_to_implement_tasks.md](../how_to_implement_tasks.md) - Task implementation process
- [main_spec.md](main_spec.md) - Project specification
- [high_level_architecture.md](../tech_spec/high_level_architecture.md) - Technical architecture

## How to Update Specifications

### Specification File Structure

**Main spec** (`docs/spec/main_spec.md`):
- High-level feature list
- Rationale for each feature
- Status and requirement counts (inline)
- Links to detailed specs

**Detailed specs** (`docs/spec/detailed/feature_xxx_detailed_spec.md`):
- Detailed requirements with unique IDs (REQ-XXX-YYY)
- Scenario descriptions
- Observable behaviors
- Acceptance criteria
- Edge cases
- Status for each requirement (inline)

### Requirement Format

Each requirement must have:

1. **Unique ID**: `REQ-{FEATURE}-{NUMBER}`
   - Example: REQ-MARKER-001, REQ-CLI-003

2. **Status** (inline): Not Implemented | Implemented | Needs Fix

3. **Type**: Product Behavior (preferred) or Technical Behavior (if necessary)

4. **Scenario**: Explicit description of when this behavior occurs

5. **Observable Behavior**: What external systems can verify (high-level description)

6. **Acceptance Criteria**: Detailed, testable criteria for this behavior

7. **Edge Cases**: List of edge cases to consider

**Note**: Requirements define WHAT behaviors are needed, not HOW to test them. Test planning happens during task implementation.

### Status Tracking (Inline Only)

**No separate tracking files.** Status lives in spec files:

- Each requirement has its own status marker
- Feature status calculated from requirements (e.g., "4/8 implemented")
- Update status when behavior is implemented and tested

## Core Principle: Product Behavior Over Technical Details

### What is Product Behavior?

**Product behavior** is what users or external systems can observe and verify.
**Technical details** are implementation choices that users cannot observe.

**Examples:**

GOOD - Product Behavior:
- "After running `trace build`, `trace list` returns all scenarios"
- "When searching for 'auth', scenarios with 'authentication' behavior are returned"
- "`trace context` includes function names with their docstrings"

BAD - Technical Details:
- "Scenario is written to SQLite database"
- "The functions table has a row inserted"
- "The parser creates a ScenarioMetadata struct"

### Why Product Behavior Matters

**Product behavior tests are robust:**
- They verify what users actually care about
- They don't break when implementation changes
- They test through the external interface

**Technical detail tests are fragile:**
- They break when you rename a table or change internal structure
- They test things users cannot observe
- They don't prove the product actually works

### The "So What?" Test

When writing a requirement, ask: **"So what? How would a user/agent know this happened?"**

BAD: "When user runs trace build, index is created"
- So what? What can the user DO with that information?

GOOD: "After running trace build, trace list returns scenarios from the coverage file"
- This is testable through CLI
- This is what users need to accomplish their goals

## How to Write Requirements

### Requirement Structure

Each requirement must have:

1. **Unique ID**: `REQ-{FEATURE}-{NUMBER}`
2. **Status**: Not Implemented | Implemented | Needs Fix
3. **Type**: Product Behavior (preferred) or Technical Behavior (if necessary)
4. **Scenario**: When does this happen?
5. **Observable Behavior**: What can external systems observe?
6. **Acceptance Criteria**: Detailed, testable criteria

### Template

```markdown
### REQ-XXX-001: [Short description of product behavior]
**Status**: Not Implemented
**Type**: Product Behavior

**Scenario**:
When [user/system action occurs]

**Observable Behavior**:
[High-level description of what happens that external systems can verify]

**Acceptance Criteria**:
- After running command, output contains expected data
- Error cases return appropriate error messages
- Edge cases are handled correctly

**Edge Cases**:
- Empty input
- Invalid input
- Missing dependencies
```

### Example: Good Product Behavior Requirement

```markdown
### REQ-CLI-004: Scenario context retrieval
**Status**: Not Implemented
**Type**: Product Behavior

**Scenario**:
When a user runs `trace context <scenario_id>` for an existing scenario

**Observable Behavior**:
User receives JSON output containing the scenario metadata and all files/functions covered by that scenario.

**Acceptance Criteria**:
- Output includes scenario id, description, behaviors, and outcome
- Output includes coverage.files with list of covered files
- Each file includes lines array and functions array
- Functions include name, line range, and docstring
- Non-existent scenario returns error with SCENARIO_NOT_FOUND code
- Invalid scenario_id format returns helpful error message

**Edge Cases**:
- Scenario with no coverage (test didn't execute any source code)
- Scenario covering files that no longer exist
- Scenario with very long docstring
```

### Example: Bad Requirement (Too Technical)

DON'T DO THIS:

```markdown
### REQ-CLI-004: Index database has context data
**Scenario**: When user runs trace build
**Behavior**:
- SQLite database is created
- scenarios table is populated
- coverage table has foreign keys

**Tests**:
- Check database file exists
- Query tables directly
```

**Why this is bad:**
- "SQLite database is created" - implementation detail
- Tests query database directly - fragile
- Doesn't prove user can DO anything with the created index

## Test Guidelines

### Test Through External Interface

For this project, the external interfaces are:
- **CLI commands**: Test via subprocess or command execution
- **MCP tools**: Test via MCP protocol
- **Python markers**: Test via pytest collection

### Test Naming Convention

Test names should describe **what behavior is verified**:

GOOD:
- `test_trace_list_returns_all_scenarios`
- `test_trace_search_finds_by_behavior`
- `test_trace_context_includes_functions`
- `test_trace_affected_finds_covering_tests`

BAD:
- `test_index_creation` (what about it?)
- `test_database_insert` (implementation detail)
- `test_parser` (too vague)

### Test Implementation Guidelines

**Tests MUST:**
- Verify behavior through CLI output
- Use the same interface users would use
- Verify complete workflows (build → query → verify)
- Test edge cases defined in spec

**Tests MUST NOT:**
- Query SQLite database directly to verify state
- Check internal struct fields
- Depend on internal file formats
- Assume specific implementation details

### Example - Good Test (Rust)

```rust
#[test]
fn test_trace_list_returns_all_scenarios() {
    // Setup: create test coverage and metadata files
    let temp_dir = setup_test_environment();

    // Build index
    let build_output = Command::new("trace")
        .args(["build", "--coverage", &coverage_path, "--scenarios", &metadata_path])
        .output()
        .expect("Failed to run trace build");
    assert!(build_output.status.success());

    // List scenarios
    let list_output = Command::new("trace")
        .args(["list"])
        .output()
        .expect("Failed to run trace list");

    let result: Value = serde_json::from_slice(&list_output.stdout).unwrap();
    let scenarios = result["scenarios"].as_array().unwrap();

    assert_eq!(scenarios.len(), 3);
    assert!(scenarios.iter().any(|s| s["id"] == "test_auth.py::test_login"));
}
```

### Example - Bad Test (Don't Do This)

```rust
#[test]
fn test_index_has_scenarios_table() {
    // DON'T DO THIS - queries database directly
    let conn = Connection::open(".trace-index/index.db").unwrap();
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM scenarios",
        [],
        |row| row.get(0)
    ).unwrap();
    assert!(count > 0);  // BAD - depends on internal schema
}
```

## Edge Cases and Test Coverage

### What to Test

For each requirement, tests must cover:

1. **Happy path** - Normal, valid usage
2. **Error conditions** - Invalid input, missing files, not found
3. **Boundary values** - Empty results, single result, many results
4. **Edge cases from spec** - Special characters, long strings, etc.

### Example Edge Case Coverage

```markdown
**Test Specification for REQ-CLI-002 (trace list)**:

**Happy Path:**
- List with multiple scenarios → returns all
- List with behavior filter → returns matching only
- List with outcome filter → returns matching only

**Error Conditions:**
- List before build → error with hint to run build
- List with invalid filter value → error message

**Boundary Values:**
- List with no scenarios → empty array, not error
- List with one scenario → single-element array
- List with 1000 scenarios → all returned (pagination future)

**Edge Cases:**
- Scenarios with special characters in description
- Scenarios with unicode in docstring
- Scenarios with empty behaviors list
```

## Summary: The Rules

### For Specifications

1. **Focus on product behavior** - what users observe
2. **Avoid technical details** - implementation is flexible
3. **Include acceptance criteria** - specific, testable
4. **Include edge cases** - don't leave to interpretation
5. **Status is inline** - no separate tracking files

### For Tests

1. **Test through CLI/MCP** - external interfaces
2. **Don't query database directly** - fragile and not observable
3. **Verify complete workflows** - build → query → verify
4. **Test names describe behavior** - not implementation
5. **Cover all scenarios from spec** - happy path + edge cases

### The Golden Rule

**If a user or agent cannot observe it, don't specify it or test it directly.**

Test the **externally observable consequences** of internal behavior.
