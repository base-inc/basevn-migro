//! basevn-migro CLI entry point.

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use basevn_migro::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose)?;

    // Execute command
    match cli.command {
        Commands::Migrate { config, resume } => {
            basevn_migro::cli::commands::run_migrate(&config, resume).await?;
        }
        Commands::Validate { config } => {
            basevn_migro::cli::commands::run_validate(&config).await?;
        }
        Commands::Status { job_id, checkpoint } => {
            basevn_migro::cli::commands::run_status(job_id, checkpoint).await?;
        }
        Commands::Update { check } => {
            basevn_migro::cli::commands::run_update(check).await?;
        }
    }

    Ok(())
}

/// Initializes the tracing-based logging infrastructure.
///
/// Sets up structured logging with configurable log levels via RUST_LOG environment variable.
/// In verbose mode, defaults to debug level; otherwise defaults to info level.
fn init_logging(verbose: bool) -> Result<()> {
    let default_level = if verbose { "debug" } else { "info" };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("basevn_migro={}", default_level)));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    Ok(())
}
