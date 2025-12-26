//! Extractor trait definition.

use async_trait::async_trait;
use futures::stream::Stream;
use std::pin::Pin;

use crate::config::SourceConfig;
use crate::core::Record;
use crate::error::{ConfigError, ExtractError};

/// Type alias for async record stream.
pub type RecordStream = Pin<Box<dyn Stream<Item = Result<Record, ExtractError>> + Send>>;

/// Trait for data extractors.
///
/// Extractors read data from various sources (CSV, Excel, REST APIs) and convert
/// them into a stream of `Record` objects.
#[async_trait]
pub trait Extractor: Send + Sync {
    /// Unique identifier for this extractor.
    fn name(&self) -> &'static str;

    /// Supported source types (e.g., ["csv", "tsv"]).
    fn supported_types(&self) -> &[&'static str];

    /// Validate source configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The source configuration to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, otherwise `ConfigError`.
    fn validate_config(&self, config: &SourceConfig) -> Result<(), ConfigError>;

    /// Extract records as an async stream.
    ///
    /// # Arguments
    ///
    /// * `config` - The source configuration
    ///
    /// # Returns
    ///
    /// A stream of `Record` items or `ExtractError`.
    ///
    /// # Errors
    ///
    /// Returns `ExtractError` if extraction fails.
    async fn extract(&self, config: &SourceConfig) -> Result<RecordStream, ExtractError>;

    /// Estimate total record count (for progress tracking).
    ///
    /// # Arguments
    ///
    /// * `config` - The source configuration
    ///
    /// # Returns
    ///
    /// Optional estimated count, or `None` if unknown.
    ///
    /// # Errors
    ///
    /// Returns `ExtractError` if estimation fails.
    async fn estimate_count(&self, config: &SourceConfig) -> Result<Option<u64>, ExtractError>;
}
