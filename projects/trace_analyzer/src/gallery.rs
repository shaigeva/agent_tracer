//! Self-contained HTML gallery for browsing and rendering flame graphs.
//!
//! Generates:
//! - `gallery.html` - single HTML file with embedded JS flame graph renderer
//! - `data/index.json` - scenario metadata index (small, loaded eagerly)
//! - `data/traces/<id>.json` - per-scenario call events (lazy-loaded on click)
//!
//! Designed to scale to thousands of scenarios: only the index is loaded
//! up-front; flame graphs are rendered client-side on demand.

use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::index::{Index, IndexError};
use crate::query;

/// Result of generating a gallery.
#[derive(Debug)]
pub struct GalleryResult {
    pub scenarios_total: usize,
    pub scenarios_with_traces: usize,
}

/// Index entry for a scenario (shown in the grid).
#[derive(Debug, Serialize)]
struct IndexEntry {
    id: String,
    safe_id: String,
    short_name: String,
    description: String,
    behaviors: Vec<String>,
    outcome: String,
    file_count: usize,
    event_count: usize,
    has_trace: bool,
}

/// Generate a complete gallery at `output_dir`.
pub fn generate_gallery(index: &Index, output_dir: &Path) -> Result<GalleryResult, IndexError> {
    fs::create_dir_all(output_dir)?;
    let data_dir = output_dir.join("data");
    fs::create_dir_all(&data_dir)?;
    let traces_dir = data_dir.join("traces");
    fs::create_dir_all(&traces_dir)?;

    let scenarios = query::list_scenarios(index, None, false)?;
    let mut scenarios_with_traces = 0;

    let mut entries: Vec<IndexEntry> = Vec::new();

    for scenario in &scenarios {
        let events = query::get_call_trace(index, &scenario.id)?;
        let context = query::get_scenario_context(index, &scenario.id)?;
        let safe_id = sanitize_filename(&scenario.id);
        let has_trace = !events.is_empty();

        if has_trace {
            // Write per-scenario trace data
            let trace_path = traces_dir.join(format!("{}.json", safe_id));
            let trace_json = serde_json::to_string(&events).map_err(|e| {
                IndexError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })?;
            fs::write(&trace_path, trace_json)?;
            scenarios_with_traces += 1;
        }

        entries.push(IndexEntry {
            id: scenario.id.clone(),
            safe_id: safe_id.clone(),
            short_name: scenario
                .id
                .split("::")
                .last()
                .unwrap_or(&scenario.id)
                .to_string(),
            description: scenario.description.clone(),
            behaviors: scenario.behaviors.clone(),
            outcome: scenario.outcome.clone(),
            file_count: context.coverage.len(),
            event_count: events.len(),
            has_trace,
        });
    }

    // Write the index
    let index_json = serde_json::to_string(&entries)
        .map_err(|e| IndexError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e))))?;
    fs::write(data_dir.join("index.json"), index_json)?;

    // Write the gallery HTML (with embedded JS)
    fs::write(output_dir.join("gallery.html"), GALLERY_HTML)?;

    // Also write the flame graph JS as a separate file so users can embed
    // it in their own pages
    fs::write(output_dir.join("flamegraph.js"), FLAMEGRAPH_JS)?;

    Ok(GalleryResult {
        scenarios_total: scenarios.len(),
        scenarios_with_traces,
    })
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// The standalone flame graph renderer JS.
/// Also written to gallery output as `flamegraph.js` for reuse.
const FLAMEGRAPH_JS: &str = include_str!("gallery_assets/flamegraph.js");

/// The gallery HTML page (loads flamegraph.js and index.json).
const GALLERY_HTML: &str = include_str!("gallery_assets/gallery.html");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(
            sanitize_filename("tests/test_auth.py::test_login"),
            "tests_test_auth_py__test_login"
        );
    }

    #[test]
    fn test_generate_gallery_empty_index() {
        use crate::index::{Index, IndexBuilder};
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let index_dir = temp.path().join(".trace-index");

        let builder = IndexBuilder::from_data(vec![], vec![]);
        builder.build(&index_dir).unwrap();

        let index = Index::open_readonly(&index_dir).unwrap();
        let output_dir = temp.path().join("gallery");
        let result = generate_gallery(&index, &output_dir).unwrap();

        assert_eq!(result.scenarios_total, 0);
        assert_eq!(result.scenarios_with_traces, 0);
        assert!(output_dir.join("gallery.html").exists());
        assert!(output_dir.join("flamegraph.js").exists());
        assert!(output_dir.join("data/index.json").exists());
    }

    #[test]
    fn test_generate_gallery_with_scenario() {
        use crate::index::{Index, IndexBuilder};
        use crate::models::{FileCoverage, Scenario, ScenarioOutcome, TestCoverage};
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let index_dir = temp.path().join(".trace-index");

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
            files: vec![FileCoverage {
                path: "src/auth.py".to_string(),
                lines: vec![10, 11, 12],
            }],
        }];

        let builder = IndexBuilder::from_data(scenarios, coverage);
        builder.build(&index_dir).unwrap();

        let index = Index::open_readonly(&index_dir).unwrap();
        let output_dir = temp.path().join("gallery");
        let result = generate_gallery(&index, &output_dir).unwrap();

        assert_eq!(result.scenarios_total, 1);
        assert_eq!(result.scenarios_with_traces, 0); // no call traces

        let index_json = fs::read_to_string(output_dir.join("data/index.json")).unwrap();
        assert!(index_json.contains("test_login"));
    }
}
