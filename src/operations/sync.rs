//! Sync modes for ETL operations.

use serde::{Deserialize, Serialize};

/// Sync mode determines how records are loaded to the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    /// Full sync: Delete all existing records and insert all new records.
    Full,

    /// Incremental sync: Match by key field, handle conflicts with strategy.
    Incremental,
}

/// Conflict resolution strategy for incremental sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConflictStrategy {
    /// Skip conflicting records (keep existing).
    Skip,

    /// Update conflicting records (overwrite existing).
    #[default]
    Update,

    /// Error on conflict (fail the record).
    Error,
}

/// Sync strategy configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncStrategy {
    /// Sync mode (full or incremental).
    pub mode: SyncMode,

    /// Key field for matching records in incremental sync.
    /// Required for incremental mode.
    pub key_field: Option<String>,

    /// Conflict resolution strategy for incremental sync.
    pub conflict_strategy: ConflictStrategy,
}

impl SyncStrategy {
    /// Creates a new full sync strategy.
    pub fn full() -> Self {
        Self {
            mode: SyncMode::Full,
            key_field: None,
            conflict_strategy: ConflictStrategy::default(),
        }
    }

    /// Creates a new incremental sync strategy.
    ///
    /// # Arguments
    ///
    /// * `key_field` - Field name to use for record matching
    /// * `conflict_strategy` - How to handle conflicting records
    pub fn incremental(key_field: String, conflict_strategy: ConflictStrategy) -> Self {
        Self {
            mode: SyncMode::Incremental,
            key_field: Some(key_field),
            conflict_strategy,
        }
    }

    /// Validates the sync strategy configuration.
    ///
    /// # Errors
    ///
    /// Returns error if incremental mode is missing key_field.
    pub fn validate(&self) -> Result<(), String> {
        if self.mode == SyncMode::Incremental && self.key_field.is_none() {
            return Err("Incremental sync requires key_field".to_string());
        }
        Ok(())
    }

    /// Returns true if this is a full sync.
    pub fn is_full(&self) -> bool {
        self.mode == SyncMode::Full
    }

    /// Returns true if this is an incremental sync.
    pub fn is_incremental(&self) -> bool {
        self.mode == SyncMode::Incremental
    }
}

impl Default for SyncStrategy {
    fn default() -> Self {
        Self::full()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_strategy_full() {
        let strategy = SyncStrategy::full();
        assert_eq!(strategy.mode, SyncMode::Full);
        assert!(strategy.key_field.is_none());
        assert!(strategy.is_full());
        assert!(!strategy.is_incremental());
        assert!(strategy.validate().is_ok());
    }

    #[test]
    fn test_sync_strategy_incremental() {
        let strategy = SyncStrategy::incremental("id".to_string(), ConflictStrategy::Update);
        assert_eq!(strategy.mode, SyncMode::Incremental);
        assert_eq!(strategy.key_field, Some("id".to_string()));
        assert_eq!(strategy.conflict_strategy, ConflictStrategy::Update);
        assert!(strategy.is_incremental());
        assert!(!strategy.is_full());
        assert!(strategy.validate().is_ok());
    }

    #[test]
    fn test_sync_strategy_incremental_missing_key_field() {
        let strategy = SyncStrategy {
            mode: SyncMode::Incremental,
            key_field: None,
            conflict_strategy: ConflictStrategy::Update,
        };

        let result = strategy.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Incremental sync requires key_field"
        );
    }

    #[test]
    fn test_sync_strategy_default() {
        let strategy = SyncStrategy::default();
        assert_eq!(strategy.mode, SyncMode::Full);
        assert!(strategy.is_full());
    }

    #[test]
    fn test_conflict_strategy_default() {
        let strategy = ConflictStrategy::default();
        assert_eq!(strategy, ConflictStrategy::Update);
    }

    #[test]
    fn test_conflict_strategy_variants() {
        assert_eq!(ConflictStrategy::Skip, ConflictStrategy::Skip);
        assert_eq!(ConflictStrategy::Update, ConflictStrategy::Update);
        assert_eq!(ConflictStrategy::Error, ConflictStrategy::Error);
        assert_ne!(ConflictStrategy::Skip, ConflictStrategy::Update);
    }

    #[test]
    fn test_sync_mode_serialization() {
        let mode = SyncMode::Full;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"full\"");

        let mode = SyncMode::Incremental;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"incremental\"");
    }

    #[test]
    fn test_conflict_strategy_serialization() {
        let strategy = ConflictStrategy::Skip;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"skip\"");

        let strategy = ConflictStrategy::Update;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"update\"");

        let strategy = ConflictStrategy::Error;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"error\"");
    }

    #[test]
    fn test_sync_strategy_serialization() {
        let strategy = SyncStrategy::incremental("email".to_string(), ConflictStrategy::Skip);
        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: SyncStrategy = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.mode, SyncMode::Incremental);
        assert_eq!(deserialized.key_field, Some("email".to_string()));
        assert_eq!(deserialized.conflict_strategy, ConflictStrategy::Skip);
    }
}
