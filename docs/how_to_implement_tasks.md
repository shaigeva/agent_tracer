# How to Implement Tasks

This guide defines the process for implementing tasks from the task list.

## Task Implementation Process

When user approves implementing tasks, work autonomously through each task using this standardized process.

Tasks follow a plan-implement-validate feedback loop, described below.

### Standardized Progress Messages

Use these exact format messages to show progress:

```markdown
### Starting Task: [Task Name]
**Implements**: REQ-XXX-YYY, REQ-XXX-ZZZ
**Status**: Not Started → In Progress

---

### Planning Task: [Task Name]
Reading spec requirements...
[description of what you're doing]

---

### Writing Implementation Plan
Creating task_implementation_plan.md...

---

### Implementing: [Task Name]
[concise description of what's being implemented]

---

### Validating: [Task Name]
Running ./devtools/run_all_agent_validations.sh...

---

### Updating Status
- Task status: In Progress → Completed
- Requirements: Not Implemented → Implemented
```

## Step 1: Plan Task

**Message**: `### PLANNING TASK: [Task Name]`

**Actions**:
1. Read ALL relevant spec requirements this task implements
2. Understand the observable behaviors required
3. Check if any existing code/tests need updates
4. Design comprehensive test coverage (see below)
5. Write task implementation plan file

### Task Implementation Plan File

**File**: `docs/tasks/task_implementation_plan.md` (overwrites each task)

**Purpose**: Detailed planning document for current task implementation

**Format**:
```markdown
# Task Implementation Plan: [Task Name]

**Task Status**: In Progress
**Date**: YYYY-MM-DD
**Implements Requirements**: REQ-XXX-YYY, REQ-XXX-ZZZ

## Behaviors to Implement

### From REQ-XXX-YYY: [Requirement Title]
**Observable Behavior**:
- [What external systems can verify]
- [CLI commands/outputs involved]
- [State changes observable through CLI]

**Acceptance Criteria**:
- [Criterion 1 from spec]
- [Criterion 2 from spec]

### From REQ-XXX-ZZZ: [Another Requirement]
[Same format...]

## Implementation Plan

### Python Project Changes (if applicable)
- [ ] Create/modify markers.py
- [ ] Create/modify collector.py
- [ ] Add data models in models.py
- [ ] Update __init__.py exports

### Rust Project Changes (if applicable)
- [ ] Create/modify CLI commands in src/cli/
- [ ] Create/modify index modules in src/index/
- [ ] Create/modify query modules in src/query/
- [ ] Update main.rs CLI structure

### Other Changes
- [ ] Update Cargo.toml dependencies
- [ ] Update pyproject.toml dependencies
- [ ] [Any other changes needed]

## Test Planning

### 1. Python Tests (if applicable)
**File**: projects/pytest_tracer_python/tests/test_*.py

**Tests for REQ-XXX-YYY**:
- `test_collector_extracts_scenario_metadata`
  - Verifies: Collector finds scenario tests
  - Steps: Create test file with markers → run collector → verify output

- `test_collector_parses_docstring_structure`
  - Verifies: GIVEN/WHEN/THEN parsed correctly
  - Steps: Create test with structured docstring → collect → verify

[List ALL Python tests needed]

### 2. Rust CLI Tests
**File**: projects/trace_analyzer/tests/test_*.rs

**Tests for REQ-XXX-YYY**:
- `test_trace_list_returns_all_scenarios`
  - Verifies: CLI outputs all scenarios
  - Steps: Setup test data → run `trace list` → verify JSON output

- `test_trace_search_finds_by_behavior`
  - Verifies: Search filters by behavior
  - Steps: Setup data → run `trace search` → verify filtered results

[List ALL Rust tests needed]

**Edge Cases**:
- Empty index
- Missing files
- Invalid JSON
- Large datasets
- Special characters in descriptions
- [Other edge cases from spec]

### 3. Integration Tests
**File**: projects/trace_analyzer/tests/integration_test.rs

**Full workflow tests**:
- `test_full_workflow_build_list_context`
  - Steps: Create coverage + metadata → build → list → context → verify

[List integration tests]

## Existing Tests to Update

- [ ] tests/test_abc.py - Update because [reason]
- [ ] tests/test_xyz.rs - Update because [reason]

## Dependencies

**Requires completion of**:
- Task N (if any)

**Blocks**:
- Task M (if any)

## Notes

[Any additional implementation notes, concerns, or decisions]
```

