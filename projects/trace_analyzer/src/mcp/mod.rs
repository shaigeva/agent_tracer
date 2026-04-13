//! MCP (Model Context Protocol) server implementation.
//!
//! This module exposes trace_analyzer query capabilities as MCP tools,
//! allowing AI agents to query scenario coverage data.

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, ErrorCode, ErrorData as McpError, ProtocolVersion, ServerCapabilities,
    ServerInfo,
};
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::call_trace;
use crate::diagram;
use crate::index::Index;
use crate::query;
use crate::run;

/// Request for scenario_search tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScenarioSearchRequest {
    /// Search query for scenario descriptions
    pub query: String,
}

/// Request for scenario_context tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScenarioContextRequest {
    /// Full scenario ID (e.g., 'tests/scenarios/test_auth.py::test_login')
    pub scenario_id: String,
}

/// Request for coverage_affected_file tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CoverageAffectedFileRequest {
    /// Source file path (e.g., 'src/auth.py')
    pub file: String,
    /// Include source code snippets for the matching lines. Saves a follow-up file read.
    #[serde(default)]
    pub with_snippets: bool,
    /// Return function names (from call traces) covering the matching lines.
    /// Requires the index to have been built with --call-traces.
    #[serde(default)]
    pub functions_only: bool,
}

/// Request for coverage_affected_line tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CoverageAffectedLineRequest {
    /// Source file path (e.g., 'src/auth.py')
    pub file: String,
    /// Line number
    pub line: u32,
    /// Include source code snippet for the line.
    #[serde(default)]
    pub with_snippets: bool,
    /// Return function names (from call traces) covering the line.
    #[serde(default)]
    pub functions_only: bool,
}

/// Request for scenario_run tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScenarioRunRequest {
    /// Full scenario ID (e.g., 'tests/scenarios/test_auth.py::test_login')
    pub scenario_id: String,
}

/// Request for diagram_scenario tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiagramScenarioRequest {
    /// Full scenario ID (e.g., 'tests/scenarios/test_auth.py::test_login')
    pub scenario_id: String,
}

/// Request for diagram_file tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiagramFileRequest {
    /// Source file path (e.g., 'src/auth.py'), optionally with line number (e.g., 'src/auth.py:25')
    pub file: String,
}

/// Request for flamegraph tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FlamegraphRequest {
    /// Full scenario ID (e.g., 'tests/scenarios/test_auth.py::test_login')
    pub scenario_id: String,
    /// Output format. For agents, 'summary' (low tokens, unique frames) or
    /// 'folded-compact' (collapsed prefixes) are recommended. Options:
    /// 'folded' | 'folded-compact' | 'summary' | 'mermaid' | 'svg' | 'html'.
    #[serde(default = "default_format")]
    pub format: String,
    /// Anchor frame pattern. When set, stacks are trimmed to start at the first
    /// frame matching this pattern. When omitted, the scenario's own test function
    /// is used automatically (so fixture setup is trimmed off).
    #[serde(default)]
    pub from: Option<String>,
    /// Emit the full trace tree including fixture setup/teardown. Overrides the
    /// default auto-anchor at the scenario's test function.
    #[serde(default)]
    pub include_fixtures: bool,
    /// Comma-separated glob patterns; keep only stacks containing a matching frame.
    /// Patterns: 'foo*' (prefix), '*foo' (suffix), 'foo' (substring).
    #[serde(default)]
    pub include: String,
    /// Comma-separated glob patterns; drop stacks containing a matching frame.
    #[serde(default)]
    pub exclude: String,
    /// Cap stack depth at N frames.
    #[serde(default)]
    pub max_depth: Option<u32>,
}

fn default_format() -> String {
    "folded".to_string()
}

