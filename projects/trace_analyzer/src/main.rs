use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "trace_analyzer")]
#[command(about = "Trace analyzer - CLI and MCP server for analyzing test traces", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as MCP server
    Mcp,
    /// Analyze a trace file
    Analyze {
        /// Path to trace file
        #[arg(value_name = "FILE")]
        file: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Mcp => {
            println!("Starting MCP server...");
            // TODO: Implement MCP server
        }
        Commands::Analyze { file } => {
            println!("Analyzing trace file: {}", file);
            // TODO: Implement trace analysis
        }
    }
}
