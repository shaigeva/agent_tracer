//! Self-contained HTML gallery of all scenarios.
//!
//! Generates a directory containing:
//! - index.html with grid of all scenarios
//! - flamegraphs/<scenario>.svg for each scenario with call traces
//! - Individual scenario detail pages with embedded flame graph, sequence diagram,
//!   and coverage info

use std::fs;
use std::path::Path;

use crate::call_trace;
use crate::index::{Index, IndexError};
use crate::query;

/// Result of generating a gallery.
#[derive(Debug)]
pub struct GalleryResult {
    pub scenarios_with_traces: usize,
    pub scenarios_without_traces: usize,
}

/// Generate a complete HTML gallery at `output_dir`.
pub fn generate_gallery(index: &Index, output_dir: &Path) -> Result<GalleryResult, IndexError> {
    // Create output directory structure
    fs::create_dir_all(output_dir)?;
    let flamegraphs_dir = output_dir.join("flamegraphs");
    fs::create_dir_all(&flamegraphs_dir)?;
    let details_dir = output_dir.join("scenarios");
    fs::create_dir_all(&details_dir)?;

    let scenarios = query::list_scenarios(index, None, false)?;

    let mut scenarios_with_traces = 0;
    let mut scenarios_without_traces = 0;

    // Collect info for the index page
    let mut entries: Vec<GalleryEntry> = Vec::new();

    for scenario in &scenarios {
        let events = query::get_call_trace(index, &scenario.id)?;
        let context = query::get_scenario_context(index, &scenario.id)?;

        let safe_filename = sanitize_filename(&scenario.id);
        let has_trace = !events.is_empty();

        // Generate flame graph SVG if we have trace data
        if has_trace {
            let short_name = scenario.id.split("::").last().unwrap_or(&scenario.id);
            match call_trace::to_svg_flamegraph(&events, short_name) {
                Ok(svg) => {
                    let svg_path = flamegraphs_dir.join(format!("{}.svg", safe_filename));
                    fs::write(&svg_path, svg)?;
                    scenarios_with_traces += 1;
                }
                Err(_) => {
                    scenarios_without_traces += 1;
                }
            }
        } else {
            scenarios_without_traces += 1;
        }

        // Generate detail page
        let detail_html = render_detail_page(scenario, &events, &context);
        let detail_path = details_dir.join(format!("{}.html", safe_filename));
        fs::write(&detail_path, detail_html)?;

        entries.push(GalleryEntry {
            short_name: scenario
                .id
                .split("::")
                .last()
                .unwrap_or(&scenario.id)
                .to_string(),
            description: scenario.description.clone(),
            behaviors: scenario.behaviors.clone(),
            outcome: scenario.outcome.clone(),
            safe_filename,
            has_trace,
            file_count: context.coverage.len(),
            event_count: events.len(),
        });
    }

    // Generate index.html
    let index_html = render_index_page(&entries);
    fs::write(output_dir.join("index.html"), index_html)?;

    Ok(GalleryResult {
        scenarios_with_traces,
        scenarios_without_traces,
    })
}