/// MCP server for trace analyzer.
#[derive(Clone)]
pub struct TraceServer {
    index_dir: Arc<PathBuf>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl TraceServer {
    /// Create a new trace server with the given index directory.
    pub fn new(index_dir: PathBuf) -> Self {
        Self {
            index_dir: Arc::new(index_dir),
            tool_router: Self::tool_router(),
        }
    }

    /// Open the index, returning an MCP error if it fails.
    fn open_index(&self) -> Result<Index, McpError> {
        Index::open_readonly(&self.index_dir).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("Failed to open index: {}", e)),
            data: None,
        })
    }

    #[tool(
        description = "List all test scenarios. Returns JSON array of scenarios with id, description, behaviors, and outcome."
    )]
    async fn scenario_list(&self) -> Result<CallToolResult, McpError> {
        let index = self.open_index()?;

        let scenarios = query::list_scenarios(&index, None, false).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("Query failed: {}", e)),
            data: None,
        })?;

        let json = serde_json::to_string_pretty(&scenarios).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("JSON error: {}", e)),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(description = "List error scenarios only. Returns JSON array of error scenarios.")]
    async fn scenario_list_errors(&self) -> Result<CallToolResult, McpError> {
        let index = self.open_index()?;

        let scenarios = query::list_scenarios(&index, None, true).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("Query failed: {}", e)),
            data: None,
        })?;

        let json = serde_json::to_string_pretty(&scenarios).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("JSON error: {}", e)),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Search for test scenarios by description or documentation text. Query is a substring match."
    )]
    async fn scenario_search(
        &self,
        params: Parameters<ScenarioSearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        let index = self.open_index()?;

        let scenarios = query::search_scenarios(&index, &params.0.query).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("Search failed: {}", e)),
            data: None,
        })?;

        let json = serde_json::to_string_pretty(&scenarios).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("JSON error: {}", e)),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Get full coverage context for a specific scenario, including all files and lines covered. scenario_id is the full pytest node ID like 'tests/scenarios/test_auth.py::test_login'."
    )]
    async fn scenario_context(
        &self,
        params: Parameters<ScenarioContextRequest>,
    ) -> Result<CallToolResult, McpError> {
        let index = self.open_index()?;

        let context =
            query::get_scenario_context(&index, &params.0.scenario_id).map_err(|e| McpError {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::from(format!("Context query failed: {}", e)),
                data: None,
            })?;

        let json = serde_json::to_string_pretty(&context).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("JSON error: {}", e)),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Find scenarios that cover a specific file. Set with_snippets:true to include source code for matching lines (saves a follow-up file read). Set functions_only:true to add function names from call traces."
    )]
    async fn coverage_affected_file(
        &self,
        params: Parameters<CoverageAffectedFileRequest>,
    ) -> Result<CallToolResult, McpError> {
        let index = self.open_index()?;

        let affected =
            query::find_affected_scenarios(&index, &params.0.file, None).map_err(|e| McpError {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::from(format!("Affected query failed: {}", e)),
                data: None,
            })?;

        let enriched = query::enrich_affected(
            &index,
            affected,
            &params.0.file,
            params.0.with_snippets,
            params.0.functions_only,
        )
        .map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("Enrichment failed: {}", e)),
            data: None,
        })?;

        let json = serde_json::to_string_pretty(&enriched).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("JSON error: {}", e)),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Find scenarios that cover a specific line in a file. Set with_snippets:true to include source code (saves a file read). Set functions_only:true to add function names from call traces."
    )]
    async fn coverage_affected_line(
        &self,
        params: Parameters<CoverageAffectedLineRequest>,
    ) -> Result<CallToolResult, McpError> {
        let index = self.open_index()?;

        let affected = query::find_affected_scenarios(&index, &params.0.file, Some(params.0.line))
            .map_err(|e| McpError {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::from(format!("Affected query failed: {}", e)),
                data: None,
            })?;

        let enriched = query::enrich_affected(
            &index,
            affected,
            &params.0.file,
            params.0.with_snippets,
            params.0.functions_only,
        )
        .map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("Enrichment failed: {}", e)),
            data: None,
        })?;

        let json = serde_json::to_string_pretty(&enriched).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("JSON error: {}", e)),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Generate a mermaid diagram showing all files covered by a specific scenario. Returns JSON with a 'mermaid' field containing the diagram source."
    )]
    async fn diagram_scenario(
        &self,
        params: Parameters<DiagramScenarioRequest>,
    ) -> Result<CallToolResult, McpError> {
        let index = self.open_index()?;

        let output =
            diagram::diagram_for_scenario(&index, &params.0.scenario_id).map_err(|e| McpError {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::from(format!("Diagram generation failed: {}", e)),
                data: None,
            })?;

        let json = serde_json::to_string_pretty(&output).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("JSON error: {}", e)),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Generate a mermaid diagram showing all scenarios that cover a specific file. The file parameter can include a line number like 'src/auth.py:25'. Returns JSON with a 'mermaid' field containing the diagram source."
    )]
    async fn diagram_file(
        &self,
        params: Parameters<DiagramFileRequest>,
    ) -> Result<CallToolResult, McpError> {
        let index = self.open_index()?;

        let (path, line) = query::parse_target(&params.0.file);
        let output = diagram::diagram_for_file(&index, &path, line).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("Diagram generation failed: {}", e)),
            data: None,
        })?;

        let json = serde_json::to_string_pretty(&output).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("JSON error: {}", e)),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    #[tool(
        description = "Generate a flame graph or call chain for a scenario. Requires call trace data (build with --call-traces). Formats: 'summary' (JSON list of unique frames - recommended for agents), 'folded-compact' (collapsed stacks), 'folded' (full stacks), 'mermaid' (sequence diagram), 'svg' | 'html' (for humans). By default, fixture (conftest.py) frames are dropped; set include_fixtures:true to see them. Use include / exclude (comma-separated globs) and max_depth to scope the output."
    )]
    async fn flamegraph(
        &self,
        params: Parameters<FlamegraphRequest>,
    ) -> Result<CallToolResult, McpError> {
        let index = self.open_index()?;

        let events =
            query::get_call_trace(&index, &params.0.scenario_id).map_err(|e| McpError {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::from(format!("Query failed: {}", e)),
                data: None,
            })?;

        if events.is_empty() {
            return Err(McpError {
                code: ErrorCode::INTERNAL_ERROR,
                message: Cow::from(
                    "No call trace data. Build the index with --call-traces to enable flame graphs.",
                ),
                data: None,
            });
        }

        // Auto-anchor at the scenario's test function unless the user
        // explicitly asked for fixtures or provided --from.
        let anchor_function: Option<String> = if let Some(f) = params.0.from.clone() {
            Some(f)
        } else if !params.0.include_fixtures {
            query::get_scenario_context(&index, &params.0.scenario_id)
                .ok()
                .map(|ctx| ctx.scenario.function)
        } else {
            None
        };

        let opts = call_trace::FilterOptions {
            anchor_function,
            include_fixtures: params.0.include_fixtures,
            include_patterns: call_trace::parse_patterns(&params.0.include),
            exclude_patterns: call_trace::parse_patterns(&params.0.exclude),
            max_depth: params.0.max_depth,
        };

        let short_name = params
            .0
            .scenario_id
            .split("::")
            .last()
            .unwrap_or(&params.0.scenario_id);

        let output = match params.0.format.as_str() {
            "summary" => {
                let summary = call_trace::to_summary(&events, &opts);
                serde_json::to_string_pretty(&summary).unwrap_or_default()
            }
            "folded-compact" => call_trace::to_folded_compact(&events, &opts),
            "mermaid" => call_trace::to_mermaid_sequence_filtered(&events, short_name, &opts),
            "svg" => call_trace::to_svg_flamegraph_filtered(&events, short_name, &opts)
                .unwrap_or_else(|e| format!("Error: {}", e)),
            "html" => call_trace::to_html_flamegraph_filtered(&events, short_name, &opts)
                .unwrap_or_else(|e| format!("Error: {}", e)),
            _ => call_trace::to_folded_stacks_filtered(&events, &opts),
        };

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(
        description = "Run a specific test scenario with coverage collection. Returns test result with pass/fail status, exit code, and output. scenario_id is the full pytest node ID like 'tests/scenarios/test_auth.py::test_login'."
    )]
    async fn scenario_run(
        &self,
        params: Parameters<ScenarioRunRequest>,
    ) -> Result<CallToolResult, McpError> {
        let result = run::run_scenario(&params.0.scenario_id).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("Run failed: {}", e)),
            data: None,
        })?;

        let json = serde_json::to_string_pretty(&result).map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: Cow::from(format!("JSON error: {}", e)),
            data: None,
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

