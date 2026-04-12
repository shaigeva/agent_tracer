"""
Command-line interface for pytest-tracer scenario collection and call tracing.

Usage:
    pytest-tracer collect [OPTIONS] PROJECT_ROOT
    pytest-tracer trace [OPTIONS] PROJECT_ROOT
"""

import argparse
import sys
from pathlib import Path

from .collector import collect_scenarios


def create_parser() -> argparse.ArgumentParser:
    """Create the argument parser for the CLI."""
    parser = argparse.ArgumentParser(
        prog="pytest-tracer",
        description="Collect scenario test metadata and call traces for trace analysis",
    )

    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # collect command
    collect_parser = subparsers.add_parser(
        "collect",
        help="Collect scenario metadata from pytest tests",
        description="Scan pytest tests for @scenario markers and extract metadata",
    )
    collect_parser.add_argument(
        "project_root",
        type=Path,
        help="Path to project root directory",
    )
    collect_parser.add_argument(
        "--test-dir",
        type=str,
        default="tests",
        help="Subdirectory containing tests (default: tests)",
    )
    collect_parser.add_argument(
        "-o",
        "--output",
        type=Path,
        default=None,
        help="Output file path (default: stdout)",
    )

    # trace command
    trace_parser = subparsers.add_parser(
        "trace",
        help="Collect call traces from scenario tests using sys.monitoring",
        description="Run scenario tests with call tracing and output call_traces.json",
    )
    trace_parser.add_argument(
        "project_root",
        type=Path,
        help="Path to project root directory",
    )
    trace_parser.add_argument(
        "--test-dir",
        type=str,
        default="tests",
        help="Subdirectory containing tests (default: tests)",
    )
    trace_parser.add_argument(
        "-o",
        "--output",
        type=Path,
        default=None,
        help="Output file path (default: stdout)",
    )

    return parser


def cmd_collect(args: argparse.Namespace) -> int:
    """Execute the collect command."""
    project_root = args.project_root.resolve()

    if not project_root.exists():
        print(f"Error: Project root does not exist: {project_root}", file=sys.stderr)
        return 1

    if not project_root.is_dir():
        print(f"Error: Project root is not a directory: {project_root}", file=sys.stderr)
        return 1

    test_path = project_root / args.test_dir
    if not test_path.exists():
        print(f"Error: Test directory does not exist: {test_path}", file=sys.stderr)
        return 1

    # Collect scenarios
    scenarios = collect_scenarios(project_root, args.test_dir)

    # Output JSON
    json_output = scenarios.to_json()

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json_output)
    else:
        print(json_output)

    return 0


def cmd_trace(args: argparse.Namespace) -> int:
    """Execute the trace command - collect call traces using sys.monitoring."""
    from .tracer import collect_call_traces, traces_to_json

    project_root = args.project_root.resolve()

    if not project_root.exists():
        print(f"Error: Project root does not exist: {project_root}", file=sys.stderr)
        return 1

    if not project_root.is_dir():
        print(f"Error: Project root is not a directory: {project_root}", file=sys.stderr)
        return 1

    test_path = project_root / args.test_dir
    if not test_path.exists():
        print(f"Error: Test directory does not exist: {test_path}", file=sys.stderr)
        return 1

    # Collect traces
    traces = collect_call_traces(project_root, args.test_dir)

    if not traces:
        print("Warning: No call traces collected. Are there @scenario-marked tests?", file=sys.stderr)
        return 0

    # Output JSON
    json_output = traces_to_json(traces, project_root)

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json_output)
        print(f"Collected {len(traces)} call traces to {args.output}", file=sys.stderr)
    else:
        print(json_output)

    return 0


def main(argv: list[str] | None = None) -> int:
    """Main entry point for the CLI."""
    parser = create_parser()
    args = parser.parse_args(argv)

    if args.command is None:
        parser.print_help()
        return 0

    if args.command == "collect":
        return cmd_collect(args)

    if args.command == "trace":
        return cmd_trace(args)

    # Should not reach here due to argparse
    return 1


if __name__ == "__main__":
    sys.exit(main())
