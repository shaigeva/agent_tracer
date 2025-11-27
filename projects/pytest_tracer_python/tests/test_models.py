"""
Tests for Pydantic models.

These tests verify:
- Model validation
- JSON serialization/deserialization
- Default values
"""

import json
from datetime import datetime

import pytest

from pytest_tracer_python.models import ScenarioMetadata, ScenariosFile


class TestScenarioMetadata:
    """Tests for ScenarioMetadata model."""

    def test_required_fields(self) -> None:
        """ScenarioMetadata requires id, file, function, description."""
        scenario = ScenarioMetadata(
            id="tests/test_auth.py::test_login",
            file="tests/test_auth.py",
            function="test_login",
            description="User logs in",
        )
        assert scenario.id == "tests/test_auth.py::test_login"
        assert scenario.file == "tests/test_auth.py"
        assert scenario.function == "test_login"
        assert scenario.description == "User logs in"

    def test_default_values(self) -> None:
        """ScenarioMetadata has sensible defaults."""
        scenario = ScenarioMetadata(
            id="test::id",
            file="test.py",
            function="test_func",
            description="Test",
        )
        assert scenario.documentation is None
        assert scenario.behaviors == []
        assert scenario.outcome == "success"

    def test_error_outcome(self) -> None:
        """ScenarioMetadata can have error outcome."""
        scenario = ScenarioMetadata(
            id="test::id",
            file="test.py",
            function="test_func",
            description="Test",
            outcome="error",
        )
        assert scenario.outcome == "error"

    def test_multiple_behaviors(self) -> None:
        """ScenarioMetadata can have multiple behaviors."""
        scenario = ScenarioMetadata(
            id="test::id",
            file="test.py",
            function="test_func",
            description="Test",
            behaviors=["auth", "session", "security"],
        )
        assert scenario.behaviors == ["auth", "session", "security"]

    def test_invalid_outcome_rejected(self) -> None:
        """ScenarioMetadata rejects invalid outcome values."""
        with pytest.raises(ValueError):
            ScenarioMetadata(
                id="test::id",
                file="test.py",
                function="test_func",
                description="Test",
                outcome="invalid",  # type: ignore
            )


class TestScenariosFile:
    """Tests for ScenariosFile model."""

    def test_default_version(self) -> None:
        """ScenariosFile has default version 1.0."""
        scenarios_file = ScenariosFile()
        assert scenarios_file.version == "1.0"

    def test_default_collected_at(self) -> None:
        """ScenariosFile has collected_at timestamp."""
        before = datetime.now()
        scenarios_file = ScenariosFile()
        after = datetime.now()

        assert before <= scenarios_file.collected_at <= after

    def test_empty_scenarios_list(self) -> None:
        """ScenariosFile defaults to empty scenarios list."""
        scenarios_file = ScenariosFile()
        assert scenarios_file.scenarios == []

    def test_with_scenarios(self) -> None:
        """ScenariosFile can contain scenarios."""
        scenario = ScenarioMetadata(
            id="test::id",
            file="test.py",
            function="test_func",
            description="Test",
        )
        scenarios_file = ScenariosFile(scenarios=[scenario])
        assert len(scenarios_file.scenarios) == 1
        assert scenarios_file.scenarios[0].id == "test::id"

    def test_json_roundtrip(self) -> None:
        """ScenariosFile can serialize and deserialize to/from JSON."""
        original = ScenariosFile(
            scenarios=[
                ScenarioMetadata(
                    id="tests/test_auth.py::test_login",
                    file="tests/test_auth.py",
                    function="test_login",
                    description="User logs in",
                    documentation="Full docstring here",
                    behaviors=["auth", "session"],
                    outcome="success",
                ),
                ScenarioMetadata(
                    id="tests/test_auth.py::test_login_fail",
                    file="tests/test_auth.py",
                    function="test_login_fail",
                    description="Login fails",
                    outcome="error",
                ),
            ]
        )

        # Serialize to JSON
        json_str = original.to_json()

        # Verify it's valid JSON
        parsed = json.loads(json_str)
        assert parsed["version"] == "1.0"
        assert len(parsed["scenarios"]) == 2

        # Deserialize back
        restored = ScenariosFile.from_json(json_str)
        assert restored.version == original.version
        assert len(restored.scenarios) == 2
        assert restored.scenarios[0].id == "tests/test_auth.py::test_login"
        assert restored.scenarios[0].behaviors == ["auth", "session"]
        assert restored.scenarios[1].outcome == "error"
