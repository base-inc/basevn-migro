//! Data extraction layer.
//!
//! This module provides extractors for various data sources including CSV, Excel, and REST APIs.

pub mod csv;
pub mod excel;
pub mod registry;
pub mod rest_api;
pub mod traits;

pub use csv::CsvExtractor;
pub use excel::ExcelExtractor;
pub use registry::ExtractorRegistry;
pub use rest_api::RestApiExtractor;
pub use traits::{Extractor, RecordStream};
