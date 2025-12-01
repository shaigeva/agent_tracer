"""
Scenario metadata collector using pytest's collection API.

This module extracts metadata from scenario tests by running pytest's
collection phase (--collect-only) in-process. This gives us accurate
marker and docstring information without running the actual tests.

Design Note:
-----------
We use pytest.main() with a custom plugin rather than AST parsing because:
1. Accurate marker resolution (handles inheritance, fixtures, etc.)
2. Proper parametrize expansion
3. Correct docstring extraction
4. pytest handles all edge cases

The cost is that we need pytest as a runtime dependency, but since this
tool is for pytest users, that's acceptable.
"""

import sys
from pathlib import Path

import pytest

from .models import ScenarioMetadata, ScenariosFile


def parse_docstring(docstring: str | None) -> tuple[str, str | None]:
    """
    Parse a docstring into description and full documentation.

    The description is the first non-empty line.
    The full documentation includes everything, preserving structure.

    Args:
        docstring: Raw docstring from function

    Returns:
        Tuple of (description, full_documentation)
        description: First line (never None, defaults to "")
        full_documentation: Full docstring or None if empty
    """
    if not docstring:
        return "", None

    # Strip and normalize
    docstring = docstring.strip()
    if not docstring:
        return "", None

    # First non-empty line is description
    lines = docstring.split("\n")
    description = ""
    for line in lines:
        stripped = line.strip()
        if stripped:
            description = stripped
            break

    return description, docstring


def extract_behaviors(item: pytest.Item) -> list[str]:
    """
    Extract behavior tags from pytest item markers.

    Args:
        item: pytest.Item object

    Returns:
        List of behavior names from @pytest.mark.behavior("name") markers
    """
    behaviors = []
    for marker in item.iter_markers("behavior"):
        if marker.args:
            behaviors.append(marker.args[0])
    return behaviors


def extract_scenario_from_item(item: pytest.Item, project_root: Path) -> ScenarioMetadata | None:
    """
    Extract scenario metadata from a pytest Item.

    This is the core extraction logic, separated for testability.

    Args:
        item: pytest.Item (test function)
        project_root: Root path for computing relative file paths

    Returns:
        ScenarioMetadata if item has @scenario marker, None otherwise
    """
    # Only process items with @scenario marker
    if not item.get_closest_marker("scenario"):
        return None

    # Get file path relative to project root
    file_path = Path(item.fspath)
    try:
        relative_file = str(file_path.relative_to(project_root))
    except ValueError:
        # File is outside project root, use absolute
        relative_file = str(file_path)

    # Extract docstring
    docstring = item.function.__doc__ if hasattr(item, "function") else None
    description, documentation = parse_docstring(docstring)

    # Extract behaviors
    behaviors = extract_behaviors(item)

    # Determine outcome (error marker = error, else success)
    is_error = item.get_closest_marker("error") is not None
    outcome = "error" if is_error else "success"

    return ScenarioMetadata(
        id=item.nodeid,
        file=relative_file,
        function=item.name,
        description=description,
        documentation=documentation,
        behaviors=behaviors,
        outcome=outcome,
    )


class ScenarioCollectorPlugin:
    """
    Pytest plugin that collects scenario metadata during test collection.

    This plugin hooks into pytest's collection phase to extract metadata
    from tests marked with @pytest.mark.scenario.
    """

    def __init__(self, project_root: Path) -> None:
        """
        Initialize collector.

        Args:
            project_root: Root path for computing relative file paths
        """
        self.project_root = project_root
        self.scenarios: list[ScenarioMetadata] = []

    def pytest_collection_finish(self, session: pytest.Session) -> None:
        """
        Called after collection is complete.

        Iterates through all collected items and extracts scenario metadata.
        """
        for item in session.items:
            scenario = extract_scenario_from_item(item, self.project_root)
            if scenario:
                self.scenarios.append(scenario)


def collect_scenarios(
    project_root: Path,
    test_dir: str = "tests",
    quiet: bool = True,
) -> ScenariosFile:
    """
    Collect scenario metadata from a project's tests.

    Runs pytest in collection-only mode to discover all tests, then
    extracts metadata from tests marked with @pytest.mark.scenario.

    Args:
        project_root: Path to project root directory
        test_dir: Subdirectory containing tests (default: "tests")
        quiet: If True, suppress pytest's stdout/stderr output (default: True)

    Returns:
        ScenariosFile containing all collected scenarios
    """
    import io
    from contextlib import redirect_stderr, redirect_stdout

    test_path = project_root / test_dir
    plugin = ScenarioCollectorPlugin(project_root)

    # Add project root to sys.path so imports work
    project_root_str = str(project_root)
    path_added = False
    if project_root_str not in sys.path:
        sys.path.insert(0, project_root_str)
        path_added = True

    try:
        # Run pytest in collect-only mode with our plugin
        # Use --rootdir to isolate from parent project's pytest config
        # Note: Don't use "-p no:terminal" as it removes the -q/-v options
        args = [
            str(test_path),
            "--collect-only",
            "-q",
            f"--rootdir={project_root}",
            # Don't inherit any filterwarnings settings from parent project
            "-o",
            "filterwarnings=",
        ]

        if quiet:
            # Suppress pytest's output when running as a library
            with redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
                pytest.main(args, plugins=[plugin])
        else:
            pytest.main(args, plugins=[plugin])
    finally:
        # Clean up sys.path
        if path_added:
            sys.path.remove(project_root_str)

    return ScenariosFile(scenarios=plugin.scenarios)
