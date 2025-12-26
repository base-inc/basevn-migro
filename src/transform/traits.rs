//! Transformer trait definition.

use crate::core::Record;
use crate::error::TransformError;

/// Trait for data transformers.
///
/// Transformers modify records as they flow through the ETL pipeline.
/// They can rename fields, validate data, apply default values, etc.
pub trait Transformer: Send + Sync {
    /// Unique identifier for this transformer.
    fn name(&self) -> &'static str;

    /// Transform a single record.
    ///
    /// # Arguments
    ///
    /// * `record` - The record to transform
    ///
    /// # Returns
    ///
    /// Transformed `Record` or `TransformError`.
    ///
    /// # Errors
    ///
    /// Returns `TransformError` if transformation fails.
    fn transform(&self, record: Record) -> Result<Record, TransformError>;

    /// Batch transform multiple records.
    ///
    /// Default implementation iterates over records and calls `transform()` on each.
    /// Transformers can override this for more efficient batch processing.
    ///
    /// # Arguments
    ///
    /// * `records` - Vector of records to transform
    ///
    /// # Returns
    ///
    /// Vector of transformed records or `TransformError`.
    ///
    /// # Errors
    ///
    /// Returns `TransformError` if any transformation fails.
    fn transform_batch(&self, records: Vec<Record>) -> Result<Vec<Record>, TransformError> {
        records.into_iter().map(|r| self.transform(r)).collect()
    }
}

/// Transformer chain for applying multiple transformers in sequence.
///
/// # Example
///
/// ```ignore
/// let chain = TransformerChain::new()
///     .with_transformer(field_mapper)
///     .with_transformer(validator);
///
/// let transformed = chain.transform(record)?;
/// ```
pub struct TransformerChain {
    transformers: Vec<Box<dyn Transformer>>,
}

impl TransformerChain {
    /// Creates a new empty transformer chain.
    pub fn new() -> Self {
        Self {
            transformers: Vec::new(),
        }
    }

    /// Adds a transformer to the chain.
    pub fn with_transformer<T: Transformer + 'static>(mut self, transformer: T) -> Self {
        self.transformers.push(Box::new(transformer));
        self
    }

    /// Transforms a record through all transformers in sequence.
    pub fn transform(&self, mut record: Record) -> Result<Record, TransformError> {
        for transformer in &self.transformers {
            record = transformer.transform(record)?;
        }
        Ok(record)
    }

    /// Batch transforms records through all transformers.
    pub fn transform_batch(&self, mut records: Vec<Record>) -> Result<Vec<Record>, TransformError> {
        for transformer in &self.transformers {
            records = transformer.transform_batch(records)?;
        }
        Ok(records)
    }

    /// Returns the number of transformers in the chain.
    pub fn len(&self) -> usize {
        self.transformers.len()
    }

    /// Returns true if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.transformers.is_empty()
    }
}

impl Default for TransformerChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    struct TestTransformer;
    impl Transformer for TestTransformer {
        fn name(&self) -> &'static str {
            "test"
        }

        fn transform(&self, mut record: Record) -> Result<Record, TransformError> {
            record.insert("test_field", "test_value");
            Ok(record)
        }
    }

    #[test]
    fn test_transformer_chain() {
        let chain = TransformerChain::new().with_transformer(TestTransformer);

        let record = Record::new();
        let result = chain.transform(record).unwrap();

        assert_eq!(
            result.get("test_field"),
            Some(&Value::String("test_value".into()))
        );
    }

    #[test]
    fn test_empty_chain() {
        let chain = TransformerChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
    }
}