#[tool_handler]
impl ServerHandler for TraceServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: rmcp::model::Implementation::from_build_env(),
            instructions: Some(
                "Trace analyzer MCP server. Query test scenario coverage data \
                 to understand which code paths are covered by which tests. \
                 Use scenario_list to see all scenarios, scenario_search to find \
                 specific tests, scenario_context to get coverage details, \
                 coverage_affected_file/coverage_affected_line to find tests covering specific code, \
                 and scenario_run to execute a specific test with coverage collection."
                    .to_string(),
            ),
        }
    }
}

/// Run the MCP server on stdio.
pub async fn run_server(index_dir: PathBuf) -> anyhow::Result<()> {
    use rmcp::transport::stdio;
    use rmcp::ServiceExt;

    let server = TraceServer::new(index_dir);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_server_creation() {
        let temp_dir = TempDir::new().unwrap();
        let server = TraceServer::new(temp_dir.path().to_path_buf());

        // Verify the index_dir is set correctly
        assert_eq!(*server.index_dir, temp_dir.path().to_path_buf());
    }

    #[test]
    fn test_server_info() {
        let temp_dir = TempDir::new().unwrap();
        let server = TraceServer::new(temp_dir.path().to_path_buf());

        let info = server.get_info();

        // Verify server info
        assert!(info.instructions.is_some());
        assert!(info
            .instructions
            .as_ref()
            .unwrap()
            .contains("Trace analyzer"));
        assert_eq!(info.protocol_version, ProtocolVersion::LATEST);
    }

    #[test]
    fn test_open_index_missing() {
        let temp_dir = TempDir::new().unwrap();
        let server = TraceServer::new(temp_dir.path().join("nonexistent"));

        // Should fail because index doesn't exist
        let result = server.open_index();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.code, ErrorCode::INTERNAL_ERROR);
        assert!(err.message.contains("Failed to open index"));
    }

    #[test]
    fn test_open_index_success() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        // Create an index first
        let index = crate::index::Index::create(&index_dir).unwrap();
        drop(index);

        let server = TraceServer::new(index_dir);
        let result = server.open_index();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_scenario_list_with_empty_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        // Create an empty index
        let index = crate::index::Index::create(&index_dir).unwrap();
        drop(index);

        let server = TraceServer::new(index_dir);
        let result = server.scenario_list().await;

        assert!(result.is_ok());
        let call_result = result.unwrap();

        // Should return empty array as JSON
        assert!(!call_result.is_error.unwrap_or(false));
        assert!(!call_result.content.is_empty());
    }

    #[tokio::test]
    async fn test_scenario_list_errors_with_empty_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        // Create an empty index
        let index = crate::index::Index::create(&index_dir).unwrap();
        drop(index);

        let server = TraceServer::new(index_dir);
        let result = server.scenario_list_errors().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_scenario_search_with_empty_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        // Create an empty index
        let index = crate::index::Index::create(&index_dir).unwrap();
        drop(index);

        let server = TraceServer::new(index_dir);
        let params = Parameters(ScenarioSearchRequest {
            query: "test".to_string(),
        });
        let result = server.scenario_search(params).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_scenario_context_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        // Create an empty index
        let index = crate::index::Index::create(&index_dir).unwrap();
        drop(index);

        let server = TraceServer::new(index_dir);
        let params = Parameters(ScenarioContextRequest {
            scenario_id: "nonexistent::test".to_string(),
        });
        let result = server.scenario_context(params).await;

        // Should fail because scenario doesn't exist
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_coverage_affected_file_empty_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        // Create an empty index
        let index = crate::index::Index::create(&index_dir).unwrap();
        drop(index);

        let server = TraceServer::new(index_dir);
        let params = Parameters(CoverageAffectedFileRequest {
            file: "src/auth.py".to_string(),
            with_snippets: false,
            functions_only: false,
        });
        let result = server.coverage_affected_file(params).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coverage_affected_line_empty_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        // Create an empty index
        let index = crate::index::Index::create(&index_dir).unwrap();
        drop(index);

        let server = TraceServer::new(index_dir);
        let params = Parameters(CoverageAffectedLineRequest {
            file: "src/auth.py".to_string(),
            line: 25,
            with_snippets: false,
            functions_only: false,
        });
        let result = server.coverage_affected_line(params).await;

        assert!(result.is_ok());
    }
}
