//! Loader trait definition.

use async_trait::async_trait;

use crate::config::TargetConfig;
use crate::core::Record;
use crate::error::{ConfigError, LoadError};

/// Result of a batch load operation.
///
/// Tracks successful and failed records with detailed error information.
#[derive(Debug, Clone)]
pub struct LoadResult {
    /// Total records in the batch.
    pub total: usize,

    /// Successfully loaded records.
    pub success: usize,

    /// Failed records.
    pub failed: usize,

    /// Detailed failure information.
    pub failures: Vec<RecordFailure>,
}

/// Information about a failed record.
#[derive(Debug, Clone)]
pub struct RecordFailure {
    /// Record index in the batch.
    pub index: usize,

    /// Record identifier (if available).
    pub id: Option<String>,

    /// Error message.
    pub error: String,
}

impl LoadResult {
    /// Creates a new load result.
    pub fn new(total: usize) -> Self {
        Self {
            total,
            success: 0,
            failed: 0,
            failures: Vec::new(),
        }
    }

    /// Creates a successful result.
    pub fn success(total: usize) -> Self {
        Self {
            total,
            success: total,
            failed: 0,
            failures: Vec::new(),
        }
    }

    /// Adds a successful record.
    pub fn add_success(&mut self) {
        self.success += 1;
    }

    /// Adds a failed record.
    pub fn add_failure(&mut self, index: usize, id: Option<String>, error: String) {
        self.failed += 1;
        self.failures.push(RecordFailure { index, id, error });
    }

    /// Returns true if all records succeeded.
    pub fn is_complete_success(&self) -> bool {
        self.failed == 0 && self.success == self.total
    }

    /// Returns the success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.success as f64 / self.total as f64) * 100.0
    }
}

/// Trait for data loaders.
///
/// Loaders write records to target systems (databases, APIs, files).
/// They handle batching, rate limiting, retries, and error tracking.
#[async_trait]
pub trait Loader: Send + Sync {
    /// Unique identifier for this loader.
    fn name(&self) -> &'static str;

    /// Validate the target configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Target configuration to validate
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if configuration is invalid.
    fn validate_config(&self, config: &TargetConfig) -> Result<(), ConfigError>;

    /// Load a batch of records to the target.
    ///
    /// # Arguments
    ///
    /// * `records` - Batch of records to load
    /// * `config` - Target configuration
    ///
    /// # Returns
    ///
    /// `LoadResult` with success/failure tracking.
    ///
    /// # Errors
    ///
    /// Returns `LoadError` if the entire batch fails (network error, auth error, etc.).
    /// Individual record failures are tracked in `LoadResult.failures`.
    async fn load_batch(
        &self,
        records: Vec<Record>,
        config: &TargetConfig,
    ) -> Result<LoadResult, LoadError>;

    /// Initialize the loader (optional).
    ///
    /// Called once before loading begins. Use for connection pooling,
    /// authentication, etc.
    ///
    /// # Arguments
    ///
    /// * `config` - Target configuration
    ///
    /// # Errors
    ///
    /// Returns `LoadError` if initialization fails.
    async fn initialize(&self, _config: &TargetConfig) -> Result<(), LoadError> {
        Ok(())
    }

    /// Finalize the loader (optional).
    ///
    /// Called once after all loading completes. Use for cleanup,
    /// flushing buffers, closing connections, etc.
    ///
    /// # Errors
    ///
    /// Returns `LoadError` if finalization fails.
    async fn finalize(&self) -> Result<(), LoadError> {
        Ok(())
    }
}
