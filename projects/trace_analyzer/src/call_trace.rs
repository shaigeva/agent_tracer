//! Parser for call trace JSON files produced by the Python tracer.
//!
//! The call_traces.json file contains per-test function call/return events
//! captured via sys.monitoring, enabling flame graph and call-chain visualization.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::error::ScenarioError;

/// A single call or return event.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct CallEvent {
    pub event: String,    // "call" or "return"
    pub file: String,     // relative file path
    pub function: String, // qualified function name
    pub line: u32,
    pub depth: u32,
    pub timestamp_ns: u64,
}

/// Root structure of call_traces.json.
#[derive(Debug, Deserialize)]
struct CallTracesFile {
    #[allow(dead_code)]
    version: String,
    traces: HashMap<String, Vec<CallEvent>>,
}

/// Parsed call traces: map from test_id to list of events.
pub type CallTraces = HashMap<String, Vec<CallEvent>>;

/// Parse a call_traces.json file.
pub fn parse_call_traces(path: &Path) -> Result<CallTraces, ScenarioError> {
    let content = std::fs::read_to_string(path)?;
    let file: CallTracesFile = serde_json::from_str(&content)?;
    Ok(file.traces)
}

/// Generate folded stacks format from call events (for flame graph tools).
///
/// Each line is: `stack;frames count\n`
/// where stack is semicolon-separated function names representing the call stack.
///
/// Frame format is `module.qualname` where module is the file stem (no path, no .py).
/// This keeps frames short enough to display on narrow bars while staying unique
/// (qualname already includes class prefix for methods).
pub fn to_folded_stacks(events: &[CallEvent]) -> String {
    let mut result = String::new();
    let mut stack: Vec<String> = Vec::new();

    for event in events {
        match event.event.as_str() {
            "call" => {
                let frame = format_frame(&event.file, &event.function);
                stack.push(frame);
                let stack_str = stack.join(";");
                result.push_str(&stack_str);
                result.push_str(" 1\n");
            }
            "return" => {
                stack.pop();
            }
            _ => {}
        }
    }

    result
}

/// Format a single frame for flame graph display.
/// Uses `module.qualname` where module is the file stem.
pub fn format_frame(file: &str, function: &str) -> String {
    let module = file_stem(file);
    format!("{}.{}", module, function)
}

/// Extract the file stem (filename without directory or extension).
fn file_stem(path: &str) -> String {
    let filename = match path.rfind('/') {
        Some(idx) => &path[idx + 1..],
        None => path,
    };
    match filename.rfind('.') {
        Some(idx) => filename[..idx].to_string(),
        None => filename.to_string(),
    }
}

/// Generate a mermaid sequence diagram from call events.
pub fn to_mermaid_sequence(events: &[CallEvent], scenario_name: &str) -> String {
    let mut mermaid = String::new();
    mermaid.push_str("sequenceDiagram\n");

    // Extract unique files as participants
    let mut seen_files: Vec<String> = Vec::new();
    for event in events {
        if event.event == "call" {
            let short = short_path(&event.file);
            if !seen_files.contains(&short) {
                seen_files.push(short);
            }
        }
    }

    // Add participants in order of first appearance
    mermaid.push_str(&format!("    participant test as {}\n", scenario_name));
    for file in &seen_files {
        let alias = file.replace(['/', '.'], "_");
        mermaid.push_str(&format!("    participant {} as {}\n", alias, file));
    }

    // Track the "current file" at each depth to draw arrows between files
    let mut depth_file: Vec<String> = vec!["test".to_string()];

    for event in events {
        if event.event == "call" {
            let target = short_path(&event.file).replace(['/', '.'], "_");
            let source = if event.depth == 0 {
                "test".to_string()
            } else {
                depth_file
                    .get(event.depth as usize)
                    .cloned()
                    .unwrap_or_else(|| "test".to_string())
            };

            // Only draw arrow if calling a different file
            if source != target {
                mermaid.push_str(&format!(
                    "    {} ->> {}: {}\n",
                    source, target, event.function
                ));
            }

            // Update depth tracking
            let new_depth = event.depth as usize + 1;
            if depth_file.len() <= new_depth {
                depth_file.resize(new_depth + 1, String::new());
            }
            depth_file[new_depth] = target;
        }
    }

    mermaid
}

