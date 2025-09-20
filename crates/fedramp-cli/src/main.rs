// Modified: 2025-09-20

//! # FedRAMP CLI
//!
//! Command-line interface for FedRAMP compliance automation

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod commands;
mod config;
mod utils;

use commands::*;

#[derive(Parser)]
#[command(name = "fedramp")]
#[command(about = "FedRAMP Compliance Automation CLI")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "FedRAMP Compliance Team")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and convert documents
    Parse(parse::ParseArgs),
    
    /// Analyze compliance gaps
    Analyze(analyze::AnalyzeArgs),
    
    /// Generate reports
    Report(report::ReportArgs),
    
    /// Generate System Security Plans
    Ssp(ssp::SspArgs),
    
    /// Convert between frameworks
    Convert(convert::ConvertArgs),
    
    /// Validate documents and data
    Validate(validate::ValidateArgs),
    
    /// Initialize new project
    Init(init::InitArgs),
    
    /// Show version information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("FedRAMP CLI v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = config::Config::load(cli.config.as_deref())?;

    // Execute command
    match cli.command {
        Commands::Parse(args) => parse::execute(args, &config).await,
        Commands::Analyze(args) => analyze::execute(args, &config).await,
        Commands::Report(args) => report::execute(args, &config).await,
        Commands::Ssp(args) => ssp::execute(args, &config).await,
        Commands::Convert(args) => convert::execute(args, &config).await,
        Commands::Validate(args) => validate::execute(args, &config).await,
        Commands::Init(args) => init::execute(args, &config).await,
        Commands::Version => {
            println!("fedramp-cli {}", env!("CARGO_PKG_VERSION"));
            println!("fedramp-core {}", fedramp_core::VERSION);
            println!("OSCAL version {}", fedramp_core::OSCAL_VERSION);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert()
    }

    #[test]
    fn test_version_command() {
        let cli = Cli::parse_from(&["fedramp", "version"]);
        assert!(matches!(cli.command, Commands::Version));
    }
}
