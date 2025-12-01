"""
Tests for the CLI module.

These tests verify:
- Argument parsing
- Command execution
- Output format
- Error handling
"""

import json
from pathlib import Path

import pytest

from pytest_tracer_python.cli import cmd_collect, create_parser, main

# Path to sample_project fixture
FIXTURES_DIR = Path(__file__).parent / "fixtures"
SAMPLE_PROJECT = FIXTURES_DIR / "sample_project"


class TestArgumentParser:
    """Tests for CLI argument parsing."""

    def test_no_command_returns_none(self) -> None:
        """No command provided results in command=None."""
        parser = create_parser()
        args = parser.parse_args([])
        assert args.command is None

    def test_collect_command_parsed(self) -> None:
        """Collect command is recognized."""
        parser = create_parser()
        args = parser.parse_args(["collect", "/some/path"])
        assert args.command == "collect"
        assert args.project_root == Path("/some/path")

    def test_collect_default_test_dir(self) -> None:
        """Default test directory is 'tests'."""
        parser = create_parser()
        args = parser.parse_args(["collect", "/some/path"])
        assert args.test_dir == "tests"

    def test_collect_custom_test_dir(self) -> None:
        """Custom test directory can be specified."""
        parser = create_parser()
        args = parser.parse_args(["collect", "/some/path", "--test-dir", "test"])
        assert args.test_dir == "test"

    def test_collect_output_default_none(self) -> None:
        """Default output is None (stdout)."""
        parser = create_parser()
        args = parser.parse_args(["collect", "/some/path"])
        assert args.output is None

    def test_collect_output_specified(self) -> None:
        """Output file can be specified."""
        parser = create_parser()
        args = parser.parse_args(["collect", "/some/path", "-o", "output.json"])
        assert args.output == Path("output.json")

    def test_collect_output_long_form(self) -> None:
        """Output file can be specified with --output."""
        parser = create_parser()
        args = parser.parse_args(["collect", "/some/path", "--output", "output.json"])
        assert args.output == Path("output.json")


class TestCollectCommand:
    """Tests for the collect command execution."""

    def test_collect_nonexistent_project(self, tmp_path: Path) -> None:
        """Error when project root doesn't exist."""
        parser = create_parser()
        args = parser.parse_args(["collect", str(tmp_path / "nonexistent")])
        result = cmd_collect(args)
        assert result == 1

    def test_collect_project_is_file(self, tmp_path: Path) -> None:
        """Error when project root is a file, not directory."""
        file_path = tmp_path / "file.txt"
        file_path.write_text("content")

        parser = create_parser()
        args = parser.parse_args(["collect", str(file_path)])
        result = cmd_collect(args)
        assert result == 1

    def test_collect_missing_test_dir(self, tmp_path: Path) -> None:
        """Error when test directory doesn't exist."""
        parser = create_parser()
        args = parser.parse_args(["collect", str(tmp_path)])
        result = cmd_collect(args)
        assert result == 1

    def test_collect_sample_project_to_stdout(self, capsys: pytest.CaptureFixture[str]) -> None:
        """Collect outputs JSON to stdout by default."""
        parser = create_parser()
        args = parser.parse_args(["collect", str(SAMPLE_PROJECT)])
        result = cmd_collect(args)

        assert result == 0

        captured = capsys.readouterr()
        output = json.loads(captured.out)

        assert "version" in output
        assert "scenarios" in output
        assert len(output["scenarios"]) == 10  # 10 scenario tests in sample_project

    def test_collect_sample_project_to_file(self, tmp_path: Path) -> None:
        """Collect outputs JSON to file when -o specified."""
        output_file = tmp_path / "scenarios.json"

        parser = create_parser()
        args = parser.parse_args(["collect", str(SAMPLE_PROJECT), "-o", str(output_file)])
        result = cmd_collect(args)

        assert result == 0
        assert output_file.exists()

        output = json.loads(output_file.read_text())
        assert "version" in output
        assert "scenarios" in output
        assert len(output["scenarios"]) == 10

    def test_collect_creates_output_directory(self, tmp_path: Path) -> None:
        """Collect creates parent directories for output file."""
        output_file = tmp_path / "subdir" / "nested" / "scenarios.json"

        parser = create_parser()
        args = parser.parse_args(["collect", str(SAMPLE_PROJECT), "-o", str(output_file)])
        result = cmd_collect(args)

        assert result == 0
        assert output_file.exists()

    def test_collect_scenario_content(self, capsys: pytest.CaptureFixture[str]) -> None:
        """Collected scenarios have expected content."""
        parser = create_parser()
        args = parser.parse_args(["collect", str(SAMPLE_PROJECT)])
        cmd_collect(args)

        captured = capsys.readouterr()
        output = json.loads(captured.out)

        # Find a specific scenario
        login_scenario = next(
            (s for s in output["scenarios"] if "test_successful_login" in s["id"]),
            None,
        )
        assert login_scenario is not None
        assert login_scenario["description"] == "User logs in with valid credentials"
        assert "authentication" in login_scenario["behaviors"]
        assert login_scenario["outcome"] == "success"


class TestMainEntryPoint:
    """Tests for the main() entry point."""

    def test_main_no_args_returns_zero(self) -> None:
        """No arguments shows help and returns 0."""
        result = main([])
        assert result == 0

    def test_main_collect_success(self, capsys: pytest.CaptureFixture[str]) -> None:
        """Main with collect command succeeds."""
        result = main(["collect", str(SAMPLE_PROJECT)])
        assert result == 0

        captured = capsys.readouterr()
        output = json.loads(captured.out)
        assert "scenarios" in output

    def test_main_collect_error(self, tmp_path: Path) -> None:
        """Main with invalid project returns error code."""
        result = main(["collect", str(tmp_path / "nonexistent")])
        assert result == 1
