//! Integration tests for trace_analyzer.
//!
//! These tests use real fixture data from the Python project to verify
//! end-to-end parsing functionality.

use std::path::PathBuf;

use trace_analyzer::coverage::CoverageParser;
use trace_analyzer::models::ScenarioOutcome;
use trace_analyzer::scenarios::ScenarioParser;

/// Get the path to the Python project's test fixtures.
fn fixtures_dir() -> PathBuf {
    // Navigate from trace_analyzer to pytest_tracer_python fixtures
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("pytest_tracer_python")
        .join("tests")
        .join("fixtures")
}

/// Find the cache directory (named with content hash).
fn find_cache_dir() -> Option<PathBuf> {
    let cache_dir = fixtures_dir().join("cache");
    if !cache_dir.exists() {
        return None;
    }

    // Find the first subdirectory (the content hash directory)
    std::fs::read_dir(&cache_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .find(|e| e.path().is_dir())
        .map(|e| e.path())
}

mod coverage_parser {
    use super::*;

    #[test]
    fn test_open_coverage_database() {
        let Some(cache_dir) = find_cache_dir() else {
            eprintln!("Skipping test: cache directory not found (run Python tests first)");
            return;
        };

        let coverage_path = cache_dir.join(".coverage");
        let parser =
            CoverageParser::open(&coverage_path).expect("Failed to open coverage database");

        let metadata = parser.read_metadata().expect("Failed to read metadata");
        assert!(metadata.version.is_some(), "Expected coverage version");
    }

    #[test]
    fn test_read_files() {
        let Some(cache_dir) = find_cache_dir() else {
            eprintln!("Skipping test: cache directory not found");
            return;
        };

        let coverage_path = cache_dir.join(".coverage");
        let parser =
            CoverageParser::open(&coverage_path).expect("Failed to open coverage database");

        let files = parser.read_files().expect("Failed to read files");

        // Should have at least auth.py and orders.py
        assert!(!files.is_empty(), "Expected at least one file");

        let paths: Vec<&str> = files.values().map(|s| s.as_str()).collect();
        assert!(
            paths.iter().any(|p| p.contains("auth.py")),
            "Expected auth.py in files"
        );
        assert!(
            paths.iter().any(|p| p.contains("orders.py")),
            "Expected orders.py in files"
        );
    }

    #[test]
    fn test_read_contexts() {
        let Some(cache_dir) = find_cache_dir() else {
            eprintln!("Skipping test: cache directory not found");
            return;
        };

        let coverage_path = cache_dir.join(".coverage");
        let parser =
            CoverageParser::open(&coverage_path).expect("Failed to open coverage database");

        let contexts = parser.read_contexts().expect("Failed to read contexts");

        // Should have test contexts (not the empty global context)
        assert!(!contexts.is_empty(), "Expected at least one context");

        // Verify some expected test IDs are present
        let test_ids: Vec<&str> = contexts.values().map(|s| s.as_str()).collect();
        assert!(
            test_ids
                .iter()
                .any(|id| id.contains("test_successful_login")),
            "Expected test_successful_login context"
        );
    }

    #[test]
    fn test_read_coverage() {
        let Some(cache_dir) = find_cache_dir() else {
            eprintln!("Skipping test: cache directory not found");
            return;
        };

        let coverage_path = cache_dir.join(".coverage");
        let parser =
            CoverageParser::open(&coverage_path).expect("Failed to open coverage database");

        let coverage = parser.read_coverage().expect("Failed to read coverage");

        // Should have coverage for multiple tests
        assert!(!coverage.is_empty(), "Expected coverage data");

        // Find coverage for test_successful_login
        let login_coverage = coverage
            .iter()
            .find(|c| c.test_id.contains("test_successful_login"));

        assert!(
            login_coverage.is_some(),
            "Expected coverage for test_successful_login"
        );

        let login_coverage = login_coverage.unwrap();
        assert!(
            !login_coverage.files.is_empty(),
            "Expected files covered by test"
        );

        // Should cover auth.py
        let auth_coverage = login_coverage
            .files
            .iter()
            .find(|f| f.path.contains("auth.py"));
        assert!(auth_coverage.is_some(), "Expected auth.py to be covered");

        let auth_coverage = auth_coverage.unwrap();
        assert!(!auth_coverage.lines.is_empty(), "Expected lines covered");
    }
}

mod scenario_parser {
    use super::*;

    #[test]
    fn test_parse_scenarios_file() {
        let Some(cache_dir) = find_cache_dir() else {
            eprintln!("Skipping test: cache directory not found");
            return;
        };

        let scenarios_path = cache_dir.join("scenarios.json");
        let scenarios = ScenarioParser::parse(&scenarios_path).expect("Failed to parse scenarios");

        // Should have 10 scenarios (from sample_project)
        assert_eq!(scenarios.len(), 10, "Expected 10 scenarios");
    }

    #[test]
    fn test_scenario_fields() {
        let Some(cache_dir) = find_cache_dir() else {
            eprintln!("Skipping test: cache directory not found");
            return;
        };

        let scenarios_path = cache_dir.join("scenarios.json");
        let scenarios = ScenarioParser::parse(&scenarios_path).expect("Failed to parse scenarios");

        // Find test_successful_login scenario
        let login = scenarios
            .iter()
            .find(|s| s.function == "test_successful_login")
            .expect("Expected test_successful_login scenario");

        assert_eq!(login.file, "tests/test_auth.py");
        assert_eq!(login.description, "User logs in with valid credentials");
        assert!(login.documentation.is_some());
        assert!(login
            .documentation
            .as_ref()
            .unwrap()
            .contains("GIVEN a registered user"));
        assert!(login.behaviors.contains(&"authentication".to_string()));
        assert_eq!(login.outcome, ScenarioOutcome::Success);
    }

    #[test]
    fn test_error_scenarios() {
        let Some(cache_dir) = find_cache_dir() else {
            eprintln!("Skipping test: cache directory not found");
            return;
        };

        let scenarios_path = cache_dir.join("scenarios.json");
        let scenarios = ScenarioParser::parse(&scenarios_path).expect("Failed to parse scenarios");

        // Count error scenarios
        let error_count = scenarios
            .iter()
            .filter(|s| s.outcome == ScenarioOutcome::Error)
            .count();

        // Should have 5 error scenarios
        assert_eq!(error_count, 5, "Expected 5 error scenarios");
    }

    #[test]
    fn test_behaviors() {
        let Some(cache_dir) = find_cache_dir() else {
            eprintln!("Skipping test: cache directory not found");
            return;
        };

        let scenarios_path = cache_dir.join("scenarios.json");
        let scenarios = ScenarioParser::parse(&scenarios_path).expect("Failed to parse scenarios");

        // Collect all unique behaviors
        let mut behaviors: Vec<&str> = scenarios
            .iter()
            .flat_map(|s| s.behaviors.iter().map(|b| b.as_str()))
            .collect();
        behaviors.sort();
        behaviors.dedup();

        // Should have authentication, session-management, orders, etc.
        assert!(
            behaviors.contains(&"authentication"),
            "Expected authentication behavior"
        );
        assert!(
            behaviors.contains(&"session-management"),
            "Expected session-management behavior"
        );
        assert!(behaviors.contains(&"orders"), "Expected orders behavior");
    }
}

mod cli_integration {
    use std::process::Command;

    use super::*;

    #[test]
    fn test_build_command() {
        let Some(cache_dir) = find_cache_dir() else {
            eprintln!("Skipping test: cache directory not found");
            return;
        };

        let coverage_path = cache_dir.join(".coverage");
        let scenarios_path = cache_dir.join("scenarios.json");

        // Create a temp directory for the index
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let index_dir = temp_dir.path().join(".trace-index");

        // Run the trace build command
        let output = Command::new(env!("CARGO_BIN_EXE_trace"))
            .arg("build")
            .arg("--coverage")
            .arg(&coverage_path)
            .arg("--scenarios")
            .arg(&scenarios_path)
            .arg("--output")
            .arg(&index_dir)
            .output()
            .expect("Failed to run trace build");

        assert!(
            output.status.success(),
            "trace build should succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Parsed") && stdout.contains("test contexts"),
            "Should report parsed test contexts. Got: {}",
            stdout
        );
        assert!(
            stdout.contains("Parsed") && stdout.contains("scenarios"),
            "Should report parsed scenarios. Got: {}",
            stdout
        );
        assert!(
            stdout.contains("Built index"),
            "Should report index built. Got: {}",
            stdout
        );

        // Verify the index was created
        assert!(
            index_dir.join("index.db").exists(),
            "Index database should be created"
        );
    }

    #[test]
    fn test_help_command() {
        let output = Command::new(env!("CARGO_BIN_EXE_trace"))
            .arg("--help")
            .output()
            .expect("Failed to run trace --help");

        assert!(output.status.success());

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("build"),
            "Help should mention build command"
        );
        assert!(stdout.contains("list"), "Help should mention list command");
        assert!(
            stdout.contains("search"),
            "Help should mention search command"
        );
        assert!(
            stdout.contains("context"),
            "Help should mention context command"
        );
        assert!(
            stdout.contains("affected"),
            "Help should mention affected command"
        );
    }
}
