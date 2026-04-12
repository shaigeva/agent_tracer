//! trace - CLI for analyzing pytest coverage traces
//!
//! This is a thin CLI layer that delegates to the trace_analyzer library.
//! It handles argument parsing, output formatting, and error presentation.

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

/// Default index directory.
const DEFAULT_INDEX_DIR: &str = ".trace-index";

#[derive(Parser)]
#[command(name = "trace")]
#[command(about = "Analyze pytest coverage traces for AI agent context", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build index from coverage data and scenario metadata
    Build {
        /// Path to .coverage SQLite file from pytest-cov
        #[arg(long)]
        coverage: PathBuf,

        /// Path to scenarios.json from pytest-tracer
        #[arg(long)]
        scenarios: PathBuf,

        /// Path to call_traces.json from pytest-tracer trace (optional)
        #[arg(long)]
        call_traces: Option<PathBuf>,

        /// Output directory for the index (default: .trace-index)
        #[arg(long, short, default_value = ".trace-index")]
        output: PathBuf,
    },

    /// List all scenarios
    List {
        /// Filter by behavior tag
        #[arg(long)]
        behavior: Option<String>,

        /// Show only error scenarios
        #[arg(long)]
        errors: bool,

        /// Index directory (default: .trace-index)
        #[arg(long, default_value = DEFAULT_INDEX_DIR)]
        index: PathBuf,
    },

    /// Search scenario descriptions
    Search {
        /// Search query
        query: String,

        /// Index directory (default: .trace-index)
        #[arg(long, default_value = DEFAULT_INDEX_DIR)]
        index: PathBuf,
    },

    /// Get full coverage context for a scenario
    Context {
        /// Scenario ID (pytest node ID)
        scenario_id: String,

        /// Index directory (default: .trace-index)
        #[arg(long, default_value = DEFAULT_INDEX_DIR)]
        index: PathBuf,
    },

    /// Find scenarios that cover a file or line
    Affected {
        /// File path, optionally with line number (e.g., "src/auth.py" or "src/auth.py:25")
        target: String,

        /// Index directory (default: .trace-index)
        #[arg(long, default_value = DEFAULT_INDEX_DIR)]
        index: PathBuf,
    },

    /// Run a scenario with coverage collection
    Run {
        /// Scenario ID to run
        scenario_id: String,
    },

    /// Generate flame graph or call-chain diagram from call traces
    Flamegraph {
        /// Scenario ID
        scenario_id: String,

        /// Output format: "folded" for folded stacks, "mermaid" for sequence diagram
        #[arg(long, default_value = "folded")]
        format: String,

        /// Index directory (default: .trace-index)
        #[arg(long, default_value = DEFAULT_INDEX_DIR)]
        index: PathBuf,
    },

    /// Generate a mermaid diagram
    Diagram {
        /// Scenario ID to diagram (shows all files covered by the scenario)
        scenario_id: Option<String>,

        /// File path to diagram (shows all scenarios covering the file)
        #[arg(long)]
        file: Option<String>,

        /// Index directory (default: .trace-index)
        #[arg(long, default_value = DEFAULT_INDEX_DIR)]
        index: PathBuf,
    },

    /// Start MCP server mode
    Mcp {
        /// Index directory (default: .trace-index)
        #[arg(long, default_value = DEFAULT_INDEX_DIR)]
        index: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build {
            coverage,
            scenarios,
            call_traces,
            output,
        } => cmd_build(&coverage, &scenarios, call_traces.as_deref(), &output),
        Commands::List {
            behavior,
            errors,
            index,
        } => cmd_list(behavior.as_deref(), errors, &index),
        Commands::Search { query, index } => cmd_search(&query, &index),
        Commands::Context { scenario_id, index } => cmd_context(&scenario_id, &index),
        Commands::Affected { target, index } => cmd_affected(&target, &index),
        Commands::Run { scenario_id } => cmd_run(&scenario_id),
        Commands::Flamegraph {
            scenario_id,
            format,
            index,
        } => cmd_flamegraph(&scenario_id, &format, &index),
        Commands::Diagram {
            scenario_id,
            file,
            index,
        } => cmd_diagram(scenario_id.as_deref(), file.as_deref(), &index),
        Commands::Mcp { index } => cmd_mcp(&index),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn cmd_build(
    coverage: &Path,
    scenarios: &Path,
    call_traces: Option<&Path>,
    output: &Path,
) -> anyhow::Result<()> {
    use trace_analyzer::index::IndexBuilder;

    // Load and build index
    let builder = IndexBuilder::load(coverage, scenarios, call_traces)?;
    let result = builder.build(output)?;

    // Print summary
    println!("Parsed {} test contexts", result.scenarios_with_coverage);
    println!("Parsed {} scenarios", result.scenarios_imported);
    println!(
        "Built index with {} coverage entries",
        result.coverage_entries
    );

    if result.call_trace_events > 0 {
        println!("Imported {} call trace events", result.call_trace_events);
    }

    if !result.scenarios_without_coverage.is_empty() {
        println!(
            "Warning: {} scenarios have no coverage data",
            result.scenarios_without_coverage.len()
        );
    }

    if !result.unmatched_contexts.is_empty() {
        println!(
            "Note: {} test contexts didn't match any scenario",
            result.unmatched_contexts.len()
        );
    }

    println!("Index written to {}", output.display());

    Ok(())
}

fn cmd_list(behavior: Option<&str>, errors: bool, index_dir: &Path) -> anyhow::Result<()> {
    use trace_analyzer::index::Index;
    use trace_analyzer::query;

    let index = Index::open_readonly(index_dir)?;
    let scenarios = query::list_scenarios(&index, behavior, errors)?;

    // Output as JSON
    println!("{}", serde_json::to_string_pretty(&scenarios)?);
    Ok(())
}

fn cmd_search(query_str: &str, index_dir: &Path) -> anyhow::Result<()> {
    use trace_analyzer::index::Index;
    use trace_analyzer::query;

    let index = Index::open_readonly(index_dir)?;
    let scenarios = query::search_scenarios(&index, query_str)?;

    // Output as JSON
    println!("{}", serde_json::to_string_pretty(&scenarios)?);
    Ok(())
}

fn cmd_context(scenario_id: &str, index_dir: &Path) -> anyhow::Result<()> {
    use trace_analyzer::index::Index;
    use trace_analyzer::query;

    let index = Index::open_readonly(index_dir)?;
    let context = query::get_scenario_context(&index, scenario_id)?;

    // Output as JSON
    println!("{}", serde_json::to_string_pretty(&context)?);
    Ok(())
}

fn cmd_affected(target: &str, index_dir: &Path) -> anyhow::Result<()> {
    use trace_analyzer::index::Index;
    use trace_analyzer::query;

    let index = Index::open_readonly(index_dir)?;
    let (file_path, line) = query::parse_target(target);
    let affected = query::find_affected_scenarios(&index, &file_path, line)?;

    // Output as JSON
    println!("{}", serde_json::to_string_pretty(&affected)?);
    Ok(())
}

fn cmd_run(scenario_id: &str) -> anyhow::Result<()> {
    use trace_analyzer::run;

    let result = run::run_scenario(scenario_id)?;

    // Output as JSON
    println!("{}", serde_json::to_string_pretty(&result)?);

    // Exit with test result code
    if !result.passed {
        std::process::exit(result.exit_code);
    }

    Ok(())
}

fn cmd_flamegraph(scenario_id: &str, format: &str, index_dir: &Path) -> anyhow::Result<()> {
    use trace_analyzer::call_trace;
    use trace_analyzer::index::Index;
    use trace_analyzer::query;

    let index = Index::open_readonly(index_dir)?;
    let events = query::get_call_trace(&index, scenario_id)?;

    if events.is_empty() {
        anyhow::bail!(
            "No call trace data for scenario '{}'. Did you build the index with --call-traces?",
            scenario_id
        );
    }

    match format {
        "folded" => {
            let folded = call_trace::to_folded_stacks(&events);
            print!("{}", folded);
        }
        "mermaid" => {
            let short_name = scenario_id.split("::").last().unwrap_or(scenario_id);
            let mermaid = call_trace::to_mermaid_sequence(&events, short_name);
            println!("{}", mermaid);
        }
        _ => {
            anyhow::bail!("Unknown format '{}'. Use 'folded' or 'mermaid'.", format);
        }
    }

    Ok(())
}

fn cmd_diagram(
    scenario_id: Option<&str>,
    file: Option<&str>,
    index_dir: &Path,
) -> anyhow::Result<()> {
    use trace_analyzer::diagram;
    use trace_analyzer::index::Index;
    use trace_analyzer::query;

    let index = Index::open_readonly(index_dir)?;

    let output = match (scenario_id, file) {
        (Some(id), _) => diagram::diagram_for_scenario(&index, id)?,
        (None, Some(file_path)) => {
            let (path, line) = query::parse_target(file_path);
            diagram::diagram_for_file(&index, &path, line)?
        }
        (None, None) => {
            anyhow::bail!("Provide either a scenario ID or --file <path>");
        }
    };

    // Output as JSON with mermaid field
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn cmd_mcp(index_dir: &Path) -> anyhow::Result<()> {
    // Run the async MCP server
    tokio::runtime::Runtime::new()?
        .block_on(async { trace_analyzer::mcp::run_server(index_dir.to_path_buf()).await })
}
