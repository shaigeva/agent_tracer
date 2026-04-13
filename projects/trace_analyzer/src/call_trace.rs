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

/// Filter options applied when rendering flame graphs and derived formats.
#[derive(Debug, Clone, Default)]
pub struct FilterOptions {
    /// Anchor frame. When set, stacks are trimmed to start at the first frame
    /// matching this pattern; stacks that never reach the anchor are dropped.
    /// This is how agents "zoom in" on the test body past fixture setup.
    ///
    /// Matching: frame ends with `.{pattern}` OR frame == pattern OR frame contains pattern.
    pub anchor_function: Option<String>,
    /// If true, ignore `anchor_function` (including any auto-anchor the caller set)
    /// and emit the full stack tree. Useful for debugging fixture setup itself.
    pub include_fixtures: bool,
    /// If non-empty, keep only stacks containing a frame matching one of these patterns.
    /// Patterns support simple globs: `foo*` (prefix), `*foo` (suffix), `foo` (substring).
    pub include_patterns: Vec<String>,
    /// Drop stacks containing a frame matching any of these patterns.
    pub exclude_patterns: Vec<String>,
    /// Cap stacks at this depth **relative to the anchor**. A value of 3 means
    /// "anchor + 3 more frames". Applied AFTER anchoring, so fixture setup
    /// above the anchor never counts toward the budget.
    pub max_depth: Option<u32>,
    /// Frames matching any of these patterns are removed from output while the
    /// surrounding stack is kept (skip-and-reparent). Unlike --exclude, which
    /// drops the entire stack, --skip removes only the matching frame.
    pub skip_patterns: Vec<String>,
}

impl FilterOptions {
    /// Should we apply the anchor filter? Yes if anchor is set AND include_fixtures is not.
    fn effective_anchor(&self) -> Option<&str> {
        if self.include_fixtures {
            None
        } else {
            self.anchor_function.as_deref()
        }
    }
}

/// Check if a frame matches the anchor pattern. Matches if frame ends with
/// `.{anchor}`, frame is exactly `anchor`, or frame contains `anchor` as a substring.
fn matches_anchor(frame: &str, anchor: &str) -> bool {
    if frame == anchor {
        return true;
    }
    let dotted = format!(".{}", anchor);
    if frame.ends_with(&dotted) {
        return true;
    }
    // Fallback to substring for patterns the user provided as globs (e.g. from --from)
    matches_pattern(frame, anchor)
}

/// Parse a comma-separated list of patterns.
pub fn parse_patterns(s: &str) -> Vec<String> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Check if a file path is a pytest conftest (fixture) file.
pub fn is_fixture_file(path: &str) -> bool {
    path.ends_with("conftest.py") || path.contains("/conftest.py")
}

/// Simple glob matching: supports `foo*` (prefix), `*foo` (suffix), `foo` (substring).
fn matches_pattern(frame: &str, pattern: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        if let Some(rest) = prefix.strip_prefix('*') {
            // *foo* = contains
            frame.contains(rest)
        } else {
            frame.starts_with(prefix)
        }
    } else if let Some(suffix) = pattern.strip_prefix('*') {
        frame.ends_with(suffix)
    } else {
        // No wildcards: substring match (agent feedback suggested this is most useful)
        frame.contains(pattern)
    }
}

fn stack_passes_filters(frames: &[String], opts: &FilterOptions) -> bool {
    if !opts.include_patterns.is_empty() {
        let any_match = frames
            .iter()
            .any(|f| opts.include_patterns.iter().any(|p| matches_pattern(f, p)));
        if !any_match {
            return false;
        }
    }
    if !opts.exclude_patterns.is_empty() {
        let any_match = frames
            .iter()
            .any(|f| opts.exclude_patterns.iter().any(|p| matches_pattern(f, p)));
        if any_match {
            return false;
        }
    }
    true
}

/// Generate folded stacks format with default (no) filtering.
/// Kept for backward compatibility; new callers should use `to_folded_stacks_filtered`.
pub fn to_folded_stacks(events: &[CallEvent]) -> String {
    to_folded_stacks_filtered(events, &FilterOptions::default())
}

