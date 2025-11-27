"""
Tests for content-addressable cache.

These tests verify:
- Content hash computation (using git tree hash)
- Cache hit/miss logic
- Cache file structure
"""

from pathlib import Path

from pytest_tracer_python.cache import compute_content_hash

# Path to sample_project fixture (committed to git)
FIXTURES_DIR = Path(__file__).parent / "fixtures"
SAMPLE_PROJECT = FIXTURES_DIR / "sample_project"


class TestContentHash:
    """Tests for compute_content_hash function.

    Since compute_content_hash uses git tree hash, we test with the
    sample_project directory which is committed to the repository.
    """

    def test_hash_is_deterministic(self) -> None:
        """Same content produces same hash."""
        hash1 = compute_content_hash(SAMPLE_PROJECT)
        hash2 = compute_content_hash(SAMPLE_PROJECT)

        assert hash1 == hash2

    def test_hash_length(self) -> None:
        """Hash is truncated to 16 characters."""
        content_hash = compute_content_hash(SAMPLE_PROJECT)

        assert len(content_hash) == 16
        # Should be valid hex
        assert all(c in "0123456789abcdef" for c in content_hash)

    def test_hash_is_git_tree_hash(self) -> None:
        """Hash comes from git tree object."""
        import subprocess

        content_hash = compute_content_hash(SAMPLE_PROJECT)

        # Verify it's a valid git object prefix
        # git cat-file -t should recognize it as a tree
        result = subprocess.run(
            ["git", "cat-file", "-t", content_hash],
            cwd=SAMPLE_PROJECT,
            capture_output=True,
            text=True,
        )
        # Should be a tree object (or at least a valid git object)
        assert result.returncode == 0
        assert result.stdout.strip() == "tree"

    def test_different_directories_have_different_hashes(self) -> None:
        """Different directories produce different hashes."""
        # Hash sample_project and its src subdirectory
        sample_hash = compute_content_hash(SAMPLE_PROJECT)
        src_hash = compute_content_hash(SAMPLE_PROJECT / "src")

        assert sample_hash != src_hash
