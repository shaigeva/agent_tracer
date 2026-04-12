"""
Call tracer using sys.monitoring (Python 3.12+).

Records function CALL and RETURN events during test execution,
filtering out standard library and third-party dependencies.
The output is a per-test call trace that can be used to generate
flame graphs and call-chain diagrams.
"""

import json
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

# sys.monitoring tool ID - use PROFILER_ID to avoid conflicts with debuggers
TOOL_ID = sys.monitoring.PROFILER_ID

# Paths to filter out (dependencies, stdlib, etc.)
_FILTER_SUBSTRINGS: tuple[str, ...] = (
    "site-packages",
    "/lib/python",
    "/lib64/python",
    "/.venv/",
    "/venv/",
    "importlib",
    "_pytest/",
    "pluggy/",
    "pytest_cov/",
    "coverage/",
)


@dataclass
class CallEvent:
    """A single call or return event."""

    event: str  # "call" or "return"
    file: str
    function: str
    line: int
    timestamp_ns: int
    depth: int


@dataclass
class TestCallTrace:
    """Call trace for a single test."""

    test_id: str
    events: list[CallEvent] = field(default_factory=list)


class CallTracer:
    """Records function call/return events using sys.monitoring."""

    def __init__(self, project_root: str | Path) -> None:
        self._project_root = str(Path(project_root).resolve())
        self._current_trace: TestCallTrace | None = None
        self._depth: int = 0
        self._active: bool = False

    def _should_trace(self, filename: str) -> bool:
        """Only trace files under the project root, excluding dependencies."""
        if not filename.startswith(self._project_root):
            return False
        for substr in _FILTER_SUBSTRINGS:
            if substr in filename:
                return False
        return True

    def _on_py_start(self, code: object, instruction_offset: int) -> None:
        """Callback for PY_START (function call) events."""
        if self._current_trace is None:
            return

        # code is a code object
        filename = code.co_filename  # type: ignore[attr-defined]
        if not self._should_trace(filename):
            return

        qualname = code.co_qualname  # type: ignore[attr-defined]
        lineno = code.co_firstlineno  # type: ignore[attr-defined]

        self._current_trace.events.append(
            CallEvent(
                event="call",
                file=filename,
                function=qualname,
                line=lineno,
                timestamp_ns=time.monotonic_ns(),
                depth=self._depth,
            )
        )
        self._depth += 1

    def _on_py_return(self, code: object, instruction_offset: int, retval: object) -> None:
        """Callback for PY_RETURN (function return) events."""
        if self._current_trace is None:
            return

        filename = code.co_filename  # type: ignore[attr-defined]
        if not self._should_trace(filename):
            return

        qualname = code.co_qualname  # type: ignore[attr-defined]
        lineno = code.co_firstlineno  # type: ignore[attr-defined]

        self._depth = max(0, self._depth - 1)
        self._current_trace.events.append(
            CallEvent(
                event="return",
                file=filename,
                function=qualname,
                line=lineno,
                timestamp_ns=time.monotonic_ns(),
                depth=self._depth,
            )
        )

    def start(self) -> None:
        """Register monitoring callbacks and start tracing."""
        if self._active:
            return
        sys.monitoring.use_tool_id(TOOL_ID, "pytest-tracer")
        sys.monitoring.register_callback(TOOL_ID, sys.monitoring.events.PY_START, self._on_py_start)
        sys.monitoring.register_callback(TOOL_ID, sys.monitoring.events.PY_RETURN, self._on_py_return)
        sys.monitoring.set_events(TOOL_ID, sys.monitoring.events.PY_START | sys.monitoring.events.PY_RETURN)
        self._active = True

    def stop(self) -> None:
        """Stop tracing and unregister callbacks."""
        if not self._active:
            return
        sys.monitoring.set_events(TOOL_ID, 0)
        sys.monitoring.register_callback(TOOL_ID, sys.monitoring.events.PY_START, None)
        sys.monitoring.register_callback(TOOL_ID, sys.monitoring.events.PY_RETURN, None)
        sys.monitoring.free_tool_id(TOOL_ID)
        self._active = False

    def begin_test(self, test_id: str) -> None:
        """Start recording a new test's call trace."""
        self._current_trace = TestCallTrace(test_id=test_id)
        self._depth = 0

    def end_test(self) -> TestCallTrace | None:
        """Finish recording and return the trace."""
        trace = self._current_trace
        self._current_trace = None
        self._depth = 0
        return trace


