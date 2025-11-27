"""
Tests for scenario collector.

These tests use the cached coverage fixture to verify scenario extraction.
The cache is generated once (on first run or when sample_project changes)
and reused for all subsequent test runs.
"""

from pytest_tracer_python.cache import CoverageCache
from pytest_tracer_python.collector import parse_docstring


class TestParseDocstring:
    """Tests for docstring parsing."""

    def test_empty_docstring(self) -> None:
        """Empty docstring returns empty description."""
        description, documentation = parse_docstring(None)
        assert description == ""
        assert documentation is None

    def test_whitespace_only_docstring(self) -> None:
        """Whitespace-only docstring returns empty description."""
        description, documentation = parse_docstring("   \n   \n   ")
        assert description == ""
        assert documentation is None

    def test_single_line_docstring(self) -> None:
        """Single line docstring becomes both description and documentation."""
        description, documentation = parse_docstring("User logs in successfully")
        assert description == "User logs in successfully"
        assert documentation == "User logs in successfully"

    def test_multiline_docstring(self) -> None:
        """First line is description, full text is documentation."""
        docstring = """User logs in with valid credentials

        GIVEN a registered user
        WHEN they submit valid credentials
        THEN they receive an auth token
        """
        description, documentation = parse_docstring(docstring)
        assert description == "User logs in with valid credentials"
        assert documentation is not None
        assert "GIVEN" in documentation
        assert "WHEN" in documentation
        assert "THEN" in documentation

    def test_leading_whitespace_stripped(self) -> None:
        """Leading whitespace is stripped from description."""
        docstring = """

        Description after blank lines
        """
        description, documentation = parse_docstring(docstring)
        assert description == "Description after blank lines"


class TestCollectorIntegration:
    """
    Integration tests for scenario collector.

    These tests use the coverage_cache fixture which:
    - On first run: generates cache via subprocess (~2-5s)
    - On subsequent runs: loads from cache (instant)
    """

    def test_finds_all_scenario_tests(self, coverage_cache: CoverageCache) -> None:
        """Collector finds all tests marked with @scenario."""
        scenarios = coverage_cache.load_scenarios()

        # We have 10 scenario tests in sample_project
        # (6 in test_auth.py + 4 in test_orders.py)
        # Note: test_regular_helper_function is NOT a scenario
        assert len(scenarios.scenarios) == 10

    def test_scenario_ids_are_correct(self, coverage_cache: CoverageCache) -> None:
        """Scenario IDs follow pytest node ID format."""
        scenarios = coverage_cache.load_scenarios()

        ids = [s.id for s in scenarios.scenarios]
        assert any("test_successful_login" in id for id in ids)
        assert any("test_create_order" in id for id in ids)

    def test_extracts_single_behavior(self, coverage_cache: CoverageCache) -> None:
        """Collector extracts single behavior marker."""
        scenarios = coverage_cache.load_scenarios()

        # Find test_logout - has only session-management behavior
        logout_scenario = next((s for s in scenarios.scenarios if "test_logout" in s.id), None)
        assert logout_scenario is not None
        assert logout_scenario.behaviors == ["session-management"]

    def test_extracts_multiple_behaviors(self, coverage_cache: CoverageCache) -> None:
        """Collector extracts multiple behavior markers."""
        scenarios = coverage_cache.load_scenarios()

        # Find test_successful_login - has authentication and session-management
        login_scenario = next((s for s in scenarios.scenarios if "test_successful_login" in s.id), None)
        assert login_scenario is not None
        assert "authentication" in login_scenario.behaviors
        assert "session-management" in login_scenario.behaviors

    def test_identifies_error_scenarios(self, coverage_cache: CoverageCache) -> None:
        """Collector identifies tests with @error marker."""
        scenarios = coverage_cache.load_scenarios()

        # Find error scenarios
        error_scenarios = [s for s in scenarios.scenarios if s.outcome == "error"]

        # We have 5 error scenarios in sample_project
        assert len(error_scenarios) == 5

        # Verify specific error scenario
        login_fail = next((s for s in error_scenarios if "test_login_invalid_password" in s.id), None)
        assert login_fail is not None
        assert login_fail.outcome == "error"

    def test_identifies_success_scenarios(self, coverage_cache: CoverageCache) -> None:
        """Collector marks non-error tests as success."""
        scenarios = coverage_cache.load_scenarios()

        success_scenarios = [s for s in scenarios.scenarios if s.outcome == "success"]

        # We have 5 success scenarios
        assert len(success_scenarios) == 5

    def test_extracts_description_from_docstring(self, coverage_cache: CoverageCache) -> None:
        """Collector extracts first line of docstring as description."""
        scenarios = coverage_cache.load_scenarios()

        login_scenario = next((s for s in scenarios.scenarios if "test_successful_login" in s.id), None)
        assert login_scenario is not None
        assert login_scenario.description == "User logs in with valid credentials"

    def test_extracts_full_documentation(self, coverage_cache: CoverageCache) -> None:
        """Collector preserves full docstring including GIVEN/WHEN/THEN."""
        scenarios = coverage_cache.load_scenarios()

        login_scenario = next((s for s in scenarios.scenarios if "test_successful_login" in s.id), None)
        assert login_scenario is not None
        assert login_scenario.documentation is not None
        assert "GIVEN" in login_scenario.documentation
        assert "WHEN" in login_scenario.documentation
        assert "THEN" in login_scenario.documentation

    def test_handles_minimal_docstring(self, coverage_cache: CoverageCache) -> None:
        """Collector handles single-line docstrings."""
        scenarios = coverage_cache.load_scenarios()

        # test_login_invalid_password has single-line docstring
        fail_scenario = next(
            (s for s in scenarios.scenarios if "test_login_invalid_password" in s.id),
            None,
        )
        assert fail_scenario is not None
        assert fail_scenario.description == "Login fails with wrong password"

    def test_excludes_non_scenario_tests(self, coverage_cache: CoverageCache) -> None:
        """Collector excludes tests without @scenario marker."""
        scenarios = coverage_cache.load_scenarios()

        ids = [s.id for s in scenarios.scenarios]
        # test_regular_helper_function should NOT be collected
        assert not any("test_regular_helper_function" in id for id in ids)

    def test_json_output_is_valid(self, coverage_cache: CoverageCache) -> None:
        """Collector produces valid JSON output."""
        scenarios = coverage_cache.load_scenarios()

        json_str = scenarios.to_json()

        # Verify it can be parsed back
        restored = scenarios.from_json(json_str)
        assert len(restored.scenarios) == len(scenarios.scenarios)
        assert restored.version == "1.0"
