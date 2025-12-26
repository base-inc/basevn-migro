//! CLI argument definitions using clap.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// basevn-migro: Open-source ETL tool for migrating data to Base.vn platform.
#[derive(Debug, Parser)]
#[command(name = "migro")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Global verbose flag for detailed logging.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run a data migration job.
    Migrate {
        /// Path to the job configuration file (YAML or JSON).
        #[arg(short, long)]
        config: PathBuf,

        /// Resume from an existing checkpoint.
        #[arg(short, long)]
        resume: bool,
    },

    /// Validate a job configuration file.
    Validate {
        /// Path to the job configuration file to validate.
        #[arg(short, long)]
        config: PathBuf,
    },

    /// Check the status of a migration job.
    Status {
        /// Job ID to check status for.
        #[arg(short, long)]
        job_id: Option<String>,

        /// Path to checkpoint file.
        #[arg(short, long)]
        checkpoint: Option<PathBuf>,
    },

    /// Update migro to the latest version.
    Update {
        /// Check for updates without installing.
        #[arg(long)]
        check: bool,
    },
}
