# Current Task List

## Task: Python Package - Scenario Metadata Collection
**Status**: In Progress
**Requirements**: REQ-MARKER-001, REQ-MARKER-002, REQ-MARKER-003, REQ-MARKER-004, REQ-META-001, REQ-META-002, REQ-META-003, REQ-META-004, REQ-META-005

### Description
Implement the Python package for scenario test markers and metadata collection. This includes:
- Pytest marker definitions
- Scenario metadata extraction using pytest's collection API
- Coverage cache system for fast, repeatable testing
- JSON export for Rust analyzer consumption

### Subtasks
1. `models.py` - Pydantic models for scenario metadata
2. `markers.py` - Pytest marker re-exports
3. `cache.py` - Content-addressable coverage cache
4. `collector.py` - Scenario extraction using pytest collection
5. Sample project fixture for testing
6. Unit and integration tests
