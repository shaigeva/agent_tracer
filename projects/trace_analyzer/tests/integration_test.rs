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

    /// Helper to build an index and return the temp dir (keeps it alive).
    fn build_test_index() -> Option<(tempfile::TempDir, PathBuf)> {
        let cache_dir = find_cache_dir()?;
        let coverage_path = cache_dir.join(".coverage");
        let scenarios_path = cache_dir.join("scenarios.json");

        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let index_dir = temp_dir.path().join(".trace-index");

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

        if !output.status.success() {
            eprintln!(
                "Failed to build index: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return None;
        }

        Some((temp_dir, index_dir))
    }

    #[test]
    fn test_list_command() {
        let Some((_temp_dir, index_dir)) = build_test_index() else {
            eprintln!("Skipping test: could not build index");
            return;
        };

        let output = Command::new(env!("CARGO_BIN_EXE_trace"))
            .arg("list")
            .arg("--index")
            .arg(&index_dir)
            .output()
            .expect("Failed to run trace list");

        assert!(
            output.status.success(),
            "trace list should succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse as JSON
        let scenarios: serde_json::Value =
            serde_json::from_str(&stdout).expect("Output should be valid JSON");

        assert!(scenarios.is_array(), "Output should be a JSON array");
        let arr = scenarios.as_array().unwrap();
        assert_eq!(arr.len(), 10, "Should have 10 scenarios");
    }

    #[test]
    fn test_list_with_behavior_filter() {
        let Some((_temp_dir, index_dir)) = build_test_index() else {
            eprintln!("Skipping test: could not build index");
            return;
        };

        let output = Command::new(env!("CARGO_BIN_EXE_trace"))
            .arg("list")
            .arg("--behavior")
            .arg("authentication")
            .arg("--index")
            .arg(&index_dir)
            .output()
            .expect("Failed to run trace list");

        assert!(output.status.success());

        let stdout = String::from_utf8_lossy(&output.stdout);
        let scenarios: serde_json::Value = serde_json::from_str(&stdout).unwrap();

        let arr = scenarios.as_array().unwrap();
        assert!(!arr.is_empty(), "Should have some authentication scenarios");

        // All should have authentication behavior
        for scenario in arr {
            let behaviors = scenario["behaviors"].as_array().unwrap();
            let has_auth = behaviors
                .iter()
                .any(|b| b.as_str() == Some("authentication"));
            assert!(
                has_auth,
                "All scenarios should have authentication behavior"
            );
        }
    }

    #[test]
    fn test_list_errors_only() {
        let Some((_temp_dir, index_dir)) = build_test_index() else {
            eprintln!("Skipping test: could not build index");
            return;
        };

        let output = Command::new(env!("CARGO_BIN_EXE_trace"))
            .arg("list")
            .arg("--errors")
            .arg("--index")
            .arg(&index_dir)
            .output()
            .expect("Failed to run trace list");

        assert!(output.status.success());

        let stdout = String::from_utf8_lossy(&output.stdout);
        let scenarios: serde_json::Value = serde_json::from_str(&stdout).unwrap();

        let arr = scenarios.as_array().unwrap();
        assert_eq!(arr.len(), 5, "Should have 5 error scenarios");

        // All should have error outcome
        for scenario in arr {
            assert_eq!(
                scenario["outcome"].as_str(),
                Some("error"),
                "All should be error scenarios"
            );
        }
    }

    #[test]
    fn test_search_command() {
        let Some((_temp_dir, index_dir)) = build_test_index() else {
            eprintln!("Skipping test: could not build index");
            return;
        };

        let output = Command::new(env!("CARGO_BIN_EXE_trace"))
            .arg("search")
            .arg("login")
            .arg("--index")
            .arg(&index_dir)
            .output()
            .expect("Failed to run trace search");

        assert!(output.status.success());

        let stdout = String::from_utf8_lossy(&output.stdout);
        let scenarios: serde_json::Value = serde_json::from_str(&stdout).unwrap();

        let arr = scenarios.as_array().unwrap();
        assert!(!arr.is_empty(), "Should find scenarios matching 'login'");

        // All should contain login in description or documentation
        for scenario in arr {
            let desc = scenario["description"].as_str().unwrap_or("");
            let doc = scenario["documentation"].as_str().unwrap_or("");
            let matches =
                desc.to_lowercase().contains("login") || doc.to_lowercase().contains("login");
            assert!(matches, "Result should match search term");
        }
    }

    #[test]
    fn test_context_command() {
        let Some((_temp_dir, index_dir)) = build_test_index() else {
            eprintln!("Skipping test: could not build index");
            return;
        };

        // First get a scenario ID
        let list_output = Command::new(env!("CARGO_BIN_EXE_trace"))
            .arg("list")
            .arg("--index")
            .arg(&index_dir)
            .output()
            .expect("Failed to run trace list");

        let stdout = String::from_utf8_lossy(&list_output.stdout);
        let scenarios: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let scenario_id = scenarios[0]["id"].as_str().unwrap();

        // Get context for that scenario
        let output = Command::new(env!("CARGO_BIN_EXE_trace"))
            .arg("context")
            .arg(scenario_id)
            .arg("--index")
            .arg(&index_dir)
            .output()
            .expect("Failed to run trace context");

        assert!(
            output.status.success(),
            "trace context should succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let context: serde_json::Value = serde_json::from_str(&stdout).unwrap();

        assert!(context["scenario"].is_object(), "Should have scenario info");
        assert!(context["coverage"].is_array(), "Should have coverage array");

        assert_eq!(
            context["scenario"]["id"].as_str(),
            Some(scenario_id),
            "Scenario ID should match"
        );
    }

    #[test]
    fn test_affected_command() {
        let Some((_temp_dir, index_dir)) = build_test_index() else {
            eprintln!("Skipping test: could not build index");
            return;
        };

        // Search for scenarios affected by auth.py
        let output = Command::new(env!("CARGO_BIN_EXE_trace"))
            .arg("affected")
            .arg("auth.py")
            .arg("--index")
            .arg(&index_dir)
            .output()
            .expect("Failed to run trace affected");

        assert!(
            output.status.success(),
            "trace affected should succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let affected: serde_json::Value = serde_json::from_str(&stdout).unwrap();

        assert!(affected.is_array(), "Output should be a JSON array");
        let arr = affected.as_array().unwrap();
        assert!(!arr.is_empty(), "Should find scenarios affecting auth.py");

        // Each result should have scenario and matching_lines
        for item in arr {
            assert!(item["scenario"].is_object(), "Should have scenario info");
            assert!(
                item["matching_lines"].is_array(),
                "Should have matching_lines"
            );
        }
    }
}
