//! trace - CLI for analyzing pytest coverage traces
//!
//! This is a thin CLI layer that delegates to the trace_analyzer library.
//! It handles argument parsing, output formatting, and error presentation.

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

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
    },

    /// Search scenario descriptions
    Search {
        /// Search query
        query: String,
    },

    /// Get full coverage context for a scenario
    Context {
        /// Scenario ID (pytest node ID)
        scenario_id: String,
    },

    /// Find scenarios that cover a file or line
    Affected {
        /// File path, optionally with line number (e.g., "src/auth.py" or "src/auth.py:25")
        target: String,
    },

    /// Run a scenario with coverage collection
    Run {
        /// Scenario ID to run
        scenario_id: String,
    },

    /// Start MCP server mode
    Mcp,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build {
            coverage,
            scenarios,
            output,
        } => cmd_build(&coverage, &scenarios, &output),
        Commands::List { behavior, errors } => cmd_list(behavior.as_deref(), errors),
        Commands::Search { query } => cmd_search(&query),
        Commands::Context { scenario_id } => cmd_context(&scenario_id),
        Commands::Affected { target } => cmd_affected(&target),
        Commands::Run { scenario_id } => cmd_run(&scenario_id),
        Commands::Mcp => cmd_mcp(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn cmd_build(coverage: &Path, scenarios: &Path, output: &Path) -> anyhow::Result<()> {
    use trace_analyzer::index::IndexBuilder;

    // Load and build index
    let builder = IndexBuilder::load(coverage, scenarios)?;
    let result = builder.build(output)?;

    // Print summary
    println!("Parsed {} test contexts", result.scenarios_with_coverage);
    println!("Parsed {} scenarios", result.scenarios_imported);
    println!(
        "Built index with {} coverage entries",
        result.coverage_entries
    );

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

fn cmd_list(_behavior: Option<&str>, _errors: bool) -> anyhow::Result<()> {
    println!("List command not yet implemented");
    Ok(())
}

fn cmd_search(_query: &str) -> anyhow::Result<()> {
    println!("Search command not yet implemented");
    Ok(())
}

fn cmd_context(_scenario_id: &str) -> anyhow::Result<()> {
    println!("Context command not yet implemented");
    Ok(())
}

fn cmd_affected(_target: &str) -> anyhow::Result<()> {
    println!("Affected command not yet implemented");
    Ok(())
}

fn cmd_run(_scenario_id: &str) -> anyhow::Result<()> {
    println!("Run command not yet implemented");
    Ok(())
}

fn cmd_mcp() -> anyhow::Result<()> {
    println!("MCP server not yet implemented");
    Ok(())
}
