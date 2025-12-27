//! Status command handler.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::operations::checkpoint::CheckpointManager;

/// Runs the status command.
///
/// # Arguments
///
/// * `job_id` - Optional job ID to check
/// * `checkpoint_path` - Optional path to checkpoint file
pub async fn run_status(job_id: Option<String>, checkpoint_path: Option<PathBuf>) -> Result<()> {
    // Determine checkpoint file path
    let path = if let Some(path) = checkpoint_path {
        // User provided explicit checkpoint path
        tracing::info!("Reading checkpoint from: {}", path.display());
        path
    } else if let Some(id) = job_id {
        // Look up checkpoint file by job ID
        tracing::info!("Checking status for job ID: {}", id);
        PathBuf::from(format!(".basevn-migro/checkpoints/{}.json", id))
    } else {
        // No job ID or path provided - list available checkpoints
        return list_checkpoints().await;
    };

    // Load checkpoint
    let checkpoint_manager = CheckpointManager::new(&path);
    let checkpoint = checkpoint_manager
        .load()
        .context("Failed to load checkpoint file")?;

    if let Some(checkpoint) = checkpoint {
        display_checkpoint(&checkpoint, &path);
    } else {
        println!("✗ No checkpoint found at: {}", path.display());
        println!("\nTo check status, provide either:");
        println!("  --job-id <ID>           Check checkpoint for specific job");
        println!("  --checkpoint <PATH>     Read checkpoint from custom path");
        println!("\nOr run without arguments to list all checkpoints.");
    }

    Ok(())
}

/// Displays checkpoint information in a user-friendly format.
fn display_checkpoint(checkpoint: &crate::operations::checkpoint::Checkpoint, path: &Path) {
    println!("\n┌─────────────────────────────────────────────────────────┐");
    println!("│               Migration Job Status                     │");
    println!("└─────────────────────────────────────────────────────────┘");

    println!("\n📋 Job Information:");
    println!("  Job ID:        {}", checkpoint.job_id);
    println!("  Checkpoint:    {}", path.display());
    println!("  Last Updated:  {}", checkpoint.timestamp);

    println!("\n📊 Progress:");
    println!("  Total Processed: {}", checkpoint.total_processed);
    println!("  Last Offset:     {}", checkpoint.last_offset);

    let success_rate = if checkpoint.total_processed > 0 {
        (checkpoint.success_count as f64 / checkpoint.total_processed as f64) * 100.0
    } else {
        0.0
    };

    println!("\n✅ Success:");
    println!("  Successful:    {} records", checkpoint.success_count);
    println!("  Success Rate:  {:.1}%", success_rate);

    println!("\n❌ Failures:");
    println!("  Failed:        {} records", checkpoint.failed_count);

    if !checkpoint.failed_record_ids.is_empty() {
        println!("\n  Failed Record IDs:");
        let display_limit = 10;
        for (i, id) in checkpoint.failed_record_ids.iter().take(display_limit).enumerate() {
            println!("    {}. {}", i + 1, id);
        }
        if checkpoint.failed_record_ids.len() > display_limit {
            println!("    ... and {} more", checkpoint.failed_record_ids.len() - display_limit);
        }
    }

    println!("\n💡 Next Steps:");
    if checkpoint.failed_count > 0 {
        println!("  • Resume migration with: migro migrate --config <config.yaml> --resume");
        println!("  • Review audit log for detailed error information");
        println!("  • Check failed record IDs in source data");
    } else {
        println!("  • All records processed successfully!");
        println!("  • Run migration again without --resume to start fresh");
    }
    println!();
}

/// Lists all available checkpoint files in the default directory.
async fn list_checkpoints() -> Result<()> {
    let checkpoint_dir = PathBuf::from(".basevn-migro/checkpoints");

    if !checkpoint_dir.exists() {
        println!("✗ No checkpoints found");
        println!("\nCheckpoint directory does not exist: {}", checkpoint_dir.display());
        println!("\nCheckpoints are created when you run:");
        println!("  migro migrate --config <config.yaml>");
        return Ok(());
    }

    // Read checkpoint directory
    let entries = std::fs::read_dir(&checkpoint_dir)
        .context("Failed to read checkpoint directory")?;

    let mut checkpoints = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip non-JSON files
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        // Try to load checkpoint
        let checkpoint_manager = CheckpointManager::new(&path);
        if let Ok(Some(checkpoint)) = checkpoint_manager.load() {
            checkpoints.push((path, checkpoint));
        }
    }

    if checkpoints.is_empty() {
        println!("✗ No valid checkpoints found in: {}", checkpoint_dir.display());
        return Ok(());
    }

    // Sort by timestamp (most recent first)
    checkpoints.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));

    println!("\n┌─────────────────────────────────────────────────────────┐");
    println!("│            Available Migration Checkpoints              │");
    println!("└─────────────────────────────────────────────────────────┘\n");

    for (path, checkpoint) in &checkpoints {
        let success_rate = if checkpoint.total_processed > 0 {
            (checkpoint.success_count as f64 / checkpoint.total_processed as f64) * 100.0
        } else {
            0.0
        };

        println!("📦 Job ID: {}", checkpoint.job_id);
        println!("   File:      {}", path.file_name().unwrap().to_string_lossy());
        println!("   Progress:  {} / {} records ({:.1}% success)",
                 checkpoint.success_count,
                 checkpoint.total_processed,
                 success_rate);
        println!("   Failed:    {} records", checkpoint.failed_count);
        println!("   Updated:   {}", checkpoint.timestamp);
        println!();
    }

    println!("💡 To view detailed status:");
    println!("   migro status --job-id <JOB_ID>");
    println!();

    Ok(())
}
