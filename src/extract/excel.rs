//! Excel file extractor implementation.
//!
//! Note: Full Excel support is planned for a future phase. For now, please convert
//! Excel files to CSV format for extraction.

use async_trait::async_trait;

use crate::config::SourceConfig;
use crate::error::{ConfigError, ExtractError};

use super::traits::{Extractor, RecordStream};

/// Excel file extractor.
///
/// Extracts records from Excel files (.xlsx, .xls) using calamine.
///
/// **Note:** Full Excel support is under active development. For the MVP release,
/// please convert Excel files to CSV format for extraction.
pub struct ExcelExtractor;

impl ExcelExtractor {
    /// Creates a new Excel extractor.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExcelExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Extractor for ExcelExtractor {
    fn name(&self) -> &'static str {
        "excel"
    }

    fn supported_types(&self) -> &[&'static str] {
        &["excel", "xlsx", "xls"]
    }

    fn validate_config(&self, config: &SourceConfig) -> Result<(), ConfigError> {
        match config {
            SourceConfig::Excel { path, .. } => {
                if path.is_empty() {
                    return Err(ConfigError::Validation("Excel path cannot be empty".into()));
                }
                Ok(())
            }
            _ => Err(ConfigError::Validation(
                "Invalid config type for Excel extractor".into(),
            )),
        }
    }

    async fn extract(&self, _config: &SourceConfig) -> Result<RecordStream, ExtractError> {
        // TODO: Implement full Excel extraction post-MVP
        // Calamine integration requires careful API handling
        Err(ExtractError::UnsupportedType(
            "Excel extraction not yet implemented. Please convert to CSV format.".into(),
        ))
    }

    async fn estimate_count(&self, _config: &SourceConfig) -> Result<Option<u64>, ExtractError> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excel_extractor_creation() {
        let extractor = ExcelExtractor::new();
        assert_eq!(extractor.name(), "excel");
        assert!(extractor.supported_types().contains(&"xlsx"));
        assert!(extractor.supported_types().contains(&"xls"));
    }

    #[tokio::test]
    async fn test_excel_returns_not_implemented() {
        let extractor = ExcelExtractor::new();
        let config = SourceConfig::Excel {
            path: "test.xlsx".to_string(),
            excel: None,
        };

        let result = extractor.extract(&config).await;
        assert!(result.is_err());
    }
}
