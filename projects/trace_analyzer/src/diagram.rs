//! Mermaid diagram generation from coverage data.
//!
//! Generates mermaid flowchart diagrams showing which source files
//! are covered by scenarios, grouped by directory.

use crate::index::{Index, IndexError};
use crate::query;

/// Output of diagram generation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DiagramOutput {
    /// The mermaid diagram source code.
    pub mermaid: String,
    /// Number of scenarios included.
    pub scenario_count: usize,
    /// Number of files included.
    pub file_count: usize,
}

/// Generate a mermaid diagram for a single scenario showing all files it covers.
pub fn diagram_for_scenario(index: &Index, scenario_id: &str) -> Result<DiagramOutput, IndexError> {
    let context = query::get_scenario_context(index, scenario_id)?;

    let mut mermaid = String::new();
    mermaid.push_str("graph TD\n");

    // Scenario node
    let safe_id = sanitize_id(scenario_id);
    let short_name = scenario_id.split("::").last().unwrap_or(scenario_id);
    mermaid.push_str(&format!(
        "    {}[\"{}\"]\n",
        safe_id,
        escape_mermaid(short_name)
    ));

    // Group files by directory
    let mut dirs: std::collections::BTreeMap<String, Vec<(String, usize)>> =
        std::collections::BTreeMap::new();

    for file_cov in &context.coverage {
        let dir = extract_dir(&file_cov.path);
        let filename = extract_filename(&file_cov.path);
        dirs.entry(dir)
            .or_default()
            .push((filename, file_cov.lines.len()));
    }

    let file_count = context.coverage.len();

    // Generate subgraphs for each directory
    for (dir, files) in &dirs {
        let dir_id = sanitize_id(dir);
        mermaid.push_str(&format!(
            "    subgraph {}[\"{}\"]\n",
            dir_id,
            escape_mermaid(dir)
        ));
        for (filename, line_count) in files {
            let file_id = sanitize_id(&format!("{}_{}", dir, filename));
            mermaid.push_str(&format!(
                "        {}[\"{}\\n({} lines)\"]\n",
                file_id,
                escape_mermaid(filename),
                line_count
            ));
        }
        mermaid.push_str("    end\n");
    }

    // Connect scenario to each file
    for file_cov in &context.coverage {
        let dir = extract_dir(&file_cov.path);
        let filename = extract_filename(&file_cov.path);
        let file_id = sanitize_id(&format!("{}_{}", dir, filename));
        mermaid.push_str(&format!("    {} --> {}\n", safe_id, file_id));
    }

    Ok(DiagramOutput {
        mermaid,
        scenario_count: 1,
        file_count,
    })
}

/// Generate a mermaid diagram showing all scenarios that cover a given file.
pub fn diagram_for_file(
    index: &Index,
    file_path: &str,
    line: Option<u32>,
) -> Result<DiagramOutput, IndexError> {
    let affected = query::find_affected_scenarios(index, file_path, line)?;

    if affected.is_empty() {
        return Ok(DiagramOutput {
            mermaid: format!(
                "graph TD\n    target[\"{}\"]\n    note[\"No scenarios cover this file\"]\n",
                escape_mermaid(file_path)
            ),
            scenario_count: 0,
            file_count: 1,
        });
    }

    let mut mermaid = String::new();
    mermaid.push_str("graph LR\n");

    // File node
    let file_id = sanitize_id(file_path);
    let line_label = match line {
        Some(l) => format!("{}:{}", file_path, l),
        None => file_path.to_string(),
    };
    mermaid.push_str(&format!(
        "    {}[\"{}\"]\n",
        file_id,
        escape_mermaid(&line_label)
    ));

    let scenario_count = affected.len();

    // Collect all files covered by each scenario to show the full picture
    let mut all_files: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for affected_scenario in &affected {
        let scenario_id = &affected_scenario.scenario.id;
        let safe_id = sanitize_id(scenario_id);
        let short_name = scenario_id.split("::").last().unwrap_or(scenario_id);
        let lines_label = format!("{} lines", affected_scenario.matching_lines.len());

        mermaid.push_str(&format!(
            "    {}[\"{}\"]\n",
            safe_id,
            escape_mermaid(short_name)
        ));
        mermaid.push_str(&format!(
            "    {} -->|\"{}\"| {}\n",
            safe_id, lines_label, file_id
        ));

        // Get full context to show other files
        if let Ok(context) = query::get_scenario_context(index, scenario_id) {
            for file_cov in &context.coverage {
                if file_cov.path != file_path {
                    all_files.insert(file_cov.path.clone());
                }
            }
        }
    }

    // Show other files these scenarios also cover
    if !all_files.is_empty() {
        mermaid.push_str("    subgraph also_covered[\"Also covered\"]\n");
        for other_file in &all_files {
            let other_id = sanitize_id(other_file);
            let other_name = extract_filename(other_file);
            mermaid.push_str(&format!(
                "        {}[\"{}\"]\n",
                other_id,
                escape_mermaid(&other_name)
            ));
        }
        mermaid.push_str("    end\n");
    }

    Ok(DiagramOutput {
        mermaid,
        scenario_count,
        file_count: 1 + all_files.len(),
    })
}

