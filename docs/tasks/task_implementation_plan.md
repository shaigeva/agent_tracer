# Task Implementation Plan: Python Package - Scenario Metadata Collection

**Task Status**: In Progress
**Date**: 2024-11-27
**Implements Requirements**: REQ-MARKER-001 through REQ-MARKER-004, REQ-META-001 through REQ-META-005

## Design Rationale

### Testing Strategy: Content-Addressable Coverage Cache

**Problem**: Testing the scenario collector requires running pytest with coverage, but:
1. Running pytest-within-pytest causes issues (shared state, plugin conflicts)
2. Running subprocess for every test is slow
3. We need reliable, repeatable test data

**Solution**: Content-addressable cache with subprocess generation

```
tests/fixtures/
├── sample_project/           # Example tests + code (checked into git)
│   ├── src/                  # Code being tested
│   └── tests/                # Scenario tests with markers
└── cache/                    # Gitignored - auto-generated
    └── {content_hash}/       # Hash of sample_project contents
        ├── .coverage         # Cached coverage database
        └── scenarios.json    # Cached scenario metadata
```

**How it works**:
1. Compute SHA256 hash of all `.py` files in `sample_project/`
2. If `cache/{hash}/` exists → load cached data (instant)
3. If cache miss → run `pytest --cov` via subprocess (one-time cost)
4. All tests use cached data

**Why subprocess is acceptable**:
- Runs only on cache miss (first run or after sample_project changes)
- Clean process isolation (no pytest-in-pytest issues)
- Simple implementation
- pytester adds complexity without benefit here

**Why not pytester**:
- pytester is designed for testing pytest plugins (hooks, markers behavior)
- We're testing our data extraction, not pytest behavior
- pytester still uses subprocess internally
- Adds dependency complexity for no gain

### Module Structure

```
pytest_tracer_python/
├── __init__.py          # Public API exports
├── markers.py           # Marker re-exports (scenario, behavior, error)
├── models.py            # Pydantic models for scenario metadata
├── cache.py             # Content-addressable cache management
├── collector.py         # Scenario extraction using pytest collection
└── cli.py               # CLI entry point (future)
```

### Testability Layers

| Module | What to Test | How |
|--------|-------------|-----|
| `models.py` | Pydantic validation, JSON serialization | Direct instantiation |
| `cache.py` | Hash computation, cache hit/miss logic | Temp directories |
| `collector.py` | Scenario extraction | Cached coverage fixture |

## Behaviors to Implement

### From REQ-MARKER-001: Scenario marker
**Observable Behavior**: Tests decorated with `@pytest.mark.scenario` are identified as scenario tests

### From REQ-MARKER-002: Behavior marker
**Observable Behavior**: Tests can have multiple `@pytest.mark.behavior("name")` markers, all captured

### From REQ-MARKER-003: Error marker
**Observable Behavior**: Tests with `@pytest.mark.error` are identified as error scenarios

### From REQ-MARKER-004: Docstring format
**Observable Behavior**: First line of docstring is description; GIVEN/WHEN/THEN sections parsed

### From REQ-META-001 through REQ-META-005: Metadata collection
**Observable Behavior**: Collector produces JSON with all scenario metadata

## Implementation Plan

### 1. models.py
- [ ] `ScenarioMetadata` - id, file, function, description, documentation, behaviors, outcome
- [ ] `ScenariosFile` - version, collected_at, scenarios list
- [ ] JSON serialization methods

### 2. markers.py
- [ ] Re-export `pytest.mark.scenario`
- [ ] Re-export `pytest.mark.behavior`
- [ ] Re-export `pytest.mark.error`

### 3. cache.py
- [ ] `compute_content_hash(directory)` - SHA256 of all .py files
- [ ] `CoverageCache` class - load/save cached data
- [ ] `get_or_create_cache()` - cache hit/miss logic
- [ ] Subprocess runner for cache generation

### 4. collector.py
- [ ] `ScenarioCollectorPlugin` - pytest plugin for collection hooks
- [ ] `extract_scenario_from_item()` - extract metadata from pytest Item
- [ ] `collect_scenarios()` - run collection via pytest.main()
- [ ] `parse_docstring()` - extract description and GIVEN/WHEN/THEN

### 5. Sample project fixture
- [ ] Create `tests/fixtures/sample_project/src/` with example code
- [ ] Create `tests/fixtures/sample_project/tests/` with scenario tests
- [ ] Include various marker combinations for testing

## Test Planning

### 1. Unit Tests - models.py
**File**: `tests/test_models.py`

- `test_scenario_metadata_required_fields` - validation works
- `test_scenario_metadata_json_roundtrip` - serialize/deserialize
- `test_scenarios_file_version_included` - version in output

### 2. Unit Tests - cache.py
**File**: `tests/test_cache.py`

- `test_content_hash_deterministic` - same input → same hash
- `test_content_hash_changes_on_file_change` - detects modifications
- `test_content_hash_ignores_non_py_files` - only .py files
- `test_cache_hit_loads_existing` - uses temp dir with pre-made cache
- `test_cache_miss_generates_new` - generates when missing

### 3. Integration Tests - collector.py
**File**: `tests/test_collector.py`

Uses `coverage_cache` session fixture:

- `test_collector_finds_scenario_tests` - finds @scenario markers
- `test_collector_extracts_behaviors` - multiple behaviors captured
- `test_collector_identifies_error_scenarios` - @error marker
- `test_collector_parses_docstring_description` - first line
- `test_collector_parses_docstring_sections` - GIVEN/WHEN/THEN
- `test_collector_generates_valid_json` - output matches schema

### 4. Sample Project Contents

```python
# tests/fixtures/sample_project/src/auth.py
def login(email: str, password: str) -> dict:
    if password == "valid":
        return {"status": 200, "token": "abc123"}
    return {"status": 401, "error": "Invalid credentials"}

# tests/fixtures/sample_project/tests/test_auth.py
import pytest
from src.auth import login

@pytest.mark.scenario
@pytest.mark.behavior("authentication")
@pytest.mark.behavior("session-management")
def test_successful_login():
    """
    User logs in with valid credentials

    GIVEN a registered user
    WHEN they submit valid credentials
    THEN they receive an auth token
    """
    result = login("user@example.com", "valid")
    assert result["status"] == 200
    assert result["token"] is not None

@pytest.mark.scenario
@pytest.mark.behavior("authentication")
@pytest.mark.error
def test_login_invalid_password():
    """Login fails with wrong password"""
    result = login("user@example.com", "wrong")
    assert result["status"] == 401
```

## Dependencies

**Requires**: None (first task)
**Blocks**: Rust analyzer index building

## Notes

- Cache directory should be in `.gitignore`
- First test run will be slow (cache generation), subsequent runs fast
- Consider adding `--regenerate-cache` option for forcing cache rebuild
