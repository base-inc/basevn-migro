//! Field mapping transformer implementation.

use crate::config::{FieldMap, MappingConfig, MissingRequiredBehavior, UnmappedFieldBehavior};
use crate::core::{Record, Value};
use crate::error::TransformError;

use super::traits::Transformer;

/// Field mapper transformer.
///
/// Maps source fields to target fields according to the mapping configuration.
/// Handles required fields, default values, and unmapped fields.
pub struct FieldMapper {
    config: MappingConfig,
}

impl FieldMapper {
    /// Creates a new field mapper from configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The mapping configuration
    pub fn new(config: MappingConfig) -> Self {
        Self { config }
    }

    /// Creates a field mapper from a list of field maps with default settings.
    ///
    /// # Arguments
    ///
    /// * `fields` - Vector of field mappings
    pub fn from_fields(fields: Vec<FieldMap>) -> Self {
        Self {
            config: MappingConfig {
                fields,
                unmapped_fields: UnmappedFieldBehavior::Ignore,
                missing_required: MissingRequiredBehavior::Error,
            },
        }
    }

    /// Applies a single field mapping to a record.
    fn apply_mapping(
        &self,
        source_record: &Record,
        target_record: &mut Record,
        mapping: &FieldMap,
    ) -> Result<(), TransformError> {
        // Get source value
        let value = if let Some(val) = source_record.get(&mapping.source) {
            val.clone()
        } else if let Some(default) = &mapping.default {
            // Use default value if source field is missing
            Value::String(default.clone())
        } else if mapping.required {
            // Required field is missing and no default
            match self.config.missing_required {
                MissingRequiredBehavior::Error => {
                    return Err(TransformError::MissingRequiredField {
                        field: mapping.source.clone(),
                    });
                }
                MissingRequiredBehavior::SkipRecord => {
                    return Err(TransformError::MissingRequiredField {
                        field: mapping.source.clone(),
                    });
                }
            }
        } else {
            // Non-required field is missing, use null
            Value::Null
        };

        // Check if value is empty and we have a default
        let final_value = if value.is_empty() && mapping.default.is_some() {
            Value::String(mapping.default.clone().unwrap())
        } else {
            value
        };

        // Insert mapped field into target record
        target_record.insert(&mapping.target, final_value);

        Ok(())
    }

    /// Handles unmapped fields from the source record.
    fn handle_unmapped_fields(&self, source_record: &Record) -> Result<(), TransformError> {
        // Get list of mapped source fields
        let mapped_sources: std::collections::HashSet<&str> = self
            .config
            .fields
            .iter()
            .map(|f| f.source.as_str())
            .collect();

        // Find unmapped fields
        let unmapped: Vec<&str> = source_record
            .fields()
            .into_iter()
            .map(|f| f.as_str())
            .filter(|f| !mapped_sources.contains(f))
            .collect();

        if !unmapped.is_empty() {
            match self.config.unmapped_fields {
                UnmappedFieldBehavior::Ignore => {
                    // Do nothing
                }
                UnmappedFieldBehavior::Warn => {
                    // Log warning (for now, just trace)
                    tracing::warn!("Unmapped fields in source record: {}", unmapped.join(", "));
                }
                UnmappedFieldBehavior::Error => {
                    return Err(TransformError::TransformFailed(format!(
                        "Unmapped source fields: {}",
                        unmapped.join(", ")
                    )));
                }
            }
        }

        Ok(())
    }
}

impl Transformer for FieldMapper {
    fn name(&self) -> &'static str {
        "field_mapper"
    }

    fn transform(&self, source_record: Record) -> Result<Record, TransformError> {
        // Handle unmapped fields check first
        self.handle_unmapped_fields(&source_record)?;

        let mut target_record = Record::new();

        // Copy metadata from source to target
        for (key, value) in &source_record.metadata {
            target_record.add_metadata(key.clone(), value.clone());
        }

        // Apply each field mapping
        for mapping in &self.config.fields {
            self.apply_mapping(&source_record, &mut target_record, mapping)?;
        }

        Ok(target_record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_field_mapping() {
        let mapper = FieldMapper::from_fields(vec![
            FieldMap {
                source: "name".into(),
                target: "full_name".into(),
                required: false,
                default: None,
            },
            FieldMap {
                source: "email".into(),
                target: "email_address".into(),
                required: false,
                default: None,
            },
        ]);

        let mut source = Record::new();
        source.insert("name", "John Doe");
        source.insert("email", "john@example.com");

        let result = mapper.transform(source).unwrap();

        assert_eq!(
            result.get("full_name"),
            Some(&Value::String("John Doe".into()))
        );
        assert_eq!(
            result.get("email_address"),
            Some(&Value::String("john@example.com".into()))
        );
    }

    #[test]
    fn test_required_field_missing() {
        let mapper = FieldMapper::from_fields(vec![FieldMap {
            source: "email".into(),
            target: "email".into(),
            required: true,
            default: None,
        }]);

        let mut source = Record::new();
        source.insert("name", "John");

        let result = mapper.transform(source);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::MissingRequiredField { .. }
        ));
    }

    #[test]
    fn test_default_value() {
        let mapper = FieldMapper::from_fields(vec![FieldMap {
            source: "status".into(),
            target: "status".into(),
            required: false,
            default: Some("active".into()),
        }]);

        let source = Record::new();
        let result = mapper.transform(source).unwrap();

        assert_eq!(result.get("status"), Some(&Value::String("active".into())));
    }

    #[test]
    fn test_default_value_for_empty() {
        let mapper = FieldMapper::from_fields(vec![FieldMap {
            source: "status".into(),
            target: "status".into(),
            required: false,
            default: Some("active".into()),
        }]);

        let mut source = Record::new();
        source.insert("status", ""); // Empty string

        let result = mapper.transform(source).unwrap();

        assert_eq!(result.get("status"), Some(&Value::String("active".into())));
    }

    #[test]
    fn test_metadata_preservation() {
        let mapper = FieldMapper::from_fields(vec![FieldMap {
            source: "name".into(),
            target: "full_name".into(),
            required: false,
            default: None,
        }]);

        let mut source = Record::new();
        source.insert("name", "John");
        source.add_metadata("source_line", "1");
        source.add_metadata("source_file", "data.csv");

        let result = mapper.transform(source).unwrap();

        assert_eq!(result.metadata.get("source_line"), Some(&"1".to_string()));
        assert_eq!(
            result.metadata.get("source_file"),
            Some(&"data.csv".to_string())
        );
    }

    #[test]
    fn test_unmapped_fields_ignore() {
        let config = MappingConfig {
            fields: vec![FieldMap {
                source: "name".into(),
                target: "full_name".into(),
                required: false,
                default: None,
            }],
            unmapped_fields: UnmappedFieldBehavior::Ignore,
            missing_required: MissingRequiredBehavior::Error,
        };

        let mapper = FieldMapper::new(config);

        let mut source = Record::new();
        source.insert("name", "John");
        source.insert("extra_field", "extra_value"); // Unmapped field

        let result = mapper.transform(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unmapped_fields_error() {
        let config = MappingConfig {
            fields: vec![FieldMap {
                source: "name".into(),
                target: "full_name".into(),
                required: false,
                default: None,
            }],
            unmapped_fields: UnmappedFieldBehavior::Error,
            missing_required: MissingRequiredBehavior::Error,
        };

        let mapper = FieldMapper::new(config);

        let mut source = Record::new();
        source.insert("name", "John");
        source.insert("extra_field", "extra_value"); // Unmapped field

        let result = mapper.transform(source);
        assert!(result.is_err());
    }
}
