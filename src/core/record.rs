//! Universal record representation for data migration.
//!
//! The `Record` type is the core data structure that flows through the ETL pipeline.
//! It represents a single unit of data (e.g., a CSV row, an API response object).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal data value type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    /// Returns true if the value is null or empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Value::Null => true,
            Value::String(s) => s.is_empty(),
            Value::Array(a) => a.is_empty(),
            Value::Object(o) => o.is_empty(),
            _ => false,
        }
    }

    /// Converts the value to a string representation.
    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            Value::Null => None,
            _ => Some(serde_json::to_string(self).unwrap_or_default()),
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

/// Field name alias.
pub type Field = String;

/// Universal record type.
///
/// Represents a single data record with named fields and values.
/// Records flow through the Extract → Transform → Load pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    /// Field-value pairs.
    fields: HashMap<Field, Value>,

    /// Optional metadata (offset, source ID, etc.).
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

impl Record {
    /// Creates a new empty record.
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Inserts a field-value pair.
    pub fn insert<K: Into<Field>, V: Into<Value>>(&mut self, key: K, value: V) {
        self.fields.insert(key.into(), value.into());
    }

    /// Gets a value by field name.
    pub fn get(&self, field: &str) -> Option<&Value> {
        self.fields.get(field)
    }

    /// Removes a field and returns its value.
    pub fn remove(&mut self, field: &str) -> Option<Value> {
        self.fields.remove(field)
    }

    /// Returns true if the record contains the field.
    pub fn contains(&self, field: &str) -> bool {
        self.fields.contains_key(field)
    }

    /// Returns all field names.
    pub fn fields(&self) -> Vec<&Field> {
        self.fields.keys().collect()
    }

    /// Returns an iterator over field-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&Field, &Value)> {
        self.fields.iter()
    }

    /// Returns the number of fields.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Returns true if the record has no fields.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Adds metadata.
    pub fn add_metadata<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.metadata.insert(key.into(), value.into());
    }
}

impl Default for Record {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> FromIterator<(K, V)> for Record
where
    K: Into<Field>,
    V: Into<Value>,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self {
            fields: iter
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let mut record = Record::new();
        record.insert("name", "John Doe");
        record.insert("age", 30.0);

        assert_eq!(record.len(), 2);
        assert_eq!(record.get("name"), Some(&Value::String("John Doe".into())));
        assert_eq!(record.get("age"), Some(&Value::Number(30.0)));
    }

    #[test]
    fn test_record_from_iter() {
        let record = Record::from_iter(vec![("name", "Jane"), ("email", "jane@example.com")]);

        assert_eq!(record.len(), 2);
        assert!(record.contains("name"));
        assert!(record.contains("email"));
    }

    #[test]
    fn test_value_is_empty() {
        assert!(Value::Null.is_empty());
        assert!(Value::String("".into()).is_empty());
        assert!(!Value::String("test".into()).is_empty());
        assert!(Value::Array(vec![]).is_empty());
        assert!(!Value::Array(vec![Value::Null]).is_empty());
    }

    #[test]
    fn test_value_as_string() {
        assert_eq!(
            Value::String("test".into()).as_string(),
            Some("test".into())
        );
        assert_eq!(Value::Number(42.5).as_string(), Some("42.5".into()));
        assert_eq!(Value::Bool(true).as_string(), Some("true".into()));
        assert_eq!(Value::Null.as_string(), None);
    }
}