/// Render a flame graph SVG from call events using inferno.
///
/// Returns an SVG string. With `fixed_width=None` the SVG is "fluid" (scales
/// to container; needs interactive JS for label layout). With a fixed width,
/// labels are computed statically so text renders correctly in non-JS viewers
/// and when rasterized to PNG.
pub fn to_svg_flamegraph(events: &[CallEvent], title: &str) -> Result<String, String> {
    render_svg(events, title, None)
}

/// Render a flame graph with a fixed pixel width (useful for static viewers).
pub fn to_svg_flamegraph_fixed(
    events: &[CallEvent],
    title: &str,
    width: usize,
) -> Result<String, String> {
    render_svg(events, title, Some(width))
}

/// Render a flame graph with a fixed pixel width. Used for PNG rasterization
/// where we want deterministic label layout since JS can't run.
fn render_svg(
    events: &[CallEvent],
    title: &str,
    fixed_width: Option<usize>,
) -> Result<String, String> {
    let folded = to_folded_stacks(events);
    if folded.is_empty() {
        return Err("No events to render".to_string());
    }

    let mut options = inferno::flamegraph::Options::default();
    options.title = title.to_string();
    options.subtitle = Some("Call trace from sys.monitoring".to_string());
    options.count_name = "calls".to_string();
    options.font_size = 12;
    options.image_width = fixed_width;

    let mut svg = Vec::new();
    inferno::flamegraph::from_lines(&mut options, folded.lines(), &mut svg)
        .map_err(|e| format!("Flame graph rendering failed: {}", e))?;

    String::from_utf8(svg).map_err(|e| format!("Invalid UTF-8 in SVG output: {}", e))
}

/// Render a flame graph PNG from call events.
///
/// Uses inferno to produce a fixed-width SVG (so labels layout correctly
/// without JS), then resvg to rasterize to PNG at 2x scale for crispness.
/// Returns PNG bytes. Static image (no interactivity) but renders in any viewer.
pub fn to_png_flamegraph(events: &[CallEvent], title: &str) -> Result<Vec<u8>, String> {
    // Use a wide canvas (2400px) so labels fit on narrow bars.
    // Without JS, inferno can only truncate based on declared width.
    let svg = render_svg(events, title, Some(2400))?;
    svg_to_png(&svg, 1.5)
}

/// Convert SVG string to PNG bytes using resvg. Scale factor doubles resolution.
fn svg_to_png(svg: &str, scale: f32) -> Result<Vec<u8>, String> {
    use resvg::usvg;

    let opt = usvg::Options::default();
    let tree =
        usvg::Tree::from_str(svg, &opt).map_err(|e| format!("Failed to parse SVG: {}", e))?;

    let size = tree.size();
    let width = (size.width() * scale) as u32;
    let height = (size.height() * scale) as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "Failed to allocate pixmap".to_string())?;

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap
        .encode_png()
        .map_err(|e| format!("PNG encoding failed: {}", e))
}

