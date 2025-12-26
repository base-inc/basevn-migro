//! Checkpointing for resume capability.

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

/// Checkpoint data for resuming interrupted jobs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Checkpoint {
    /// Job identifier.
    pub job_id: String,

    /// Last successfully processed record offset/index.
    pub last_offset: usize,

    /// Total records processed so far.
    pub total_processed: usize,

    /// Successful records count.
    pub success_count: usize,

    /// Failed records count.
    pub failed_count: usize,

    /// Record IDs that failed (for retry).
    pub failed_record_ids: Vec<String>,

    /// Timestamp of checkpoint (ISO 8601).
    pub timestamp: String,
}

impl Checkpoint {
    /// Creates a new checkpoint.
    pub fn new(job_id: String) -> Self {
        Self {
            job_id,
            last_offset: 0,
            total_processed: 0,
            success_count: 0,
            failed_count: 0,
            failed_record_ids: Vec::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Updates checkpoint with batch results.
    pub fn update(
        &mut self,
        offset: usize,
        success: usize,
        failed: usize,
        failed_ids: Vec<String>,
    ) {
        self.last_offset = offset;
        self.total_processed += success + failed;
        self.success_count += success;
        self.failed_count += failed;
        self.failed_record_ids.extend(failed_ids);
        self.timestamp = chrono::Utc::now().to_rfc3339();
    }
}

/// Manager for checkpoint operations with atomic writes.
pub struct CheckpointManager {
    checkpoint_path: PathBuf,
}

impl CheckpointManager {
    /// Creates a new checkpoint manager.
    ///
    /// # Arguments
    ///
    /// * `checkpoint_path` - Path to the checkpoint file
    pub fn new<P: AsRef<Path>>(checkpoint_path: P) -> Self {
        Self {
            checkpoint_path: checkpoint_path.as_ref().to_path_buf(),
        }
    }

    /// Loads checkpoint from file if it exists.
    ///
    /// # Returns
    ///
    /// The checkpoint if file exists and is valid, otherwise `None`.
    pub fn load(&self) -> std::io::Result<Option<Checkpoint>> {
        if !self.checkpoint_path.exists() {
            return Ok(None);
        }

        let file = File::open(&self.checkpoint_path)?;
        let reader = BufReader::new(file);
        let checkpoint: Checkpoint = serde_json::from_reader(reader)?;

        Ok(Some(checkpoint))
    }

    /// Saves checkpoint with atomic write (temp file + rename).
    ///
    /// # Arguments
    ///
    /// * `checkpoint` - The checkpoint to save
    ///
    /// # Errors
    ///
    /// Returns error if write or rename fails.
    pub fn save(&self, checkpoint: &Checkpoint) -> std::io::Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = self.checkpoint_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write to temporary file
        let temp_path = self.checkpoint_path.with_extension("tmp");
        let temp_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)?;

        let mut writer = BufWriter::new(temp_file);
        serde_json::to_writer_pretty(&mut writer, checkpoint)?;
        writer.flush()?;

        // Sync to disk
        writer.get_ref().sync_all()?;

        // Atomic rename
        std::fs::rename(&temp_path, &self.checkpoint_path)?;

        Ok(())
    }

    /// Deletes the checkpoint file.
    pub fn delete(&self) -> std::io::Result<()> {
        if self.checkpoint_path.exists() {
            std::fs::remove_file(&self.checkpoint_path)?;
        }
        Ok(())
    }

