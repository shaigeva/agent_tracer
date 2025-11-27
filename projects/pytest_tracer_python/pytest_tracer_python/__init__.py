"""
pytest-tracer: Test execution tracing for AI coding agents.

This package provides:
1. Pytest markers for annotating scenario tests
2. Scenario metadata collection
3. Coverage cache management for testing

Usage:
    from pytest_tracer_python.markers import scenario, behavior, error

    @scenario
    @behavior("authentication")
    def test_login():
        '''User logs in successfully'''
        ...
"""

from .markers import behavior, error, scenario
from .models import ScenarioMetadata, ScenariosFile

__all__ = [
    # Markers
    "scenario",
    "behavior",
    "error",
    # Models
    "ScenarioMetadata",
    "ScenariosFile",
]