/// Wrap an SVG in an HTML page for guaranteed browser rendering.
///
/// Some browsers restrict scripts in SVGs loaded via file://, and VS Code shows
/// SVGs as XML text by default. Wrapping in HTML sidesteps both issues.
pub fn to_html_flamegraph(events: &[CallEvent], title: &str) -> Result<String, String> {
    let svg = to_svg_flamegraph(events, title)?;
    Ok(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>{}</title>
<style>
  body {{ margin: 0; padding: 1rem; font-family: -apple-system, sans-serif; background: #f5f5f7; }}
  h1 {{ margin: 0 0 1rem 0; font-size: 1.1rem; font-family: ui-monospace, Menlo, monospace; }}
  .hint {{ color: #6e6e73; font-size: 0.85rem; margin-bottom: 1rem; }}
  .container {{ background: white; border-radius: 8px; padding: 1rem; box-shadow: 0 2px 6px rgba(0,0,0,0.05); }}
</style>
</head>
<body>
<h1>{}</h1>
<p class="hint">Interactive flame graph. Click bars to zoom. Right-click to zoom out. Type in the Search field to highlight.</p>
<div class="container">
{}
</div>
</body>
</html>
"#,
        html_escape(title),
        html_escape(title),
        svg
    ))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Shorten a file path to just the last directory + filename.
fn short_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        format!("{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_folded_stacks() {
        let events = vec![
            CallEvent {
                event: "call".to_string(),
                file: "src/routes/order_routes.py".to_string(),
                function: "OrderRoutes.post_order".to_string(),
                line: 10,
                depth: 0,
                timestamp_ns: 100,
            },
            CallEvent {
                event: "call".to_string(),
                file: "src/middleware/auth.py".to_string(),
                function: "AuthMiddleware.create_order".to_string(),
                line: 20,
                depth: 1,
                timestamp_ns: 200,
            },
            CallEvent {
                event: "call".to_string(),
                file: "src/services/order_service.py".to_string(),
                function: "OrderService.create_order".to_string(),
                line: 30,
                depth: 2,
                timestamp_ns: 300,
            },
            CallEvent {
                event: "return".to_string(),
                file: "src/services/order_service.py".to_string(),
                function: "OrderService.create_order".to_string(),
                line: 30,
                depth: 2,
                timestamp_ns: 400,
            },
            CallEvent {
                event: "return".to_string(),
                file: "src/middleware/auth.py".to_string(),
                function: "AuthMiddleware.create_order".to_string(),
                line: 20,
                depth: 1,
                timestamp_ns: 500,
            },
            CallEvent {
                event: "return".to_string(),
                file: "src/routes/order_routes.py".to_string(),
                function: "OrderRoutes.post_order".to_string(),
                line: 10,
                depth: 0,
                timestamp_ns: 600,
            },
        ];

        let folded = to_folded_stacks(&events);
        let lines: Vec<&str> = folded.trim().lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("OrderRoutes.post_order"));
        assert!(lines[1].contains("AuthMiddleware.create_order"));
        assert!(lines[2].contains("OrderService.create_order"));
        // Verify nesting
        assert!(lines[2].contains(";"));
    }

    #[test]
    fn test_to_mermaid_sequence() {
        let events = vec![
            CallEvent {
                event: "call".to_string(),
                file: "src/routes/order_routes.py".to_string(),
                function: "post_order".to_string(),
                line: 10,
                depth: 0,
                timestamp_ns: 100,
            },
            CallEvent {
                event: "call".to_string(),
                file: "src/middleware/auth.py".to_string(),
                function: "create_order".to_string(),
                line: 20,
                depth: 1,
                timestamp_ns: 200,
            },
            CallEvent {
                event: "return".to_string(),
                file: "src/middleware/auth.py".to_string(),
                function: "create_order".to_string(),
                line: 20,
                depth: 1,
                timestamp_ns: 300,
            },
            CallEvent {
                event: "return".to_string(),
                file: "src/routes/order_routes.py".to_string(),
                function: "post_order".to_string(),
                line: 10,
                depth: 0,
                timestamp_ns: 400,
            },
        ];

        let mermaid = to_mermaid_sequence(&events, "test_order");
        assert!(mermaid.contains("sequenceDiagram"));
        assert!(mermaid.contains("test ->> "));
        assert!(mermaid.contains("create_order"));
    }

    #[test]
    fn test_short_path() {
        assert_eq!(
            short_path("src/routes/order_routes.py"),
            "routes/order_routes.py"
        );
        assert_eq!(short_path("auth.py"), "auth.py");
    }

    #[test]
    fn test_parse_call_traces() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let json = r#"{
            "version": "1.0",
            "traces": {
                "tests/test_auth.py::test_login": [
                    {"event": "call", "file": "src/auth.py", "function": "login", "line": 10, "depth": 0, "timestamp_ns": 100},
                    {"event": "return", "file": "src/auth.py", "function": "login", "line": 10, "depth": 0, "timestamp_ns": 200}
                ]
            }
        }"#;

        let mut f = NamedTempFile::new().unwrap();
        f.write_all(json.as_bytes()).unwrap();

        let traces = parse_call_traces(f.path()).unwrap();
        assert_eq!(traces.len(), 1);
        let events = &traces["tests/test_auth.py::test_login"];
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event, "call");
        assert_eq!(events[0].function, "login");
    }
}