class TracerPlugin:
    """Pytest plugin that records call traces per scenario test."""

    def __init__(self, project_root: str | Path) -> None:
        self.tracer = CallTracer(project_root)
        self.traces: list[TestCallTrace] = []

    def pytest_sessionstart(self, session: object) -> None:  # noqa: ANN001
        """Start the tracer at session begin."""
        self.tracer.start()

    def pytest_sessionfinish(self, session: object, exitstatus: int) -> None:  # noqa: ANN001
        """Stop the tracer at session end."""
        self.tracer.stop()

    def pytest_runtest_setup(self, item: object) -> None:  # noqa: ANN001
        """Begin tracing before each test."""
        node_id = item.nodeid  # type: ignore[attr-defined]
        # Only trace scenario-marked tests
        markers = [m.name for m in item.iter_markers()]  # type: ignore[attr-defined]
        if "scenario" in markers:
            self.tracer.begin_test(node_id)

    def pytest_runtest_teardown(self, item: object, nextitem: object) -> None:  # noqa: ANN001
        """End tracing after each test."""
        trace = self.tracer.end_test()
        if trace is not None and trace.events:
            self.traces.append(trace)


def traces_to_json(traces: list[TestCallTrace], project_root: str | Path) -> str:
    """Serialize call traces to JSON format for the Rust analyzer."""
    project_root_str = str(Path(project_root).resolve())

    def relativize(path: str) -> str:
        if path.startswith(project_root_str):
            rel = path[len(project_root_str) :]
            if rel.startswith("/"):
                rel = rel[1:]
            return rel
        return path

    data: dict[str, object] = {
        "version": "1.0",
        "traces": {},
    }

    traces_dict: dict[str, list[dict[str, object]]] = {}
    for trace in traces:
        events = []
        for e in trace.events:
            events.append(
                {
                    "event": e.event,
                    "file": relativize(e.file),
                    "function": e.function,
                    "line": e.line,
                    "depth": e.depth,
                    "timestamp_ns": e.timestamp_ns,
                }
            )
        traces_dict[trace.test_id] = events

    data["traces"] = traces_dict
    return json.dumps(data, indent=2)


def collect_call_traces(
    project_root: Path,
    test_dir: str = "tests",
) -> list[TestCallTrace]:
    """Run pytest with call tracing on scenario tests and return traces.

    Args:
        project_root: Path to the project root
        test_dir: Subdirectory containing tests

    Returns:
        List of call traces, one per scenario test
    """
    import subprocess
    import tempfile

    # We run pytest in a subprocess with a conftest that enables our tracer plugin.
    # This avoids polluting the current process's sys.monitoring state.
    tracer_script = f"""
import sys
import json
from pathlib import Path

# Add project root to path
sys.path.insert(0, {str(project_root)!r})

from pytest_tracer_python.tracer import TracerPlugin, traces_to_json

project_root = Path({str(project_root)!r})
plugin = TracerPlugin(project_root)

import pytest
exit_code = pytest.main([
    {str(project_root / test_dir)!r},
    "-q",
    "--override-ini=addopts=",
], plugins=[plugin])

# Write traces to stdout as JSON
output = traces_to_json(plugin.traces, project_root)
# Use a marker so we can find the JSON in output
print("__TRACES_START__")
print(output)
print("__TRACES_END__")
"""

    with tempfile.NamedTemporaryFile(mode="w", suffix=".py", delete=False) as f:
        f.write(tracer_script)
        script_path = f.name

    try:
        result = subprocess.run(
            [sys.executable, script_path],
            capture_output=True,
            text=True,
            cwd=str(project_root),
        )

        # Extract JSON from output
        stdout = result.stdout
        start_marker = "__TRACES_START__"
        end_marker = "__TRACES_END__"

        start_idx = stdout.find(start_marker)
        end_idx = stdout.find(end_marker)

        if start_idx == -1 or end_idx == -1:
            # No traces found - might be no scenario tests or an error
            if result.returncode != 0:
                print(f"Warning: pytest exited with code {result.returncode}", file=sys.stderr)
                if result.stderr:
                    print(result.stderr, file=sys.stderr)
            return []

        json_str = stdout[start_idx + len(start_marker) : end_idx].strip()
        data = json.loads(json_str)

        traces = []
        for test_id, events in data.get("traces", {}).items():
            trace = TestCallTrace(test_id=test_id)
            for e in events:
                trace.events.append(
                    CallEvent(
                        event=e["event"],
                        file=e["file"],
                        function=e["function"],
                        line=e["line"],
                        depth=e["depth"],
                        timestamp_ns=e["timestamp_ns"],
                    )
                )
            traces.append(trace)

        return traces

    finally:
        import os

        os.unlink(script_path)
