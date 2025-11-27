"""
Content-addressable cache for coverage data.

Design Rationale:
-----------------
Testing the scenario collector requires running pytest with coverage, but:
1. Running pytest-within-pytest causes issues (shared state, plugin conflicts)
2. Running subprocess for every test would be slow
3. We need reliable, repeatable test data

Solution: Content-addressable cache with subprocess generation

How it works:
1. Use git to compute tree hash of the sample project directory
2. If cache/{hash}/ exists → load cached data (instant)
3. If cache miss → run pytest --cov via subprocess (one-time cost)
4. All tests use cached data

Why git tree hash:
- Fast: git already indexes file contents
- Reliable: handles all edge cases (renames, permissions, etc.)
- Deterministic: same content always produces same hash

Why subprocess is acceptable:
- Runs only on cache miss (first run or after sample_project changes)
- Clean process isolation (no pytest-in-pytest issues)
- Simple implementation

Cache structure:
    tests/fixtures/
    ├── sample_project/           # Example tests + code (checked into git)
    │   ├── src/                  # Code being tested
    │   └── tests/                # Scenario tests with markers
    └── cache/                    # Gitignored - auto-generated
        └── {content_hash}/       # Git tree hash of sample_project
            ├── .coverage         # Cached coverage database
            └── scenarios.json    # Cached scenario metadata
"""

import os
import shutil
import subprocess
import sys
from pathlib import Path

from .models import ScenariosFile


def compute_content_hash(directory: Path) -> str:
    """
    Compute content hash using git's hashing of the working tree.

    Uses `git write-tree` with a temporary index to compute a tree hash
    of the current directory contents (including uncommitted changes).
    This is fast and reliable.

    Args:
        directory: Path to directory to hash

    Returns:
        First 16 characters of git tree hash
    """
    import tempfile

    directory = directory.resolve()

    # Create a path for temporary index (don't create the file - git needs it empty)
    tmp_dir = tempfile.gettempdir()
    tmp_index = Path(tmp_dir) / f"git-index-{os.getpid()}"

    # Ensure it doesn't exist (git will create it)
    tmp_index.unlink(missing_ok=True)

    try:
        env = {**os.environ, "GIT_INDEX_FILE": str(tmp_index)}

        # Add only files within this specific directory to the temporary index
        # Using "." with cwd ensures we only add files in this directory tree
        subprocess.run(
            ["git", "add", "."],
            cwd=directory,
            env=env,
            capture_output=True,
            text=True,
            check=True,
        )

        # Write the tree object and get its hash
        # Use --prefix="" to get tree relative to the directory
        result = subprocess.run(
            ["git", "write-tree", f"--prefix={_get_relative_git_path(directory)}/"],
            cwd=directory,
            env=env,
            capture_output=True,
            text=True,
            check=True,
        )

        tree_hash = result.stdout.strip()
        return tree_hash[:16]
    finally:
        # Clean up temporary index
        tmp_index.unlink(missing_ok=True)


def _get_relative_git_path(directory: Path) -> str:
    """Get directory path relative to git repo root."""
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        cwd=directory,
        capture_output=True,
        text=True,
        check=True,
    )
    repo_root = Path(result.stdout.strip())
    return str(directory.relative_to(repo_root))


class CoverageCache:
    """
    Manages cached coverage data for a sample project.

    Provides load/save operations and paths to cached files.
    """

    def __init__(self, cache_path: Path) -> None:
        """
        Initialize cache with path to cache directory.

        Args:
            cache_path: Path to cache directory (e.g., cache/{hash}/)
        """
        self.cache_path = cache_path

    @property
    def coverage_file(self) -> Path:
        """Path to .coverage SQLite database."""
        return self.cache_path / ".coverage"

    @property
    def scenarios_file(self) -> Path:
        """Path to scenarios.json metadata file."""
        return self.cache_path / "scenarios.json"

    def exists(self) -> bool:
        """Check if cache exists and has required files."""
        return self.coverage_file.exists() and self.scenarios_file.exists()

    def load_scenarios(self) -> ScenariosFile:
        """Load scenarios from cached JSON file."""
        return ScenariosFile.from_json(self.scenarios_file.read_text())

    def save_scenarios(self, scenarios: ScenariosFile) -> None:
        """Save scenarios to JSON file."""
        self.cache_path.mkdir(parents=True, exist_ok=True)
        self.scenarios_file.write_text(scenarios.to_json())


def generate_coverage_cache(
    sample_project: Path,
    cache: CoverageCache,
    source_dir: str = "src",
    test_dir: str = "tests",
) -> None:
    """
    Generate coverage cache by running pytest with coverage.

    This runs pytest in a subprocess to avoid pytest-in-pytest issues.
    The subprocess cost is acceptable because this only runs on cache miss.

    Args:
        sample_project: Path to sample project directory
        cache: CoverageCache to store results in
        source_dir: Subdirectory containing source code (default: "src")
        test_dir: Subdirectory containing tests (default: "tests")
    """
    cache.cache_path.mkdir(parents=True, exist_ok=True)

    src_path = sample_project / source_dir
    test_path = sample_project / test_dir

    # Set environment for isolated pytest run
    env = {
        **os.environ,
        # Control where coverage data is written
        "COVERAGE_FILE": str(cache.coverage_file),
        # Add sample_project to PYTHONPATH so imports work
        "PYTHONPATH": str(sample_project),
    }

    # Run pytest with coverage in subprocess
    # Use --rootdir to isolate from parent pytest.ini
    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "pytest",
            str(test_path),
            f"--cov={src_path}",
            "--cov-context=test",
            "-q",
            "--tb=short",
            # Isolate from parent project's pytest config
            f"--rootdir={sample_project}",
            "-o",
            "filterwarnings=",
        ],
        cwd=sample_project,
        env=env,
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        raise RuntimeError(f"pytest failed during cache generation:\nstdout: {result.stdout}\nstderr: {result.stderr}")

    # Collect scenario metadata
    # Import here to avoid circular imports
    from .collector import collect_scenarios

    scenarios = collect_scenarios(sample_project, test_dir)
    cache.save_scenarios(scenarios)


def get_or_create_cache(
    sample_project: Path,
    cache_dir: Path,
    source_dir: str = "src",
    test_dir: str = "tests",
    clean_old_caches: bool = True,
) -> CoverageCache:
    """
    Get cached coverage data, generating if needed.

    This is the main entry point for the caching system.
    On cache hit, returns instantly. On cache miss, runs pytest subprocess.

    Args:
        sample_project: Path to sample project directory
        cache_dir: Path to cache root directory
        source_dir: Subdirectory containing source code
        test_dir: Subdirectory containing tests
        clean_old_caches: If True, remove old cache directories on regeneration

    Returns:
        CoverageCache with coverage data
    """
    content_hash = compute_content_hash(sample_project)
    cache = CoverageCache(cache_dir / content_hash)

    if cache.exists():
        return cache

    # Cache miss - clean old caches if requested
    if clean_old_caches and cache_dir.exists():
        for old_cache in cache_dir.iterdir():
            if old_cache.is_dir() and old_cache.name != content_hash:
                shutil.rmtree(old_cache)

    # Generate new cache
    generate_coverage_cache(sample_project, cache, source_dir, test_dir)
    return cache