/// Generate folded stacks format from call events with filter options applied.
///
/// Each line is: `stack;frames count\n`
/// where stack is semicolon-separated function names representing the call stack.
///
/// Frame format is `module.qualname` where module is the file stem (no path, no .py).
pub fn to_folded_stacks_filtered(events: &[CallEvent], opts: &FilterOptions) -> String {
    let lines = build_stack_lines(events, opts);
    let mut result = String::with_capacity(lines.len() * 64);
    for line in &lines {
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// Build the raw list of stack lines (each: "frame1;frame2;frame3 1") with filters applied.
///
/// When `anchor_function` is set, each emitted line is trimmed to start at the
/// first frame matching the anchor; stacks that never reach the anchor are dropped.
/// `max_depth` and `skip_patterns` are applied AFTER anchoring, so fixture setup
/// above the anchor never counts toward the depth budget and skipped frames
/// are removed without dropping the surrounding stack.
fn build_stack_lines(events: &[CallEvent], opts: &FilterOptions) -> Vec<String> {
    let mut stack: Vec<String> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    let anchor = opts.effective_anchor();

    for event in events {
        match event.event.as_str() {
            "call" => {
                let frame = format_frame(&event.file, &event.function);
                stack.push(frame);

                // Compute the anchored slice (trim to anchor if set)
                let anchored: Option<&[String]> = if let Some(anchor_pat) = anchor {
                    stack
                        .iter()
                        .position(|f| matches_anchor(f, anchor_pat))
                        .map(|i| &stack[i..])
                } else {
                    Some(&stack[..])
                };

                if let Some(frames) = anchored {
                    // Skip-and-reparent: remove frames matching skip_patterns
                    let after_skip: Vec<&String> = frames
                        .iter()
                        .filter(|f| !opts.skip_patterns.iter().any(|p| matches_pattern(f, p)))
                        .collect();

                    // Apply max_depth relative to anchor (i.e. the trimmed length)
                    let limited: Vec<&String> = if let Some(max) = opts.max_depth {
                        let limit = (max as usize) + 1;
                        after_skip.into_iter().take(limit).collect()
                    } else {
                        after_skip
                    };

                    if !limited.is_empty() {
                        let owned: Vec<String> = limited.iter().map(|s| s.to_string()).collect();
                        if stack_passes_filters(&owned, opts) {
                            lines.push(format!("{} 1", owned.join(";")));
                        }
                    }
                }
            }
            "return" => {
                stack.pop();
            }
            _ => {}
        }
    }

    lines
}

/// Generate folded-compact stacks: prefix collapse with ellipsis.
///
/// Each line after the first replaces its common prefix with the previous line
/// by `...(N)` where N is the number of collapsed frames. Dramatically reduces
/// token count for deeply-nested traces.
pub fn to_folded_compact(events: &[CallEvent], opts: &FilterOptions) -> String {
    let lines = build_stack_lines(events, opts);
    let mut result = String::new();
    let mut prev_frames: Vec<&str> = Vec::new();

    for line in &lines {
        let (stack_part, count_part) = line.rsplit_once(' ').unwrap_or((line.as_str(), "1"));
        let frames: Vec<&str> = stack_part.split(';').collect();

        // Find common prefix length with previous line
        let common = frames
            .iter()
            .zip(prev_frames.iter())
            .take_while(|(a, b)| a == b)
            .count();

        if common >= 2 && common < frames.len() {
            let rest = &frames[common..];
            result.push_str(&format!(
                "...({}) ;{} {}\n",
                common,
                rest.join(";"),
                count_part
            ));
        } else {
            result.push_str(&format!("{} {}\n", frames.join(";"), count_part));
        }

        prev_frames = frames;
    }

    result
}

/// Entry in the summary format.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SummaryFrame {
    pub frame: String,
    pub depth: u32,
    pub calls: u32,
    pub file: String,
}

/// Generate a compact summary: unique frames in order of first appearance,
/// with their depth and total call count.
///
/// This is typically 10-50x shorter than folded stacks and is what agents
/// usually want: "what functions does this test touch?"
pub fn to_summary(events: &[CallEvent], opts: &FilterOptions) -> Vec<SummaryFrame> {
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut result: Vec<SummaryFrame> = Vec::new();
    let anchor = opts.effective_anchor();
    let mut anchor_depth: Option<u32> = None;

    for event in events {
        match event.event.as_str() {
            "call" => {
                let frame = format_frame(&event.file, &event.function);

                // Check if this frame establishes the anchor
                if anchor_depth.is_none() {
                    if let Some(a) = anchor {
                        if matches_anchor(&frame, a) {
                            anchor_depth = Some(event.depth);
                        }
                    }
                }

                // If an anchor is required but not yet reached, skip emission
                if anchor.is_some() && anchor_depth.is_none() {
                    continue;
                }

                // Compute depth relative to anchor (0-based) when anchored
                let emit_depth = match anchor_depth {
                    Some(ad) => event.depth.saturating_sub(ad),
                    None => event.depth,
                };

                // Apply max_depth RELATIVE TO ANCHOR
                if let Some(max) = opts.max_depth {
                    if emit_depth > max {
                        continue;
                    }
                }

                // Skip-and-reparent: drop this frame from the summary but keep
                // processing subsequent frames (no reparenting needed in summary
                // since each frame is emitted independently).
                if opts
                    .skip_patterns
                    .iter()
                    .any(|p| matches_pattern(&frame, p))
                {
                    continue;
                }

                // Apply include/exclude filters on the single frame
                if !opts.include_patterns.is_empty()
                    && !opts
                        .include_patterns
                        .iter()
                        .any(|p| matches_pattern(&frame, p))
                {
                    continue;
                }
                if opts
                    .exclude_patterns
                    .iter()
                    .any(|p| matches_pattern(&frame, p))
                {
                    continue;
                }

                if let Some(&idx) = seen.get(&frame) {
                    result[idx].calls += 1;
                } else {
                    seen.insert(frame.clone(), result.len());
                    result.push(SummaryFrame {
                        frame,
                        depth: emit_depth,
                        calls: 1,
                        file: event.file.clone(),
                    });
                }
            }
            "return" => {
                // If we're exiting the anchor frame, reset so re-entries work.
                if let Some(ad) = anchor_depth {
                    if event.depth < ad {
                        anchor_depth = None;
                    }
                }
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

/// Generate a mermaid sequence diagram from call events with default (no) filtering.
pub fn to_mermaid_sequence(events: &[CallEvent], scenario_name: &str) -> String {
    to_mermaid_sequence_filtered(events, scenario_name, &FilterOptions::default())
}

/// Generate a mermaid sequence diagram from call events with filter options.
pub fn to_mermaid_sequence_filtered(
    events: &[CallEvent],
    scenario_name: &str,
    opts: &FilterOptions,
) -> String {
    // Apply fixture / depth filtering to produce the effective event sequence
    let filtered = filter_events(events, opts);
    to_mermaid_sequence_impl(&filtered, scenario_name)
}

/// Produce a filtered event list: trim everything before the anchor frame,
/// apply depth cap relative to anchor, skip matching frames.
/// Pattern filters (include/exclude) are applied per-stack, not here.
fn filter_events(events: &[CallEvent], opts: &FilterOptions) -> Vec<CallEvent> {
    let mut result = Vec::with_capacity(events.len());
    let anchor = opts.effective_anchor();
    let mut anchor_depth: Option<u32> = None;

    for event in events {
        // When an anchor is required, skip everything until we see a call matching it.
        if let Some(a) = anchor {
            if anchor_depth.is_none() {
                if event.event == "call" {
                    let frame = format_frame(&event.file, &event.function);
                    if matches_anchor(&frame, a) {
                        anchor_depth = Some(event.depth);
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            } else if event.event == "return" {
                if let Some(ad) = anchor_depth {
                    if event.depth < ad {
                        anchor_depth = None;
                        continue;
                    }
                }
            }
        }

        // Relative depth (0 at the anchor) for max_depth and matching.
        let emit_depth = match anchor_depth {
            Some(ad) => event.depth.saturating_sub(ad),
            None => event.depth,
        };

        if let Some(max) = opts.max_depth {
            if emit_depth > max {
                continue;
            }
        }

        // Skip frames matching skip_patterns (keep stack otherwise intact)
        if event.event == "call" {
            let frame = format_frame(&event.file, &event.function);
            if opts
                .skip_patterns
                .iter()
                .any(|p| matches_pattern(&frame, p))
            {
                continue;
            }
        }
        result.push(event.clone());
    }

    result
}

fn to_mermaid_sequence_impl(events: &[CallEvent], scenario_name: &str) -> String {
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

/// Render a flame graph SVG from call events using inferno (default filters).
pub fn to_svg_flamegraph(events: &[CallEvent], title: &str) -> Result<String, String> {
    render_svg(events, title, None, &FilterOptions::default())
}

/// Render a flame graph SVG with explicit filter options.
pub fn to_svg_flamegraph_filtered(
    events: &[CallEvent],
    title: &str,
    opts: &FilterOptions,
) -> Result<String, String> {
    render_svg(events, title, None, opts)
}

/// Render a flame graph with a fixed pixel width (useful for static viewers).
pub fn to_svg_flamegraph_fixed(
    events: &[CallEvent],
    title: &str,
    width: usize,
) -> Result<String, String> {
    render_svg(events, title, Some(width), &FilterOptions::default())
}

/// Render a flame graph SVG with fixed width and filter options.
fn render_svg(
    events: &[CallEvent],
    title: &str,
    fixed_width: Option<usize>,
    opts: &FilterOptions,
) -> Result<String, String> {
    let folded = to_folded_stacks_filtered(events, opts);
    if folded.is_empty() {
        return Err("No events to render (all filtered out? try --include-fixtures)".to_string());
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

/// Render a flame graph PNG from call events (default filters).
pub fn to_png_flamegraph(events: &[CallEvent], title: &str) -> Result<Vec<u8>, String> {
    to_png_flamegraph_filtered(events, title, &FilterOptions::default())
}

/// Render a flame graph PNG with filter options applied.
pub fn to_png_flamegraph_filtered(
    events: &[CallEvent],
    title: &str,
    opts: &FilterOptions,
) -> Result<Vec<u8>, String> {
    // Use a wide canvas (2400px) so labels fit on narrow bars.
    let svg = render_svg(events, title, Some(2400), opts)?;
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

/// Wrap an SVG in an HTML page for guaranteed browser rendering (default filters).
pub fn to_html_flamegraph(events: &[CallEvent], title: &str) -> Result<String, String> {
    to_html_flamegraph_filtered(events, title, &FilterOptions::default())
}

/// Wrap an SVG in an HTML page with filter options applied.
pub fn to_html_flamegraph_filtered(
    events: &[CallEvent],
    title: &str,
    opts: &FilterOptions,
) -> Result<String, String> {
    let svg = to_svg_flamegraph_filtered(events, title, opts)?;
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