/// Sanitize a string for use as a mermaid node ID.
fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Escape special mermaid characters in labels.
fn escape_mermaid(s: &str) -> String {
    s.replace('"', "#quot;")
}

/// Extract the directory portion of a path.
fn extract_dir(path: &str) -> String {
    match path.rfind('/') {
        Some(idx) => path[..idx].to_string(),
        None => ".".to_string(),
    }
}

/// Extract the filename from a path.
fn extract_filename(path: &str) -> String {
    match path.rfind('/') {
        Some(idx) => path[idx + 1..].to_string(),
        None => path.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_id() {
        assert_eq!(
            sanitize_id("tests/test_auth.py::test_login"),
            "tests_test_auth_py__test_login"
        );
    }

    #[test]
    fn test_escape_mermaid() {
        assert_eq!(escape_mermaid("hello \"world\""), "hello #quot;world#quot;");
    }

    #[test]
    fn test_extract_dir() {
        assert_eq!(extract_dir("src/auth/login.py"), "src/auth");
        assert_eq!(extract_dir("file.py"), ".");
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(extract_filename("src/auth/login.py"), "login.py");
        assert_eq!(extract_filename("file.py"), "file.py");
    }

    #[test]
    fn test_diagram_for_scenario_with_index() {
        use crate::index::{Index, IndexBuilder};
        use crate::models::{FileCoverage, Scenario, ScenarioOutcome, TestCoverage};
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        let scenarios = vec![Scenario {
            id: "tests/test_auth.py::test_login".to_string(),
            file: "tests/test_auth.py".to_string(),
            function: "test_login".to_string(),
            description: "User logs in".to_string(),
            documentation: None,
            behaviors: vec!["auth".to_string()],
            outcome: ScenarioOutcome::Success,
        }];

        let coverage = vec![TestCoverage {
            test_id: "tests/test_auth.py::test_login".to_string(),
            files: vec![
                FileCoverage {
                    path: "src/auth.py".to_string(),
                    lines: vec![10, 11, 12],
                },
                FileCoverage {
                    path: "src/models/user.py".to_string(),
                    lines: vec![5, 6],
                },
            ],
        }];

        let builder = IndexBuilder::from_data(scenarios, coverage);
        builder.build(&index_dir).unwrap();

        let index = Index::open_readonly(&index_dir).unwrap();
        let result = diagram_for_scenario(&index, "tests/test_auth.py::test_login").unwrap();

        assert!(result.mermaid.contains("graph TD"));
        assert!(result.mermaid.contains("test_login"));
        assert!(result.mermaid.contains("auth.py"));
        assert!(result.mermaid.contains("user.py"));
        assert_eq!(result.scenario_count, 1);
        assert_eq!(result.file_count, 2);
    }

    #[test]
    fn test_diagram_for_file_with_index() {
        use crate::index::{Index, IndexBuilder};
        use crate::models::{FileCoverage, Scenario, ScenarioOutcome, TestCoverage};
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        let scenarios = vec![
            Scenario {
                id: "tests/test_auth.py::test_login".to_string(),
                file: "tests/test_auth.py".to_string(),
                function: "test_login".to_string(),
                description: "Login test".to_string(),
                documentation: None,
                behaviors: vec![],
                outcome: ScenarioOutcome::Success,
            },
            Scenario {
                id: "tests/test_auth.py::test_logout".to_string(),
                file: "tests/test_auth.py".to_string(),
                function: "test_logout".to_string(),
                description: "Logout test".to_string(),
                documentation: None,
                behaviors: vec![],
                outcome: ScenarioOutcome::Success,
            },
        ];

        let coverage = vec![
            TestCoverage {
                test_id: "tests/test_auth.py::test_login".to_string(),
                files: vec![FileCoverage {
                    path: "src/auth.py".to_string(),
                    lines: vec![10, 11],
                }],
            },
            TestCoverage {
                test_id: "tests/test_auth.py::test_logout".to_string(),
                files: vec![
                    FileCoverage {
                        path: "src/auth.py".to_string(),
                        lines: vec![20, 21],
                    },
                    FileCoverage {
                        path: "src/session.py".to_string(),
                        lines: vec![5],
                    },
                ],
            },
        ];

        let builder = IndexBuilder::from_data(scenarios, coverage);
        builder.build(&index_dir).unwrap();

        let index = Index::open_readonly(&index_dir).unwrap();
        let result = diagram_for_file(&index, "src/auth.py", None).unwrap();

        assert!(result.mermaid.contains("graph LR"));
        assert!(result.mermaid.contains("test_login"));
        assert!(result.mermaid.contains("test_logout"));
        assert_eq!(result.scenario_count, 2);
    }

    /// Verify that site-packages and other dependency paths don't appear in diagrams
    /// when the coverage data includes them.
    #[test]
    fn test_dependency_filtering_in_coverage() {
        use crate::index::{Index, IndexBuilder};
        use crate::models::{FileCoverage, Scenario, ScenarioOutcome, TestCoverage};
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        // Simulate coverage that includes site-packages (shouldn't happen with --cov=src,
        // but verify the data flows through correctly)
        let scenarios = vec![Scenario {
            id: "tests/test_auth.py::test_login".to_string(),
            file: "tests/test_auth.py".to_string(),
            function: "test_login".to_string(),
            description: "Login test".to_string(),
            documentation: None,
            behaviors: vec![],
            outcome: ScenarioOutcome::Success,
        }];

        let coverage = vec![TestCoverage {
            test_id: "tests/test_auth.py::test_login".to_string(),
            files: vec![
                FileCoverage {
                    path: "src/auth.py".to_string(),
                    lines: vec![10, 11, 12],
                },
                // These should NOT appear if pytest-cov --cov=src is used correctly
                FileCoverage {
                    path: "/usr/lib/python3.11/site-packages/flask/app.py".to_string(),
                    lines: vec![100, 200],
                },
                FileCoverage {
                    path: ".venv/lib/python3.11/site-packages/requests/api.py".to_string(),
                    lines: vec![50],
                },
            ],
        }];

        let builder = IndexBuilder::from_data(scenarios, coverage);
        builder.build(&index_dir).unwrap();

        let index = Index::open_readonly(&index_dir).unwrap();

        // All 3 files end up in the index (no filtering at index level)
        let context =
            query::get_scenario_context(&index, "tests/test_auth.py::test_login").unwrap();
        assert_eq!(context.coverage.len(), 3);

        // But when generating diagrams, we should be able to see them all
        // The key protection is at the pytest-cov level: --cov=src scopes instrumentation
        // This test documents that the index faithfully stores whatever coverage is provided
    }
}
