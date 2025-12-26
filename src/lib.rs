//! basevn-migro: Open-source ETL tool for migrating data to Base.vn platform.
//!
//! This library provides the core functionality for extracting, transforming,
//! and loading data from various sources to the Base.vn platform.

pub mod cli;
pub mod config;
pub mod core;
pub mod error;
pub mod extract;
pub mod load;
pub mod transform;

// Re-export commonly used types
pub use config::JobConfig;
pub use core::{Record, Value};
pub use error::MigroError;
pub use extract::{CsvExtractor, ExcelExtractor, Extractor, RestApiExtractor};
pub use load::{BaseVnLoader, LoadResult, Loader};
pub use transform::{FieldMapper, Transformer};
