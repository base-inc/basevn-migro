//! Field mapping configuration.

use serde::{Deserialize, Serialize};

/// Field mapping configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingConfig {
    /// Field mappings.
    pub fields: Vec<FieldMap>,

    /// How to handle unmapped source fields.
    #[serde(default = "default_unmapped_fields")]
    pub unmapped_fields: UnmappedFieldBehavior,

    /// How to handle missing required fields.
    #[serde(default = "default_missing_required")]
    pub missing_required: MissingRequiredBehavior,
}

fn default_unmapped_fields() -> UnmappedFieldBehavior {
    UnmappedFieldBehavior::Ignore
}

fn default_missing_required() -> MissingRequiredBehavior {
    MissingRequiredBehavior::Error
}

/// Single field mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMap {
    /// Source field name.
    pub source: String,

    /// Target field name.
    pub target: String,

    /// Is this field required?
    #[serde(default)]
    pub required: bool,

    /// Default value if source is null/empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Behavior for unmapped source fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnmappedFieldBehavior {
    Ignore,
    Warn,
    Error,
}

/// Behavior for missing required fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingRequiredBehavior {
    Error,
    SkipRecord,
}
