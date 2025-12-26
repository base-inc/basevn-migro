//! Field filter transformer implementation.

use crate::core::Record;
use crate::error::TransformError;

use super::traits::Transformer;

/// Field filter transformer.
///
/// Keeps only specified fields or removes specified fields from records.
/// Useful for reducing data size or removing sensitive information.
#[derive(Clone)]
pub struct FieldFilter {
    mode: FilterMode,
    fields: Vec<String>,
}

/// Filter mode determines whether to keep or remove fields.
#[derive(Clone, Debug, PartialEq)]
pub enum FilterMode {
    /// Keep only the specified fields.
    Keep,
    /// Remove the specified fields.
    Remove,
}

impl FieldFilter {
    /// Creates a new field filter that keeps only specified fields.
    ///
    /// # Arguments
    ///
    /// * `fields` - Fields to keep
    pub fn keep(fields: Vec<String>) -> Self {
        Self {
            mode: FilterMode::Keep,
            fields,
        }
    }

    /// Creates a new field filter that removes specified fields.
    ///
    /// # Arguments
    ///
    /// * `fields` - Fields to remove
    pub fn remove(fields: Vec<String>) -> Self {
        Self {
            mode: FilterMode::Remove,
            fields,
        }
    }
}

impl Transformer for FieldFilter {
    fn name(&self) -> &'static str {
        "field_filter"
    }

    fn transform(&self, record: Record) -> Result<Record, TransformError> {
        let mut result = Record::new();

        // Copy metadata
        for (key, value) in record.iter() {
            match self.mode {
                FilterMode::Keep => {
                    // Keep only specified fields
                    if self.fields.contains(&key.to_string()) {
                        result.insert(key, value.clone());
                    }
                }
                FilterMode::Remove => {
                    // Remove specified fields
                    if !self.fields.contains(&key.to_string()) {
                        result.insert(key, value.clone());
                    }
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_keep_fields() {
        let filter = FieldFilter::keep(vec!["name".to_string(), "email".to_string()]);

        let mut record = Record::new();
        record.insert("name", "John Doe");
        record.insert("email", "john@example.com");
        record.insert("age", 30.0);
        record.insert("phone", "123-456-7890");

        let result = filter.transform(record).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(
            result.get("name"),
            Some(&Value::String("John Doe".into()))
        );
        assert_eq!(
            result.get("email"),
            Some(&Value::String("john@example.com".into()))
        );
        assert_eq!(result.get("age"), None);
        assert_eq!(result.get("phone"), None);
    }

    #[test]
    fn test_remove_fields() {
        let filter = FieldFilter::remove(vec!["password".to_string(), "ssn".to_string()]);

        let mut record = Record::new();
        record.insert("name", "John Doe");
        record.insert("email", "john@example.com");
        record.insert("password", "secret123");
        record.insert("ssn", "123-45-6789");

        let result = filter.transform(record).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(
            result.get("name"),
            Some(&Value::String("John Doe".into()))
        );
        assert_eq!(
            result.get("email"),
            Some(&Value::String("john@example.com".into()))
        );
        assert_eq!(result.get("password"), None);
        assert_eq!(result.get("ssn"), None);
    }

    #[test]
    fn test_keep_empty_list() {
        let filter = FieldFilter::keep(vec![]);

        let mut record = Record::new();
        record.insert("name", "John Doe");
        record.insert("email", "john@example.com");

        let result = filter.transform(record).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_remove_empty_list() {
        let filter = FieldFilter::remove(vec![]);

        let mut record = Record::new();
        record.insert("name", "John Doe");
        record.insert("email", "john@example.com");

        let result = filter.transform(record).unwrap();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_keep_nonexistent_fields() {
        let filter = FieldFilter::keep(vec!["nonexistent".to_string()]);

        let mut record = Record::new();
        record.insert("name", "John Doe");

        let result = filter.transform(record).unwrap();

        assert_eq!(result.len(), 0);
    }
}
