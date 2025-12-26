//! Transformer registry for managing transformers.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::Record;
use crate::error::TransformError;

use super::traits::Transformer;

/// Registry of available transformers.
///
/// Provides a centralized place to manage and access transformers.
/// Useful for building transformer chains from configuration.
pub struct TransformerRegistry {
    transformers: HashMap<String, Arc<dyn Transformer>>,
}

impl TransformerRegistry {
    /// Creates a new empty transformer registry.
    pub fn new() -> Self {
        Self {
            transformers: HashMap::new(),
        }
    }

    /// Registers a transformer by name.
    ///
    /// # Arguments
    ///
    /// * `transformer` - The transformer to register
    pub fn register(&mut self, transformer: Arc<dyn Transformer>) {
        self.transformers
            .insert(transformer.name().to_string(), transformer);
    }

    /// Gets a transformer by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The transformer name
    ///
    /// # Returns
    ///
    /// The transformer if found, otherwise `None`.
    pub fn get(&self, name: &str) -> Option<Arc<dyn Transformer>> {
        self.transformers.get(name).cloned()
    }

    /// Checks if a transformer is registered.
    ///
    /// # Arguments
    ///
    /// * `name` - The transformer name to check
    pub fn has(&self, name: &str) -> bool {
        self.transformers.contains_key(name)
    }

    /// Lists all registered transformer names.
    pub fn list(&self) -> Vec<String> {
        self.transformers.keys().cloned().collect()
    }

    /// Removes a transformer by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The transformer name to remove
    ///
    /// # Returns
    ///
    /// The removed transformer if it existed.
    pub fn remove(&mut self, name: &str) -> Option<Arc<dyn Transformer>> {
        self.transformers.remove(name)
    }

    /// Returns the number of registered transformers.
    pub fn len(&self) -> usize {
        self.transformers.len()
    }

    /// Returns true if no transformers are registered.
    pub fn is_empty(&self) -> bool {
        self.transformers.is_empty()
    }

    /// Clears all registered transformers.
    pub fn clear(&mut self) {
        self.transformers.clear();
    }
}

impl Default for TransformerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating transformer chains from configuration.
pub struct TransformerChainBuilder {
    registry: Arc<TransformerRegistry>,
    transformers: Vec<Arc<dyn Transformer>>,
}

impl TransformerChainBuilder {
    /// Creates a new builder with the given registry.
    pub fn new(registry: Arc<TransformerRegistry>) -> Self {
        Self {
            registry,
            transformers: Vec::new(),
        }
    }

    /// Adds a transformer by name from the registry.
    ///
    /// # Arguments
    ///
    /// * `name` - The transformer name to add
    ///
    /// # Errors
    ///
    /// Returns error if transformer not found in registry.
    pub fn with_transformer(mut self, name: &str) -> Result<Self, TransformError> {
        let transformer = self
            .registry
            .get(name)
            .ok_or_else(|| {
                TransformError::TransformFailed(format!("Transformer '{}' not found", name))
            })?;

        self.transformers.push(transformer);
        Ok(self)
    }

    /// Adds a transformer instance directly.
    pub fn with_transformer_instance<T: Transformer + 'static>(
        mut self,
        transformer: T,
    ) -> Self {
        self.transformers.push(Arc::new(transformer));
        self
    }

    /// Transforms a record through all configured transformers.
    pub fn transform(&self, mut record: Record) -> Result<Record, TransformError> {
        for transformer in &self.transformers {
            record = transformer.transform(record)?;
        }
        Ok(record)
    }

    /// Batch transforms records through all configured transformers.
    pub fn transform_batch(&self, mut records: Vec<Record>) -> Result<Vec<Record>, TransformError> {
        for transformer in &self.transformers {
            records = transformer.transform_batch(records)?;
        }
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FieldMap, MappingConfig, MissingRequiredBehavior, UnmappedFieldBehavior};
    use crate::core::Value;
    use crate::transform::FieldMapper;

    #[test]
    fn test_registry_creation() {
        let registry = TransformerRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = TransformerRegistry::new();

        let mapper: Arc<dyn Transformer> = Arc::new(FieldMapper::new(MappingConfig {
            fields: vec![],
            unmapped_fields: UnmappedFieldBehavior::Ignore,
            missing_required: MissingRequiredBehavior::Error,
        }));

        registry.register(mapper);

        assert_eq!(registry.len(), 1);
        assert!(registry.has("field_mapper"));

        let retrieved = registry.get("field_mapper");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_list_transformers() {
        let mut registry = TransformerRegistry::new();

        let mapper: Arc<dyn Transformer> = Arc::new(FieldMapper::new(MappingConfig {
            fields: vec![],
            unmapped_fields: UnmappedFieldBehavior::Ignore,
            missing_required: MissingRequiredBehavior::Error,
        }));

        registry.register(mapper);

        let list = registry.list();
        assert_eq!(list.len(), 1);
        assert!(list.contains(&"field_mapper".to_string()));
    }

    #[test]
    fn test_remove_transformer() {
        let mut registry = TransformerRegistry::new();

        let mapper: Arc<dyn Transformer> = Arc::new(FieldMapper::new(MappingConfig {
            fields: vec![],
            unmapped_fields: UnmappedFieldBehavior::Ignore,
            missing_required: MissingRequiredBehavior::Error,
        }));

        registry.register(mapper);
        assert_eq!(registry.len(), 1);

        let removed = registry.remove("field_mapper");
        assert!(removed.is_some());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_chain_builder() {
        let mut registry = TransformerRegistry::new();

        let mapper: Arc<dyn Transformer> = Arc::new(FieldMapper::from_fields(vec![
            FieldMap {
                source: "name".to_string(),
                target: "full_name".to_string(),
                required: false,
                default: None,
            },
        ]));

        registry.register(mapper);

        let builder = TransformerChainBuilder::new(Arc::new(registry))
            .with_transformer("field_mapper")
            .unwrap();

        let mut record = Record::new();
        record.insert("name", "John Doe");

        let result = builder.transform(record).unwrap();
        assert_eq!(
            result.get("full_name"),
            Some(&Value::String("John Doe".into()))
        );
    }

    #[test]
    fn test_chain_builder_not_found() {
        let registry = TransformerRegistry::new();

        let result = TransformerChainBuilder::new(Arc::new(registry))
            .with_transformer("nonexistent");

        assert!(result.is_err());
    }
}
