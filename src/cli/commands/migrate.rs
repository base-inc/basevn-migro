//! Migrate command handler.

use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Utc;
use futures::StreamExt;

use crate::config::{load_config, SourceConfig, TargetConfig};
use crate::extract::registry::ExtractorRegistry;
use crate::load::registry::LoaderRegistry;
use crate::operations::audit::{AuditEvent, AuditLogger};
use crate::operations::checkpoint::{Checkpoint, CheckpointManager};
use crate::operations::progress::ProgressTracker;
use crate::transform::field_mapping::FieldMapper;
use crate::transform::traits::Transformer;

/// Runs the migrate command.
///
/// # Arguments
///
/// * `config_path` - Path to the job configuration file
/// * `resume` - Whether to resume from a checkpoint
pub async fn run_migrate(config_path: &Path, resume: bool) -> Result<()> {
    tracing::info!("Starting migration from config: {}", config_path.display());

    // Load configuration
    let config = load_config(config_path)?;
    tracing::info!("Loaded configuration for job: {}", config.job.id);

    // Initialize checkpoint manager
    let checkpoint_path = PathBuf::from(format!(".basevn-migro/checkpoints/{}.json", config.job.id));
    let checkpoint_manager = CheckpointManager::new(&checkpoint_path);

    // Initialize audit logger
    let audit_path = PathBuf::from(format!(".basevn-migro/logs/{}_audit.jsonl", config.job.id));
    let mut audit_logger = AuditLogger::new(&audit_path)?;

    // Load or create checkpoint
    let mut checkpoint = if resume {
        if let Some(existing) = checkpoint_manager.load()? {
            tracing::info!("Resuming from checkpoint: {} records processed", existing.total_processed);
            audit_logger.log(&AuditEvent::JobStart {
                job_id: config.job.id.clone(),
                timestamp: Utc::now(),
                source_type: "resume".to_string(),
                target_type: get_target_type(&config.target),
            })?;
            existing
        } else {
            tracing::warn!("Resume requested but no checkpoint found, starting fresh");
            Checkpoint::new(config.job.id.clone())
        }
    } else {
        Checkpoint::new(config.job.id.clone())
    };

    // Create extractor registry and get extractor for source
    let extractor_registry = ExtractorRegistry::default();
    let estimated_count = extractor_registry.estimate_count(&config.source).await?;

    // Create loader registry and get loader for target
    let loader_registry = LoaderRegistry::new();
    let loader = loader_registry.get_for_config(&config.target)?;
    loader.validate_config(&config.target)?;
    loader.initialize(&config.target).await?;

    // Create transformer from config.mapping
    let field_mapper = FieldMapper::new(config.mapping.clone());

    // Initialize progress tracker with estimated count
    let mut progress_tracker = ProgressTracker::new(estimated_count.unwrap_or(1000) as usize);

    // Log job start
    if !resume {
        audit_logger.log(&AuditEvent::JobStart {
            job_id: config.job.id.clone(),
            timestamp: Utc::now(),
            source_type: get_source_type(&config.source),
            target_type: get_target_type(&config.target),
        })?;
    }

    // Start extraction stream
    let mut stream = extractor_registry.extract(&config.source).await?;

    let batch_size = match &config.target {
        TargetConfig::BaseVn { basevn } => basevn.options.batch_size,
    };

    let mut current_batch = Vec::new();
    let mut record_index = checkpoint.last_offset;
    let mut batch_number = 0;

    // Stream processing loop
    while let Some(extract_result) = stream.next().await {
        let record = match extract_result {
            Ok(rec) => rec,
            Err(e) => {
                tracing::error!("Extraction error at record {}: {}", record_index, e);
                audit_logger.log(&AuditEvent::RecordFailure {
                    job_id: config.job.id.clone(),
                    timestamp: Utc::now(),
                    record_index,
                    record_id: None,
                    error: e.to_string(),
                })?;
                progress_tracker.inc_failure();
                record_index += 1;
                continue;
            }
        };

        // Transform record
        let transformed = match field_mapper.transform(record) {
            Ok(rec) => rec,
            Err(e) => {
                tracing::warn!("Transform error at record {}: {}", record_index, e);
                audit_logger.log(&AuditEvent::RecordFailure {
                    job_id: config.job.id.clone(),
                    timestamp: Utc::now(),
                    record_index,
                    record_id: None,
                    error: e.to_string(),
                })?;
                progress_tracker.inc_failure();
                record_index += 1;
                continue;
            }
        };

        current_batch.push(transformed);
        record_index += 1;

        // Load batch when full
        if current_batch.len() >= batch_size {
            batch_number += 1;
            let load_result = loader.load_batch(current_batch.clone(), &config.target).await?;

            // Update progress tracker
            progress_tracker.record_batch(load_result.success, load_result.failed);

            // Log batch completion
            audit_logger.log(&AuditEvent::BatchComplete {
                job_id: config.job.id.clone(),
                timestamp: Utc::now(),
                batch_number,
                success_count: load_result.success,
                failure_count: load_result.failed,
            })?;

            // Log individual failures
            for failure in &load_result.failures {
                audit_logger.log(&AuditEvent::RecordFailure {
                    job_id: config.job.id.clone(),
                    timestamp: Utc::now(),
                    record_index: checkpoint.last_offset + failure.index,
                    record_id: failure.id.clone(),
                    error: failure.error.clone(),
                })?;
            }

            // Update checkpoint
            let failed_ids: Vec<String> = load_result.failures.iter()
                .filter_map(|f| f.id.clone())
                .collect();
            checkpoint.update(record_index, load_result.success, load_result.failed, failed_ids);
            checkpoint_manager.save(&checkpoint)?;

            current_batch.clear();
        }
    }

    // Process remaining records in final batch
    if !current_batch.is_empty() {
        batch_number += 1;
        let load_result = loader.load_batch(current_batch.clone(), &config.target).await?;

        progress_tracker.record_batch(load_result.success, load_result.failed);

        audit_logger.log(&AuditEvent::BatchComplete {
            job_id: config.job.id.clone(),
            timestamp: Utc::now(),
            batch_number,
            success_count: load_result.success,
            failure_count: load_result.failed,
        })?;

        for failure in &load_result.failures {
            audit_logger.log(&AuditEvent::RecordFailure {
                job_id: config.job.id.clone(),
                timestamp: Utc::now(),
                record_index: checkpoint.last_offset + failure.index,
                record_id: failure.id.clone(),
                error: failure.error.clone(),
            })?;
        }

        let failed_ids: Vec<String> = load_result.failures.iter()
            .filter_map(|f| f.id.clone())
            .collect();
        checkpoint.update(record_index, load_result.success, load_result.failed, failed_ids);
        checkpoint_manager.save(&checkpoint)?;
    }

    // Finalize loader
    loader.finalize().await?;

    // Get final stats
    let stats = progress_tracker.stats();

    // Log job completion
    audit_logger.log(&AuditEvent::JobComplete {
        job_id: config.job.id.clone(),
        timestamp: Utc::now(),
        total_processed: stats.processed,
        total_success: stats.success,
        total_failed: stats.failed,
        duration_secs: stats.elapsed_secs,
    })?;
    audit_logger.flush()?;

    // Finish progress bar
    progress_tracker.finish();

    // Delete checkpoint on success
    if stats.failed == 0 {
        checkpoint_manager.delete()?;
        tracing::info!("Migration completed successfully, checkpoint deleted");
    } else {
        tracing::warn!("Migration completed with {} failures, checkpoint retained", stats.failed);
    }

    // Print summary
    println!("\n✓ Migration completed!");
    println!("  Total: {} | Success: {} | Failed: {}",
             stats.processed, stats.success, stats.failed);
    println!("  Success rate: {:.1}%", stats.success_rate);
    println!("  Duration: {}s", stats.elapsed_secs);
    if stats.failed > 0 {
        println!("  Checkpoint saved for resume");
    }

    Ok(())
}

/// Helper function to get source type string.
fn get_source_type(source: &SourceConfig) -> String {
    match source {
        SourceConfig::Csv { .. } => "csv".to_string(),
        SourceConfig::Excel { .. } => "excel".to_string(),
        SourceConfig::RestApi { .. } => "restapi".to_string(),
    }
}

/// Helper function to get target type string.
fn get_target_type(target: &TargetConfig) -> String {
    match target {
        TargetConfig::BaseVn { .. } => "basevn".to_string(),
    }
}