    /// Returns the checkpoint file path.
    pub fn path(&self) -> &Path {
        &self.checkpoint_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_creation() {
        let checkpoint = Checkpoint::new("test-job".to_string());
        assert_eq!(checkpoint.job_id, "test-job");
        assert_eq!(checkpoint.last_offset, 0);
        assert_eq!(checkpoint.total_processed, 0);
        assert_eq!(checkpoint.success_count, 0);
        assert_eq!(checkpoint.failed_count, 0);
        assert!(checkpoint.failed_record_ids.is_empty());
    }

    #[test]
    fn test_checkpoint_update() {
        let mut checkpoint = Checkpoint::new("test-job".to_string());

        checkpoint.update(10, 8, 2, vec!["rec_1".to_string(), "rec_2".to_string()]);

        assert_eq!(checkpoint.last_offset, 10);
        assert_eq!(checkpoint.total_processed, 10);
        assert_eq!(checkpoint.success_count, 8);
        assert_eq!(checkpoint.failed_count, 2);
        assert_eq!(checkpoint.failed_record_ids.len(), 2);

        // Update again
        checkpoint.update(20, 9, 1, vec!["rec_3".to_string()]);

        assert_eq!(checkpoint.last_offset, 20);
        assert_eq!(checkpoint.total_processed, 20);
        assert_eq!(checkpoint.success_count, 17);
        assert_eq!(checkpoint.failed_count, 3);
        assert_eq!(checkpoint.failed_record_ids.len(), 3);
    }

    #[test]
    fn test_checkpoint_manager_save_and_load() {
        let temp_dir = std::env::temp_dir();
        let checkpoint_path = temp_dir.join("test_checkpoint_save_load.json");

        // Clean up if exists
        let _ = std::fs::remove_file(&checkpoint_path);

        let manager = CheckpointManager::new(&checkpoint_path);

        // Create and save checkpoint
        let mut checkpoint = Checkpoint::new("test-job".to_string());
        checkpoint.update(100, 95, 5, vec!["rec_1".to_string()]);

        manager.save(&checkpoint).unwrap();
        assert!(checkpoint_path.exists());

        // Load and verify
        let loaded = manager.load().unwrap().unwrap();
        assert_eq!(loaded.job_id, "test-job");
        assert_eq!(loaded.last_offset, 100);
        assert_eq!(loaded.total_processed, 100);
        assert_eq!(loaded.success_count, 95);
        assert_eq!(loaded.failed_count, 5);
        assert_eq!(loaded.failed_record_ids, vec!["rec_1".to_string()]);

        // Clean up
        let _ = std::fs::remove_file(&checkpoint_path);
    }

    #[test]
    fn test_checkpoint_manager_load_nonexistent() {
        let temp_dir = std::env::temp_dir();
        let checkpoint_path = temp_dir.join("test_checkpoint_nonexistent.json");

        // Ensure it doesn't exist
        let _ = std::fs::remove_file(&checkpoint_path);

        let manager = CheckpointManager::new(&checkpoint_path);
        let loaded = manager.load().unwrap();

        assert!(loaded.is_none());
    }

    #[test]
    fn test_checkpoint_manager_delete() {
        let temp_dir = std::env::temp_dir();
        let checkpoint_path = temp_dir.join("test_checkpoint_delete.json");

        // Clean up if exists
        let _ = std::fs::remove_file(&checkpoint_path);

        let manager = CheckpointManager::new(&checkpoint_path);

        // Create and save checkpoint
        let checkpoint = Checkpoint::new("test-job".to_string());
        manager.save(&checkpoint).unwrap();
        assert!(checkpoint_path.exists());

        // Delete
        manager.delete().unwrap();
        assert!(!checkpoint_path.exists());
    }

    #[test]
    fn test_checkpoint_manager_atomic_write() {
        let temp_dir = std::env::temp_dir();
        let checkpoint_path = temp_dir.join("test_checkpoint_atomic.json");

        // Clean up if exists
        let _ = std::fs::remove_file(&checkpoint_path);

        let manager = CheckpointManager::new(&checkpoint_path);

        // Save first checkpoint
        let checkpoint1 = Checkpoint::new("job1".to_string());
        manager.save(&checkpoint1).unwrap();

        // Save second checkpoint (should overwrite atomically)
        let mut checkpoint2 = Checkpoint::new("job2".to_string());
        checkpoint2.update(50, 45, 5, vec![]);
        manager.save(&checkpoint2).unwrap();

        // Load and verify it's the second checkpoint
        let loaded = manager.load().unwrap().unwrap();
        assert_eq!(loaded.job_id, "job2");
        assert_eq!(loaded.last_offset, 50);

        // Temporary file should not exist
        let temp_path = checkpoint_path.with_extension("tmp");
        assert!(!temp_path.exists());

        // Clean up
        let _ = std::fs::remove_file(&checkpoint_path);
    }
}
