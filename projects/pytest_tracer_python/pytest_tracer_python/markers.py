"""
Pytest marker re-exports for scenario tests.

These markers are used to identify and annotate scenario tests:

- @pytest.mark.scenario: Marks a test as a scenario test
- @pytest.mark.behavior("name"): Tags a test with a behavior (can be used multiple times)
- @pytest.mark.error: Marks a test as an error/failure scenario

Example:
    @pytest.mark.scenario
    @pytest.mark.behavior("authentication")
    @pytest.mark.behavior("session-management")
    def test_successful_login():
        '''User logs in with valid credentials'''
        ...

    @pytest.mark.scenario
    @pytest.mark.behavior("authentication")
    @pytest.mark.error
    def test_login_invalid_password():
        '''Login fails with wrong password'''
        ...
"""

import pytest

# Re-export markers for convenient importing
# Usage: from pytest_tracer_python.markers import scenario, behavior, error

scenario = pytest.mark.scenario
"""Marks a test as a scenario test for trace analysis."""

behavior = pytest.mark.behavior
"""Tags a test with a behavior name. Can be used multiple times."""

error = pytest.mark.error
"""Marks a test as an error/failure scenario (expected outcome is failure)."""
