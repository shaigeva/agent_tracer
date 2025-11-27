# Minimal conftest for isolated sample project test runs
# This file ensures the sample project is treated as an independent pytest project

import pytest


def pytest_configure(config: pytest.Config) -> None:
    """Register custom markers used in scenario tests."""
    config.addinivalue_line("markers", "scenario: marks test as a scenario test")
    config.addinivalue_line("markers", "behavior(name): categorizes test by behavior")
    config.addinivalue_line("markers", "error: marks test as an error scenario")
