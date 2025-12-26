//! Loader registry for managing loaders.

use std::collections::HashMap;
use std::sync::Arc;

use crate::config::TargetConfig;
use crate::error::ConfigError;

use super::traits::Loader;

/// Registry of available loaders.
///
/// Provides a centralized place to manage and access loaders.
/// Useful for building load pipelines from configuration.
pub struct LoaderRegistry {
    loaders: HashMap<String, Arc<dyn Loader>>,
}

impl LoaderRegistry {
    /// Creates a new empty loader registry.
    pub fn new() -> Self {
        Self {
            loaders: HashMap::new(),
        }
    }

    /// Registers a loader by name.
    ///
    /// # Arguments
    ///
    /// * `loader` - The loader to register
    pub fn register(&mut self, loader: Arc<dyn Loader>) {
        self.loaders.insert(loader.name().to_string(), loader);
    }

    /// Gets a loader by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The loader name
    ///
    /// # Returns
    ///
    /// The loader if found, otherwise returns an error.
    pub fn get(&self, name: &str) -> Result<Arc<dyn Loader>, ConfigError> {
        self.loaders
            .get(name)
            .cloned()
            .ok_or_else(|| ConfigError::Validation(format!("Loader '{}' not found", name)))
    }

    /// Gets a loader for the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The target configuration
    ///
    /// # Returns
    ///
    /// The appropriate loader for the config type.
    pub fn get_for_config(&self, config: &TargetConfig) -> Result<Arc<dyn Loader>, ConfigError> {
        let loader_type = match config {
            TargetConfig::BaseVn { .. } => "basevn",
        };

        self.get(loader_type)
    }

    /// Checks if a loader is registered.
    ///
    /// # Arguments
    ///
    /// * `name` - The loader name to check
    pub fn has(&self, name: &str) -> bool {
        self.loaders.contains_key(name)
    }

    /// Lists all registered loader names.
    pub fn list(&self) -> Vec<String> {
        self.loaders.keys().cloned().collect()
    }

    /// Removes a loader by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The loader name to remove
    ///
    /// # Returns
    ///
    /// The removed loader if it existed.
    pub fn remove(&mut self, name: &str) -> Option<Arc<dyn Loader>> {
        self.loaders.remove(name)
    }

    /// Returns the number of registered loaders.
    pub fn len(&self) -> usize {
        self.loaders.len()
    }

    /// Returns true if no loaders are registered.
    pub fn is_empty(&self) -> bool {
        self.loaders.is_empty()
    }

    /// Clears all registered loaders.
    pub fn clear(&mut self) {
        self.loaders.clear();
    }
}

impl Default for LoaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a registry with all built-in loaders pre-registered.
pub fn create_default_registry() -> LoaderRegistry {
    let mut registry = LoaderRegistry::new();

    // Register built-in loaders
    use super::basevn::BaseVnLoader;
    registry.register(Arc::new(BaseVnLoader::new()));

    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AuthConfig, AuthType, BaseVnConfig, BaseVnOptions};
    use crate::load::BaseVnLoader;

    #[test]
    fn test_registry_creation() {
        let registry = LoaderRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = LoaderRegistry::new();

        let loader: Arc<dyn Loader> = Arc::new(BaseVnLoader::new());
        registry.register(loader);

        assert_eq!(registry.len(), 1);
        assert!(registry.has("basevn"));

        let retrieved = registry.get("basevn");
        assert!(retrieved.is_ok());
    }

    #[test]
    fn test_get_loader_not_found() {
        let registry = LoaderRegistry::new();

        let result = registry.get("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_loaders() {
        let mut registry = LoaderRegistry::new();

        let loader: Arc<dyn Loader> = Arc::new(BaseVnLoader::new());
        registry.register(loader);

        let list = registry.list();
        assert_eq!(list.len(), 1);
        assert!(list.contains(&"basevn".to_string()));
    }

    #[test]
    fn test_remove_loader() {
        let mut registry = LoaderRegistry::new();

        let loader: Arc<dyn Loader> = Arc::new(BaseVnLoader::new());
        registry.register(loader);
        assert_eq!(registry.len(), 1);

        let removed = registry.remove("basevn");
        assert!(removed.is_some());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_get_for_config() {
        let mut registry = LoaderRegistry::new();

        let loader: Arc<dyn Loader> = Arc::new(BaseVnLoader::new());
        registry.register(loader);

        let config = TargetConfig::BaseVn {
            basevn: BaseVnConfig {
                base_url: "https://api.base.vn".into(),
                app: "test_app".into(),
                entity: "users".into(),
                auth: AuthConfig {
                    auth_type: AuthType::AccessToken,
                    token: Some("test_token".into()),
                },
                options: BaseVnOptions::default(),
            },
        };

        let result = registry.get_for_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_registry() {
        let registry = create_default_registry();

        assert_eq!(registry.len(), 1);
        assert!(registry.has("basevn"));
    }
}
