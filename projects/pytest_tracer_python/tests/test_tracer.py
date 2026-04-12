"""Tests for the call tracer using sys.monitoring."""

import json
from pathlib import Path

from pytest_tracer_python.tracer import collect_call_traces, traces_to_json

FIXTURES_DIR = Path(__file__).parent / "fixtures"


def test_collect_call_traces_sample_project_3() -> None:
    """Call tracer captures events across all layers of sample_project_3."""
    project_root = FIXTURES_DIR / "sample_project_3"
    traces = collect_call_traces(project_root, test_dir="tests")

    # Should have traces for scenario-marked tests
    assert len(traces) > 0

    # Find the full-stack test
    trace_ids = {t.test_id for t in traces}
    assert "tests/test_order_flow.py::test_add_item_to_order" in trace_ids

    # Check that the add_item trace covers multiple layers
    add_item_trace = next(t for t in traces if "test_add_item_to_order" in t.test_id)
    files_touched = {e.file for e in add_item_trace.events if e.event == "call"}

    # Should touch files across routes, middleware, services, repositories, models
    file_names = {f.split("/")[-1] for f in files_touched}
    assert "order_routes.py" in file_names, f"Missing routes layer. Files: {file_names}"
    assert "auth.py" in file_names, f"Missing middleware layer. Files: {file_names}"
    assert "order_service.py" in file_names, f"Missing service layer. Files: {file_names}"
    assert "order_repository.py" in file_names, f"Missing repository layer. Files: {file_names}"


def test_call_trace_has_depth() -> None:
    """Call traces include depth information for call nesting."""
    project_root = FIXTURES_DIR / "sample_project_3"
    traces = collect_call_traces(project_root, test_dir="tests")

    add_item_trace = next(t for t in traces if "test_add_item_to_order" in t.test_id)

    # Should have calls at different depths
    depths = {e.depth for e in add_item_trace.events}
    assert len(depths) > 1, f"All events at same depth: {depths}"
    assert max(depths) >= 2, f"Max depth too shallow: {max(depths)}"


def test_call_trace_filters_dependencies() -> None:
    """Call tracer filters out site-packages and stdlib."""
    project_root = FIXTURES_DIR / "sample_project_3"
    traces = collect_call_traces(project_root, test_dir="tests")

    for trace in traces:
        for event in trace.events:
            assert "site-packages" not in event.file, f"Dependency leaked: {event.file}"
            assert "/_pytest/" not in event.file, f"Pytest internals leaked: {event.file}"


def test_call_trace_has_call_return_pairs() -> None:
    """Each call event should have a matching return event."""
    project_root = FIXTURES_DIR / "sample_project_3"
    traces = collect_call_traces(project_root, test_dir="tests")

    for trace in traces:
        calls = [e for e in trace.events if e.event == "call"]
        returns = [e for e in trace.events if e.event == "return"]
        # Allow slight mismatch (exceptions can skip returns)
        # but the counts should be close
        assert abs(len(calls) - len(returns)) <= len(calls) * 0.1 + 1, (
            f"Call/return mismatch for {trace.test_id}: {len(calls)} calls, {len(returns)} returns"
        )


def test_traces_to_json_format() -> None:
    """JSON output has the expected structure."""
    project_root = FIXTURES_DIR / "sample_project_3"
    traces = collect_call_traces(project_root, test_dir="tests")

    json_str = traces_to_json(traces, project_root)
    data = json.loads(json_str)

    assert data["version"] == "1.0"
    assert "traces" in data
    assert isinstance(data["traces"], dict)

    # Check a trace entry has the right fields
    for _test_id, events in data["traces"].items():
        assert len(events) > 0
        first = events[0]
        assert "event" in first
        assert "file" in first
        assert "function" in first
        assert "line" in first
        assert "depth" in first
        assert "timestamp_ns" in first
        break


def test_traces_json_uses_relative_paths() -> None:
    """JSON output uses relative file paths, not absolute."""
    project_root = FIXTURES_DIR / "sample_project_3"
    traces = collect_call_traces(project_root, test_dir="tests")

    json_str = traces_to_json(traces, project_root)
    data = json.loads(json_str)

    for _test_id, events in data["traces"].items():
        for event in events:
            assert not event["file"].startswith("/"), f"Absolute path in JSON: {event['file']}"