## Step 2: Implement Task

**Message**: `### IMPLEMENTING: [Task Name]`

**Actions**:
1. Implement all code as planned
2. Implement ALL tests as planned
3. Ensure complete coverage of all requirements
4. No partial implementations

**Implementation Order (Python)**:
1. Data models (if needed)
2. Core logic
3. Unit tests (verify logic works)
4. Integration points

**Implementation Order (Rust)**:
1. Data structures / types
2. Core logic modules
3. CLI integration
4. Tests (verify complete workflows)

## Step 3: Validate

**Message**: `### VALIDATING: [Task Name]`

**Actions**:
1. Run appropriate validation script:
   - All projects: `./devtools/run_all_agent_validations.sh` (from root)
   - Python only: `cd projects/pytest_tracer_python && ./devtools/run_all_agent_validations.sh`
   - Rust only: `cd projects/trace_analyzer && ./devtools/run_all_agent_validations.sh`
2. Fix any failures (see validation feedback loop below)
3. Repeat until ZERO errors/warnings

### Validation Feedback Loop

When validation fails:

1. **Identify failure** - test, lint, type error, clippy?

2. **Check spec FIRST**:
   - Re-read relevant spec section
   - Confirm correct behavior
   - Verify understanding matches spec

3. **Determine fix**:
   - Code wrong? → Fix code to match spec
   - Test wrong? → Verify against spec, then fix test
   - NEVER change tests just to make them pass

4. **Apply fix and re-run** - Repeat until passing

**ZERO TOLERANCE**
- ZERO test failures
- ZERO linting errors (ruff for Python, clippy for Rust)
- ZERO type errors (ty for Python)
- ZERO warnings

**ONLY 2 ACCEPTABLE OUTCOMES**
- All validations pass
- You've tried to fix and failed (tell user)

## Step 4: Update Status

**Message**: `### Updating Status`

**Actions**:
1. Update task status in `docs/tasks/current_task_list.md`: In Progress → Completed
2. Update requirement statuses in spec files: Not Implemented → Implemented
3. Update feature status counts if needed

## Step 5: Request Commit Approval

**Message**: `### Task Complete - Ready to Commit`

**Actions**:
1. Summarize what was implemented
2. List requirements completed
3. Confirm all validations passed
4. Ask user for approval to commit

**Format**:
```
### Task Complete - Ready to Commit

**Implemented**: [Task name]
**Requirements completed**: REQ-XXX-YYY, REQ-XXX-ZZZ
**Tests added**: [count] Python tests, [count] Rust tests
**Validations**: All passed (zero errors/warnings)

Ready to commit?
```

## Test Coverage Requirements

Every task implementation MUST include tests for:

### 1. CLI Behavior Tests (Rust) - ALWAYS REQUIRED
- Test through CLI command execution
- Verify JSON output structure
- Test all scenarios from spec
- Cover all edge cases
- **File location**: `projects/trace_analyzer/tests/`

### 2. Python Unit Tests - IF APPLICABLE
- Test collector functionality
- Test marker behavior
- Test metadata extraction
- **File location**: `projects/pytest_tracer_python/tests/`

### 3. Integration Tests - FOR CROSS-PROJECT FEATURES
- Test full workflows (Python coverage → Rust index → queries)
- Test real pytest-cov output parsing
- **File location**: `projects/trace_analyzer/tests/integration_test.rs`

## Important Notes

### Complete Capabilities Only
- No partial implementations
- All requirements in task must be 100% done
- All planned tests must be implemented
- All validations must pass

### Test Through External Interfaces
For this project, external interfaces are:
- **CLI commands**: Test via subprocess execution
- **JSON output**: Verify structure and content
- **Python functions**: Test via pytest

### Test Planning is Major Work
- Test planning is not an afterthought
- Spend significant time designing tests
- Document all planned tests in implementation plan
- Tests prove the spec is implemented correctly

### Multi-Project Considerations
- Some tasks touch both Python and Rust projects
- Run validations for both when making cross-project changes
- Integration tests may require running Python first, then Rust
