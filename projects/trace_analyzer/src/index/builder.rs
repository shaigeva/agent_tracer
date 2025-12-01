//! Index builder - combines coverage data and scenario metadata into a queryable index.

use std::collections::HashMap;
use std::path::Path;

use crate::coverage::CoverageParser;
use crate::error::CoverageError;
use crate::models::{Scenario, ScenarioOutcome, TestCoverage};
use crate::scenarios::ScenarioParser;

use super::schema::{Index, IndexError};

/// Result of building an index.
#[derive(Debug)]
pub struct BuildResult {
    /// Number of scenarios imported.
    pub scenarios_imported: usize,
    /// Number of scenarios with coverage data.
    pub scenarios_with_coverage: usize,
    /// Number of coverage entries (file/line pairs) imported.
    pub coverage_entries: usize,
    /// Scenarios that were in metadata but not in coverage.
    pub scenarios_without_coverage: Vec<String>,
    /// Test contexts in coverage that didn't match any scenario.
    pub unmatched_contexts: Vec<String>,
}

/// Builder for creating the index from coverage and scenario data.
pub struct IndexBuilder {
    scenarios: Vec<Scenario>,
    coverage: Vec<TestCoverage>,
}

impl IndexBuilder {
    /// Create a new builder by loading data from coverage and scenario files.
    pub fn load(coverage_path: &Path, scenarios_path: &Path) -> Result<Self, BuildError> {
        // Parse coverage data
        let coverage_parser = CoverageParser::open(coverage_path)?;
        let coverage = coverage_parser.read_coverage()?;

        // Parse scenarios
        let scenarios = ScenarioParser::parse(scenarios_path)?;

        Ok(Self {
            scenarios,
            coverage,
        })
    }

    /// Create a builder from pre-loaded data (useful for testing).
    pub fn from_data(scenarios: Vec<Scenario>, coverage: Vec<TestCoverage>) -> Self {
        Self {
            scenarios,
            coverage,
        }
    }

    /// Build the index at the given location.
    pub fn build(self, index_dir: &Path) -> Result<BuildResult, BuildError> {
        // Create or recreate the index
        let index = Index::create(index_dir)?;
        index.clear()?;

        // Build a map from test_id to coverage for quick lookup
        let coverage_map: HashMap<&str, &TestCoverage> = self
            .coverage
            .iter()
            .map(|c| (c.test_id.as_str(), c))
            .collect();

        let mut result = BuildResult {
            scenarios_imported: 0,
            scenarios_with_coverage: 0,
            coverage_entries: 0,
            scenarios_without_coverage: Vec::new(),
            unmatched_contexts: Vec::new(),
        };

        // Insert scenarios and their coverage
        let conn = index.connection();

        for scenario in &self.scenarios {
            // Insert scenario
            conn.execute(
                "INSERT INTO scenarios (id, file, function, description, documentation, outcome)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (
                    &scenario.id,
                    &scenario.file,
                    &scenario.function,
                    &scenario.description,
                    &scenario.documentation,
                    outcome_to_str(scenario.outcome),
                ),
            )?;

            // Insert behaviors
            for behavior in &scenario.behaviors {
                conn.execute(
                    "INSERT INTO scenario_behaviors (scenario_id, behavior)
                     VALUES (?1, ?2)",
                    (&scenario.id, behavior),
                )?;
            }

            result.scenarios_imported += 1;

            // Find and insert coverage data
            if let Some(test_coverage) = coverage_map.get(scenario.id.as_str()) {
                result.scenarios_with_coverage += 1;

                for file_cov in &test_coverage.files {
                    for &line in &file_cov.lines {
                        conn.execute(
                            "INSERT OR IGNORE INTO coverage (scenario_id, file_path, line_number)
                             VALUES (?1, ?2, ?3)",
                            (&scenario.id, &file_cov.path, line),
                        )?;
                        result.coverage_entries += 1;
                    }
                }
            } else {
                result.scenarios_without_coverage.push(scenario.id.clone());
            }
        }

        // Find unmatched coverage contexts
        let scenario_ids: std::collections::HashSet<&str> =
            self.scenarios.iter().map(|s| s.id.as_str()).collect();

        for test_coverage in &self.coverage {
            if !scenario_ids.contains(test_coverage.test_id.as_str()) {
                result
                    .unmatched_contexts
                    .push(test_coverage.test_id.clone());
            }
        }

        Ok(result)
    }
}

fn outcome_to_str(outcome: ScenarioOutcome) -> &'static str {
    match outcome {
        ScenarioOutcome::Success => "success",
        ScenarioOutcome::Error => "error",
    }
}

/// Errors that can occur during index building.
#[derive(Debug)]
pub enum BuildError {
    Coverage(CoverageError),
    Scenario(crate::error::ScenarioError),
    Index(IndexError),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::Coverage(e) => write!(f, "Coverage error: {}", e),
            BuildError::Scenario(e) => write!(f, "Scenario error: {}", e),
            BuildError::Index(e) => write!(f, "Index error: {}", e),
        }
    }
}

impl std::error::Error for BuildError {}

