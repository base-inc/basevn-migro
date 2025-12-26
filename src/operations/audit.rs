//! Audit logging for ETL operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

/// Audit event types for ETL operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditEvent {
    /// Job started event.
    JobStart {
        job_id: String,
        timestamp: DateTime<Utc>,
        source_type: String,
        target_type: String,
    },

    /// Record processing success event.
    RecordSuccess {
        job_id: String,
        timestamp: DateTime<Utc>,
        record_index: usize,
        record_id: Option<String>,
    },

    /// Record processing failure event.
    RecordFailure {
        job_id: String,
        timestamp: DateTime<Utc>,
        record_index: usize,
        record_id: Option<String>,
        error: String,
    },

    /// Batch completion event.
    BatchComplete {
        job_id: String,
        timestamp: DateTime<Utc>,
        batch_number: usize,
        success_count: usize,
        failure_count: usize,
    },

    /// Job completion event.
    JobComplete {
        job_id: String,
        timestamp: DateTime<Utc>,
        total_processed: usize,
        total_success: usize,
        total_failed: usize,
        duration_secs: u64,
    },

    /// Job error event.
    JobError {
        job_id: String,
        timestamp: DateTime<Utc>,
        error: String,
    },
}

/// Audit logger that writes events to JSONL format.
///
/// Each event is written as a single JSON line for easy parsing with jq/grep.
pub struct AuditLogger {
    writer: BufWriter<File>,
    log_path: PathBuf,
}

impl AuditLogger {
    /// Creates a new audit logger writing to the specified file.
    ///
    /// # Arguments
    ///
    /// * `log_path` - Path to the audit log file (will be created/appended)
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be opened or created.
    pub fn new<P: AsRef<Path>>(log_path: P) -> std::io::Result<Self> {
        let log_path = log_path.as_ref().to_path_buf();

        // Create parent directories if they don't exist
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        let writer = BufWriter::new(file);

        Ok(Self { writer, log_path })
    }

    /// Logs an audit event.
    ///
    /// # Arguments
    ///
    /// * `event` - The audit event to log
    ///
    /// # Errors
    ///
    /// Returns error if serialization or file write fails.
    pub fn log(&mut self, event: &AuditEvent) -> std::io::Result<()> {
        let json = serde_json::to_string(event)?;
        writeln!(self.writer, "{}", json)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Returns the path to the audit log file.
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }

    /// Flushes any buffered events to disk.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;

    #[test]
    fn test_audit_logger_creation() {
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join("test_audit_creation.jsonl");

        // Clean up if exists
        let _ = std::fs::remove_file(&log_path);

        let logger = AuditLogger::new(&log_path).unwrap();
        assert_eq!(logger.log_path(), log_path);
        assert!(log_path.exists());

        // Clean up
        let _ = std::fs::remove_file(&log_path);
    }

    #[test]
    fn test_audit_logger_log_job_start() {
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join("test_audit_job_start.jsonl");

        // Clean up if exists
        let _ = std::fs::remove_file(&log_path);

        let mut logger = AuditLogger::new(&log_path).unwrap();

        let event = AuditEvent::JobStart {
            job_id: "test-job".to_string(),
            timestamp: Utc::now(),
            source_type: "csv".to_string(),
            target_type: "basevn".to_string(),
        };

        logger.log(&event).unwrap();
        logger.flush().unwrap();

        // Read back and verify
        let file = File::open(&log_path).unwrap();
        let reader = std::io::BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();

        assert_eq!(lines.len(), 1);
        let parsed: AuditEvent = serde_json::from_str(&lines[0]).unwrap();
        assert!(matches!(parsed, AuditEvent::JobStart { .. }));

        // Clean up
        let _ = std::fs::remove_file(&log_path);
    }

    #[test]
    fn test_audit_logger_log_multiple_events() {
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join("test_audit_multiple.jsonl");

        // Clean up if exists
        let _ = std::fs::remove_file(&log_path);

        let mut logger = AuditLogger::new(&log_path).unwrap();

        let events = vec![
            AuditEvent::JobStart {
                job_id: "test-job".to_string(),
                timestamp: Utc::now(),
                source_type: "csv".to_string(),
                target_type: "basevn".to_string(),
            },
            AuditEvent::RecordSuccess {
                job_id: "test-job".to_string(),
                timestamp: Utc::now(),
                record_index: 0,
                record_id: Some("rec_123".to_string()),
            },
            AuditEvent::RecordFailure {
                job_id: "test-job".to_string(),
                timestamp: Utc::now(),
                record_index: 1,
                record_id: None,
                error: "Validation failed".to_string(),
            },
            AuditEvent::JobComplete {
                job_id: "test-job".to_string(),
                timestamp: Utc::now(),
                total_processed: 2,
                total_success: 1,
                total_failed: 1,
                duration_secs: 10,
            },
        ];

        for event in &events {
            logger.log(event).unwrap();
        }
        logger.flush().unwrap();

        // Read back and verify
        let file = File::open(&log_path).unwrap();
        let reader = std::io::BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();

        assert_eq!(lines.len(), 4);

        // Verify each event type
        let parsed_events: Vec<AuditEvent> = lines
            .iter()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();

        assert!(matches!(parsed_events[0], AuditEvent::JobStart { .. }));
        assert!(matches!(parsed_events[1], AuditEvent::RecordSuccess { .. }));
        assert!(matches!(parsed_events[2], AuditEvent::RecordFailure { .. }));
        assert!(matches!(parsed_events[3], AuditEvent::JobComplete { .. }));

        // Clean up
        let _ = std::fs::remove_file(&log_path);
    }

    #[test]
    fn test_audit_logger_append_mode() {
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join("test_audit_append.jsonl");

        // Clean up if exists
        let _ = std::fs::remove_file(&log_path);

        // First logger instance
        {
            let mut logger = AuditLogger::new(&log_path).unwrap();
            let event = AuditEvent::JobStart {
                job_id: "job1".to_string(),
                timestamp: Utc::now(),
                source_type: "csv".to_string(),
                target_type: "basevn".to_string(),
            };
            logger.log(&event).unwrap();
        }

        // Second logger instance (should append)
        {
            let mut logger = AuditLogger::new(&log_path).unwrap();
            let event = AuditEvent::JobStart {
                job_id: "job2".to_string(),
                timestamp: Utc::now(),
                source_type: "rest_api".to_string(),
                target_type: "basevn".to_string(),
            };
            logger.log(&event).unwrap();
        }

        // Read back and verify both events
        let file = File::open(&log_path).unwrap();
        let reader = std::io::BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();

        assert_eq!(lines.len(), 2);

        // Clean up
        let _ = std::fs::remove_file(&log_path);
    }
}
