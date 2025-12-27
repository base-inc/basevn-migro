//! CSV file extractor implementation.

use async_trait::async_trait;
use csv::ReaderBuilder;
use futures::stream;
use std::path::{Path, PathBuf};

use crate::config::{CsvOptions, SourceConfig};
use crate::core::{Record, Value};
use crate::error::{ConfigError, ExtractError};

use super::traits::{Extractor, RecordStream};

/// CSV file extractor.
///
/// Extracts records from CSV files with configurable delimiters, encoding, and headers.
pub struct CsvExtractor;

impl CsvExtractor {
    /// Creates a new CSV extractor.
    pub fn new() -> Self {
        Self
    }

    /// Counts lines in a CSV file for progress estimation.
    async fn count_lines(path: &Path) -> Result<u64, ExtractError> {
        let content =
            tokio::fs::read_to_string(path)
                .await
                .map_err(|source| ExtractError::FileRead {
                    path: path.display().to_string(),
                    source,
                })?;

        let count = content.lines().count() as u64;
        // Subtract 1 for header if present
        Ok(count.saturating_sub(1))
    }
}

impl Default for CsvExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Extractor for CsvExtractor {
    fn name(&self) -> &'static str {
        "csv"
    }

    fn supported_types(&self) -> &[&'static str] {
        &["csv", "tsv"]
    }

    fn validate_config(&self, config: &SourceConfig) -> Result<(), ConfigError> {
        match config {
            SourceConfig::Csv { path, .. } => {
                if path.is_empty() {
                    return Err(ConfigError::Validation("CSV path cannot be empty".into()));
                }
                Ok(())
            }
            _ => Err(ConfigError::Validation(
                "Invalid config type for CSV extractor".into(),
            )),
        }
    }

    async fn extract(&self, config: &SourceConfig) -> Result<RecordStream, ExtractError> {
        let (path, csv_options) = match config {
            SourceConfig::Csv { path, csv } => (PathBuf::from(path), csv.clone()),
            _ => {
                return Err(ExtractError::UnsupportedType(
                    "CSV extractor requires CSV source config".into(),
                ))
            }
        };

        // Read file content
        let content =
            tokio::fs::read_to_string(&path)
                .await
                .map_err(|source| ExtractError::FileRead {
                    path: path.display().to_string(),
                    source,
                })?;

        // Parse CSV options
        let options = csv_options.unwrap_or_else(|| CsvOptions {
            delimiter: ",".to_string(),
            quote_char: "\"".to_string(),
            has_header: true,
            skip_rows: 0,
            encoding: "utf-8".to_string(),
        });

        // Build CSV reader
        let delimiter = options.delimiter.chars().next().unwrap_or(',') as u8;

        let quote = options.quote_char.chars().next().unwrap_or('"') as u8;

        let mut reader = ReaderBuilder::new()
            .delimiter(delimiter)
            .quote(quote)
            .has_headers(options.has_header)
            .from_reader(content.as_bytes());

        // Get headers
        let headers = if options.has_header {
            reader
                .headers()
                .map_err(|e| ExtractError::CsvParse {
                    line: 0,
                    message: e.to_string(),
                })?
                .iter()
                .map(|h| h.to_string())
                .collect::<Vec<_>>()
        } else {
            // Generate column names: col_0, col_1, etc.
            (0..reader.headers().map(|h| h.len()).unwrap_or(0))
                .map(|i| format!("col_{}", i))
                .collect()
        };

        // Convert records to stream
        let mut records = Vec::new();
        let mut line_number = if options.has_header { 1 } else { 0 };

        for result in reader.records().skip(options.skip_rows) {
            line_number += 1;

            let csv_record = result.map_err(|e| ExtractError::CsvParse {
                line: line_number,
                message: e.to_string(),
            })?;

            let mut record = Record::new();
            record.add_metadata("source_line", line_number.to_string());
            record.add_metadata("source_file", path.display().to_string());

            for (i, field) in csv_record.iter().enumerate() {
                if let Some(header) = headers.get(i) {
                    let value = if field.is_empty() {
                        Value::Null
                    } else {
                        Value::String(field.to_string())
                    };
                    record.insert(header.clone(), value);
                }
            }

            records.push(Ok(record));
        }

        Ok(Box::pin(stream::iter(records)))
    }

    async fn estimate_count(&self, config: &SourceConfig) -> Result<Option<u64>, ExtractError> {
        match config {
            SourceConfig::Csv { path, .. } => {
                let path = Path::new(path);
                let count = Self::count_lines(path).await?;
                Ok(Some(count))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_csv_extractor_basic() {
        // Create temp CSV file
        let csv_content =
            "name,email,age\nJohn Doe,john@example.com,30\nJane Smith,jane@example.com,25";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(csv_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = SourceConfig::Csv {
            path: temp_file.path().to_str().unwrap().to_string(),
            csv: None,
        };

        let extractor = CsvExtractor::new();
        let mut stream = extractor.extract(&config).await.unwrap();

        // First record
        let record1 = stream.next().await.unwrap().unwrap();
        assert_eq!(
            record1.get("name").unwrap(),
            &Value::String("John Doe".into())
        );
        assert_eq!(
            record1.get("email").unwrap(),
            &Value::String("john@example.com".into())
        );
        assert_eq!(record1.get("age").unwrap(), &Value::String("30".into()));

        // Second record
        let record2 = stream.next().await.unwrap().unwrap();
        assert_eq!(
            record2.get("name").unwrap(),
            &Value::String("Jane Smith".into())
        );

        // No more records
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_csv_extractor_empty_fields() {
        let csv_content = "name,email\nJohn,\n,jane@example.com";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(csv_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = SourceConfig::Csv {
            path: temp_file.path().to_str().unwrap().to_string(),
            csv: None,
        };

        let extractor = CsvExtractor::new();
        let mut stream = extractor.extract(&config).await.unwrap();

        let record1 = stream.next().await.unwrap().unwrap();
        assert_eq!(record1.get("name").unwrap(), &Value::String("John".into()));
        assert_eq!(record1.get("email").unwrap(), &Value::Null);

        let record2 = stream.next().await.unwrap().unwrap();
        assert_eq!(record2.get("name").unwrap(), &Value::Null);
        assert_eq!(
            record2.get("email").unwrap(),
            &Value::String("jane@example.com".into())
        );
    }

    #[tokio::test]
    async fn test_csv_extractor_estimate_count() {
        let csv_content = "name,email\nJohn,john@example.com\nJane,jane@example.com";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(csv_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = SourceConfig::Csv {
            path: temp_file.path().to_str().unwrap().to_string(),
            csv: None,
        };

        let extractor = CsvExtractor::new();
        let count = extractor.estimate_count(&config).await.unwrap();
        assert_eq!(count, Some(2)); // 3 lines - 1 header = 2 records
    }
}