impl From<CoverageError> for BuildError {
    fn from(e: CoverageError) -> Self {
        BuildError::Coverage(e)
    }
}

impl From<crate::error::ScenarioError> for BuildError {
    fn from(e: crate::error::ScenarioError) -> Self {
        BuildError::Scenario(e)
    }
}

impl From<IndexError> for BuildError {
    fn from(e: IndexError) -> Self {
        BuildError::Index(e)
    }
}

impl From<rusqlite::Error> for BuildError {
    fn from(e: rusqlite::Error) -> Self {
        BuildError::Index(IndexError::Database(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FileCoverage;
    use tempfile::TempDir;

    fn make_scenario(id: &str, behaviors: Vec<&str>) -> Scenario {
        Scenario {
            id: id.to_string(),
            file: format!("tests/{}.py", id.split("::").next().unwrap_or("test")),
            function: id.split("::").last().unwrap_or("test").to_string(),
            description: format!("Test {}", id),
            documentation: None,
            behaviors: behaviors.into_iter().map(String::from).collect(),
            outcome: ScenarioOutcome::Success,
        }
    }

    fn make_coverage(test_id: &str, files: Vec<(&str, Vec<u32>)>) -> TestCoverage {
        TestCoverage {
            test_id: test_id.to_string(),
            files: files
                .into_iter()
                .map(|(path, lines)| FileCoverage {
                    path: path.to_string(),
                    lines,
                })
                .collect(),
        }
    }

    #[test]
    fn test_build_empty_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        let builder = IndexBuilder::from_data(vec![], vec![]);
        let result = builder.build(&index_dir).unwrap();

        assert_eq!(result.scenarios_imported, 0);
        assert_eq!(result.coverage_entries, 0);
    }

    #[test]
    fn test_build_with_scenarios() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        let scenarios = vec![
            make_scenario("tests/test_auth.py::test_login", vec!["auth"]),
            make_scenario("tests/test_auth.py::test_logout", vec!["auth", "session"]),
        ];

        let coverage = vec![
            make_coverage(
                "tests/test_auth.py::test_login",
                vec![("src/auth.py", vec![10, 11, 12])],
            ),
            make_coverage(
                "tests/test_auth.py::test_logout",
                vec![("src/auth.py", vec![20, 21]), ("src/session.py", vec![5])],
            ),
        ];

        let builder = IndexBuilder::from_data(scenarios, coverage);
        let result = builder.build(&index_dir).unwrap();

        assert_eq!(result.scenarios_imported, 2);
        assert_eq!(result.scenarios_with_coverage, 2);
        assert_eq!(result.coverage_entries, 6); // 3 + 2 + 1
        assert!(result.scenarios_without_coverage.is_empty());
        assert!(result.unmatched_contexts.is_empty());
    }

    #[test]
    fn test_build_with_missing_coverage() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        let scenarios = vec![
            make_scenario("tests/test_auth.py::test_login", vec!["auth"]),
            make_scenario("tests/test_auth.py::test_logout", vec!["auth"]),
        ];

        // Only coverage for one test
        let coverage = vec![make_coverage(
            "tests/test_auth.py::test_login",
            vec![("src/auth.py", vec![10, 11])],
        )];

        let builder = IndexBuilder::from_data(scenarios, coverage);
        let result = builder.build(&index_dir).unwrap();

        assert_eq!(result.scenarios_imported, 2);
        assert_eq!(result.scenarios_with_coverage, 1);
        assert_eq!(
            result.scenarios_without_coverage,
            vec!["tests/test_auth.py::test_logout"]
        );
    }

    #[test]
    fn test_build_with_unmatched_coverage() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        let scenarios = vec![make_scenario(
            "tests/test_auth.py::test_login",
            vec!["auth"],
        )];

        // Coverage includes tests not in scenarios
        let coverage = vec![
            make_coverage(
                "tests/test_auth.py::test_login",
                vec![("src/auth.py", vec![10])],
            ),
            make_coverage(
                "tests/test_other.py::test_something",
                vec![("src/other.py", vec![1, 2, 3])],
            ),
        ];

        let builder = IndexBuilder::from_data(scenarios, coverage);
        let result = builder.build(&index_dir).unwrap();

        assert_eq!(result.scenarios_imported, 1);
        assert_eq!(
            result.unmatched_contexts,
            vec!["tests/test_other.py::test_something"]
        );
    }

    #[test]
    fn test_build_preserves_behaviors() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        let scenarios = vec![make_scenario(
            "tests/test.py::test_foo",
            vec!["auth", "session", "api"],
        )];

        let builder = IndexBuilder::from_data(scenarios, vec![]);
        builder.build(&index_dir).unwrap();

        // Open and verify behaviors
        let index = Index::open(&index_dir).unwrap();
        let behaviors: Vec<String> = index
            .connection()
            .prepare("SELECT behavior FROM scenario_behaviors WHERE scenario_id = ?")
            .unwrap()
            .query_map(["tests/test.py::test_foo"], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(behaviors.len(), 3);
        assert!(behaviors.contains(&"auth".to_string()));
        assert!(behaviors.contains(&"session".to_string()));
        assert!(behaviors.contains(&"api".to_string()));
    }
}
