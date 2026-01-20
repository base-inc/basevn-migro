//! Extractor registry for dynamic dispatch.

use std::collections::HashMap;
use std::sync::Arc;

use crate::config::SourceConfig;
use crate::error::{ConfigError, ExtractError};

use super::traits::{Extractor, RecordStream};
use super::{CsvExtractor, ExcelExtractor, RestApiExtractor};

/// Registry of available extractors.
///
/// Provides dynamic dispatch to the appropriate extractor based on source type.
pub struct ExtractorRegistry {
    extractors: HashMap<String, Arc<dyn Extractor>>,
}

impl ExtractorRegistry {
    /// Creates a new registry with all built-in extractors.
    pub fn new() -> Self {
        let mut registry = Self {
            extractors: HashMap::new(),
        };

        // Register built-in extractors
        registry.register(Arc::new(CsvExtractor::new()));
        registry.register(Arc::new(ExcelExtractor::new()));
        registry.register(Arc::new(RestApiExtractor::new()));

        registry
    }

    /// Registers an extractor.
    ///
    /// # Arguments
    ///
    /// * `extractor` - The extractor to register
    pub fn register(&mut self, extractor: Arc<dyn Extractor>) {
        for type_name in extractor.supported_types() {
            self.extractors
                .insert(type_name.to_string(), Arc::clone(&extractor));
        }
    }

    /// Gets an extractor for the given source type.
    ///
    /// # Arguments
    ///
    /// * `source_type` - The source type (e.g., "csv", "excel", "rest_api")
    ///
    /// # Returns
    ///
    /// The appropriate extractor, or an error if not found.
    pub fn get(&self, source_type: &str) -> Result<Arc<dyn Extractor>, ConfigError> {
        self.extractors.get(source_type).cloned().ok_or_else(|| {
            ConfigError::Validation(format!("Unsupported source type: {}", source_type))
        })
    }

    /// Gets an extractor for the given source configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The source configuration
    ///
    /// # Returns
    ///
    /// The appropriate extractor based on config type.
    pub fn get_for_config(&self, config: &SourceConfig) -> Result<Arc<dyn Extractor>, ConfigError> {
        let source_type = match config {
            SourceConfig::Csv { .. } => "csv",
            SourceConfig::Excel { .. } => "excel",
            SourceConfig::RestApi { .. } => "rest_api",
        };

        self.get(source_type)
    }

    /// Validates a source configuration using the appropriate extractor.
    ///
    /// # Arguments
    ///
    /// * `config` - The source configuration to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, otherwise `ConfigError`.
    pub fn validate_config(&self, config: &SourceConfig) -> Result<(), ConfigError> {
        let extractor = self.get_for_config(config)?;
        extractor.validate_config(config)
    }

    /// Extracts records using the appropriate extractor.
    ///
    /// # Arguments
    ///
    /// * `config` - The source configuration
    ///
    /// # Returns
    ///
    /// A stream of records from the source.
    pub async fn extract(&self, config: &SourceConfig) -> Result<RecordStream, ExtractError> {
        let extractor = self.get_for_config(config).map_err(|e| {
            ExtractError::UnsupportedType(format!("Failed to get extractor: {}", e))
        })?;

        extractor.extract(config).await
    }

    /// Estimates the record count for a source.
    ///
    /// # Arguments
    ///
    /// * `config` - The source configuration
    ///
    /// # Returns
    ///
    /// Optional estimated count.
    pub async fn estimate_count(&self, config: &SourceConfig) -> Result<Option<u64>, ExtractError> {
        let extractor = self.get_for_config(config).map_err(|e| {
            ExtractError::UnsupportedType(format!("Failed to get extractor: {}", e))
        })?;

        extractor.estimate_count(config).await
    }

    /// Lists all supported source types.
    pub fn supported_types(&self) -> Vec<String> {
        self.extractors.keys().cloned().collect()
    }
}

impl Default for ExtractorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ExtractorRegistry::new();
        let types = registry.supported_types();

        assert!(types.contains(&"csv".to_string()));
        assert!(types.contains(&"excel".to_string()));
        assert!(types.contains(&"rest_api".to_string()));
    }

    #[test]
    fn test_get_extractor_by_type() {
        let registry = ExtractorRegistry::new();

        let csv_extractor = registry.get("csv").unwrap();
        assert_eq!(csv_extractor.name(), "csv");

        let excel_extractor = registry.get("excel").unwrap();
        assert_eq!(excel_extractor.name(), "excel");

        let api_extractor = registry.get("rest_api").unwrap();
        assert_eq!(api_extractor.name(), "rest_api");
    }

    #[test]
    fn test_get_extractor_unsupported_type() {
        let registry = ExtractorRegistry::new();
        let result = registry.get("unsupported");

        assert!(result.is_err());
    }

    #[test]
    fn test_get_for_config() {
        let registry = ExtractorRegistry::new();

        let csv_config = SourceConfig::Csv {
            path: "test.csv".to_string(),
            csv: None,
        };
        let extractor = registry.get_for_config(&csv_config).unwrap();
        assert_eq!(extractor.name(), "csv");

        let excel_config = SourceConfig::Excel {
            path: "test.xlsx".to_string(),
            excel: None,
        };
        let extractor = registry.get_for_config(&excel_config).unwrap();
        assert_eq!(extractor.name(), "excel");
    }
}
