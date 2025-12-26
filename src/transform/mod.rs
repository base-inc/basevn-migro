//! Data transformation layer.
//!
//! This module provides transformers for modifying records during the ETL pipeline.

pub mod field_mapping;
pub mod traits;

pub use field_mapping::FieldMapper;
pub use traits::Transformer;
