//! Core data models for trace analysis.
//!
//! These models represent the domain concepts independent of
//! storage format or serialization.

use serde::{Deserialize, Serialize};

/// A scenario test with its metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scenario {
    /// Unique identifier (pytest node ID), e.g., "tests/test_auth.py::test_login"
    pub id: String,

    /// Path to the test file relative to project root
    pub file: String,

    /// Test function name
    pub function: String,

    /// Short description (first line of docstring)
    pub description: String,

    /// Full documentation including GIVEN/WHEN/THEN sections
    #[serde(default)]
    pub documentation: Option<String>,

    /// Behavior tags from @pytest.mark.behavior markers
    #[serde(default)]
    pub behaviors: Vec<String>,

    /// Whether this is a success or error scenario
    #[serde(default = "default_outcome")]
    pub outcome: ScenarioOutcome,
}

fn default_outcome() -> ScenarioOutcome {
    ScenarioOutcome::Success
}

/// The expected outcome of a scenario test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ScenarioOutcome {
    #[default]
    Success,
    Error,
}

/// Coverage data for a single test context (scenario).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCoverage {
    /// The test context identifier (matches scenario id with "|run" suffix stripped)
    pub test_id: String,

    /// Files covered by this test, with their line numbers
    pub files: Vec<FileCoverage>,
}

/// Coverage data for a single file within a test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileCoverage {
    /// Absolute path to the source file
    pub path: String,

    /// Line numbers that were executed (sorted)
    pub lines: Vec<u32>,
}

/// Metadata about a coverage database.
#[derive(Debug, Clone)]
pub struct CoverageMetadata {
    /// Whether branch coverage was recorded
    pub has_arcs: bool,

    /// Version of coverage.py that created the database
    pub version: Option<String>,

    /// When the coverage was recorded
    pub when: Option<String>,
}
