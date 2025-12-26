//! Configuration management for basevn-migro.
//!
//! This module handles loading and validating job configurations from YAML or JSON files.

pub mod job;
pub mod loader;
pub mod mapping;
pub mod source;
pub mod target;

pub use job::JobConfig;
pub use loader::load_config;
pub use mapping::{FieldMap, MappingConfig, MissingRequiredBehavior, UnmappedFieldBehavior};
pub use source::{CsvOptions, ExcelOptions, PaginationType, RestApiOptions, SourceConfig};
pub use target::{AuthConfig, AuthType, BaseVnConfig, BaseVnOptions, TargetConfig};
