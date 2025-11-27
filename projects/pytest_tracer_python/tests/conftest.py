"""
Pytest configuration and fixtures for pytest-tracer tests.

This module provides the coverage cache fixture that enables fast,
repeatable testing of the scenario collector.
"""

from pathlib import Path

import pytest

from pytest_tracer_python.cache import CoverageCache, get_or_create_cache

# Paths relative to this file
FIXTURES_DIR = Path(__file__).parent / "fixtures"
SAMPLE_PROJECT_DIR = FIXTURES_DIR / "sample_project"
CACHE_DIR = FIXTURES_DIR / "cache"


@pytest.fixture(scope="session")
def sample_project_path() -> Path:
    """Path to the sample project fixture."""
    return SAMPLE_PROJECT_DIR


@pytest.fixture(scope="session")
def coverage_cache(sample_project_path: Path) -> CoverageCache:
    """
    Session-scoped fixture providing cached coverage data.

    On first run (cache miss):
    - Runs pytest with coverage on sample_project via subprocess
    - Stores results in cache/{content_hash}/
    - Takes ~2-5 seconds

    On subsequent runs (cache hit):
    - Loads existing cache instantly
    - Only regenerates if sample_project files change

    The cache is content-addressable: the directory name is a hash of
    all .py files in sample_project. This ensures the cache is always
    valid for the current state of the sample project.
    """
    return get_or_create_cache(
        sample_project=sample_project_path,
        cache_dir=CACHE_DIR,
        source_dir="src",
        test_dir="tests",
    )