struct GalleryEntry {
    short_name: String,
    description: String,
    behaviors: Vec<String>,
    outcome: String,
    safe_filename: String,
    has_trace: bool,
    file_count: usize,
    event_count: usize,
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

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render_index_page(entries: &[GalleryEntry]) -> String {
    let mut html = String::new();
    html.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Trace Gallery</title>
<style>
  body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; margin: 0; padding: 2rem; background: #f5f5f7; color: #1d1d1f; }
  h1 { margin: 0 0 0.5rem 0; font-size: 2rem; }
  .subtitle { color: #6e6e73; margin-bottom: 2rem; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(380px, 1fr)); gap: 1.5rem; }
  .card { background: white; border-radius: 12px; padding: 1.25rem; box-shadow: 0 2px 8px rgba(0,0,0,0.06); transition: transform 0.1s, box-shadow 0.1s; text-decoration: none; color: inherit; display: flex; flex-direction: column; }
  .card:hover { transform: translateY(-2px); box-shadow: 0 6px 20px rgba(0,0,0,0.1); }
  .card h3 { margin: 0 0 0.5rem 0; font-size: 1.05rem; font-family: ui-monospace, Menlo, monospace; }
  .card .desc { color: #424245; font-size: 0.9rem; margin-bottom: 0.75rem; flex-grow: 1; }
  .card .meta { display: flex; gap: 0.5rem; flex-wrap: wrap; font-size: 0.75rem; margin-bottom: 0.5rem; }
  .tag { background: #e8e8ed; padding: 2px 8px; border-radius: 4px; color: #424245; }
  .tag.error { background: #ffe5e5; color: #a30000; }
  .tag.success { background: #d5f5d5; color: #006400; }
  .stats { font-size: 0.75rem; color: #6e6e73; margin-top: auto; padding-top: 0.5rem; border-top: 1px solid #f0f0f0; }
  .no-trace { color: #a00; font-style: italic; }
  .summary { background: white; border-radius: 12px; padding: 1rem 1.5rem; margin-bottom: 2rem; box-shadow: 0 2px 8px rgba(0,0,0,0.06); }
  .thumbnail { width: 100%; height: 80px; background: #fafafa; border: 1px solid #e8e8ed; border-radius: 6px; margin-bottom: 0.75rem; overflow: hidden; position: relative; }
  .thumbnail object { width: 100%; height: 100%; pointer-events: none; }
  .thumbnail.empty { display: flex; align-items: center; justify-content: center; color: #999; font-size: 0.85rem; }
</style>
</head>
<body>
"#);

    html.push_str(&format!(
        r#"<h1>Trace Gallery</h1>
<p class="subtitle">{} scenarios</p>
"#,
        entries.len()
    ));

    let with_traces = entries.iter().filter(|e| e.has_trace).count();
    let without_traces = entries.len() - with_traces;
    html.push_str(&format!(
        r#"<div class="summary">
  <strong>{}</strong> with flame graphs &middot;
  <strong>{}</strong> coverage only
</div>
"#,
        with_traces, without_traces
    ));

    html.push_str(r#"<div class="grid">"#);

    for entry in entries {
        let outcome_class = if entry.outcome == "error" {
            "error"
        } else {
            "success"
        };
        let behaviors_html: String = entry
            .behaviors
            .iter()
            .map(|b| format!(r#"<span class="tag">{}</span>"#, html_escape(b)))
            .collect();

        let thumbnail = if entry.has_trace {
            format!(
                r#"<div class="thumbnail"><object data="flamegraphs/{}.svg" type="image/svg+xml"></object></div>"#,
                entry.safe_filename
            )
        } else {
            r#"<div class="thumbnail empty">no call trace</div>"#.to_string()
        };

        let trace_info = if entry.has_trace {
            format!(
                "{} files &middot; {} call events",
                entry.file_count, entry.event_count
            )
        } else {
            format!(
                r#"<span class="no-trace">{} files (coverage only)</span>"#,
                entry.file_count
            )
        };

        html.push_str(&format!(
            r#"<a class="card" href="scenarios/{}.html">
  {}
  <h3>{}</h3>
  <div class="desc">{}</div>
  <div class="meta">
    <span class="tag {}">{}</span>
    {}
  </div>
  <div class="stats">{}</div>
</a>
"#,
            entry.safe_filename,
            thumbnail,
            html_escape(&entry.short_name),
            html_escape(&entry.description),
            outcome_class,
            entry.outcome,
            behaviors_html,
            trace_info
        ));
    }

    html.push_str(
        r#"</div>
</body>
</html>
"#,
    );

    html
}

fn render_detail_page(
    scenario: &query::ScenarioInfo,
    events: &[call_trace::CallEvent],
    context: &query::ScenarioContext,
) -> String {
    let mut html = String::new();
    html.push_str(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>"#,
    );
    html.push_str(&html_escape(&scenario.id));
    html.push_str(r#"</title>
<style>
  body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; margin: 0; padding: 2rem; background: #f5f5f7; color: #1d1d1f; max-width: 1400px; margin: 0 auto; }
  h1 { margin: 0 0 0.5rem 0; font-size: 1.5rem; font-family: ui-monospace, Menlo, monospace; }
  h2 { font-size: 1.25rem; margin-top: 2rem; }
  .back { color: #007aff; text-decoration: none; }
  .back:hover { text-decoration: underline; }
  .section { background: white; border-radius: 12px; padding: 1.5rem; margin-bottom: 1.5rem; box-shadow: 0 2px 8px rgba(0,0,0,0.06); }
  .description { color: #424245; font-size: 1rem; }
  .meta { display: flex; gap: 0.5rem; flex-wrap: wrap; margin: 0.5rem 0; }
  .tag { background: #e8e8ed; padding: 2px 10px; border-radius: 4px; font-size: 0.85rem; color: #424245; }
  .tag.error { background: #ffe5e5; color: #a30000; }
  .tag.success { background: #d5f5d5; color: #006400; }
  .docs { background: #f5f5f7; padding: 1rem; border-radius: 8px; font-family: ui-monospace, Menlo, monospace; font-size: 0.85rem; white-space: pre-wrap; }
  .flamegraph-container { overflow: auto; }
  .flamegraph-container object { width: 100%; min-height: 400px; }
  table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
  th, td { text-align: left; padding: 6px 10px; border-bottom: 1px solid #f0f0f0; }
  th { color: #6e6e73; font-weight: 500; font-size: 0.8rem; text-transform: uppercase; }
  td.file { font-family: ui-monospace, Menlo, monospace; }
  .no-data { color: #6e6e73; font-style: italic; }
</style>
</head>
<body>
"#);

    html.push_str(r#"<p><a class="back" href="../index.html">&larr; Back to gallery</a></p>"#);

    let outcome_class = if scenario.outcome == "error" {
        "error"
    } else {
        "success"
    };
    let behaviors_html: String = scenario
        .behaviors
        .iter()
        .map(|b| format!(r#"<span class="tag">{}</span>"#, html_escape(b)))
        .collect();

    html.push_str(&format!(
        r#"<div class="section">
  <h1>{}</h1>
  <div class="meta">
    <span class="tag {}">{}</span>
    {}
  </div>
  <p class="description">{}</p>
"#,
        html_escape(&scenario.id),
        outcome_class,
        scenario.outcome,
        behaviors_html,
        html_escape(&scenario.description)
    ));

    if let Some(ref doc) = scenario.documentation {
        if doc != &scenario.description {
            html.push_str(&format!(
                r#"<div class="docs">{}</div>
"#,
                html_escape(doc)
            ));
        }
    }

    html.push_str("</div>\n");

    // Flame graph section
    html.push_str(
        r#"<div class="section">
  <h2>Flame Graph</h2>
"#,
    );
    if !events.is_empty() {
        let safe = sanitize_filename(&scenario.id);
        html.push_str(&format!(
            r#"<div class="flamegraph-container">
    <object data="../flamegraphs/{}.svg" type="image/svg+xml"></object>
  </div>
  <p style="font-size: 0.85rem; color: #6e6e73;">Click bars to zoom. {} call events recorded.</p>
"#,
            safe,
            events.len()
        ));
    } else {
        html.push_str(
            r#"<p class="no-data">No call trace data. Build the index with --call-traces to enable flame graphs.</p>"#,
        );
    }
    html.push_str("</div>\n");

    // Call-chain sequence diagram (mermaid)
    if !events.is_empty() {
        let short_name = scenario.id.split("::").last().unwrap_or(&scenario.id);
        let mermaid = call_trace::to_mermaid_sequence(events, short_name);
        html.push_str(
            r#"<div class="section">
  <h2>Call Sequence</h2>
  <pre class="docs">"#,
        );
        html.push_str(&html_escape(&mermaid));
        html.push_str(r#"</pre>
  <p style="font-size: 0.85rem; color: #6e6e73;">Paste this mermaid source at <a href="https://mermaid.live" target="_blank">mermaid.live</a> to view as a sequence diagram.</p>
</div>
"#);
    }

    // Coverage section
    html.push_str(
        r#"<div class="section">
  <h2>Coverage</h2>
"#,
    );
    if !context.coverage.is_empty() {
        html.push_str(&format!(
            r#"<p>{} files touched</p>
<table>
  <thead><tr><th>File</th><th>Lines covered</th></tr></thead>
  <tbody>
"#,
            context.coverage.len()
        ));
        let mut sorted = context.coverage.clone();
        sorted.sort_by(|a, b| a.path.cmp(&b.path));
        for file_cov in &sorted {
            html.push_str(&format!(
                r#"<tr><td class="file">{}</td><td>{}</td></tr>
"#,
                html_escape(&file_cov.path),
                file_cov.lines.len()
            ));
        }
        html.push_str("</tbody></table>\n");
    } else {
        html.push_str(r#"<p class="no-data">No coverage data.</p>"#);
    }
    html.push_str("</div>\n");

    html.push_str("</body>\n</html>\n");
    html
}

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
    fn test_html_escape() {
        assert_eq!(html_escape("a<b>c&d\"e"), "a&lt;b&gt;c&amp;d&quot;e");
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

        assert_eq!(result.scenarios_with_traces, 0);
        assert_eq!(result.scenarios_without_traces, 0);
        assert!(output_dir.join("index.html").exists());
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

        // No traces because we didn't provide any call trace events
        assert_eq!(result.scenarios_without_traces, 1);
        assert!(output_dir.join("index.html").exists());
        assert!(output_dir
            .join("scenarios")
            .join("tests_test_auth_py__test_login.html")
            .exists());
    }
}
