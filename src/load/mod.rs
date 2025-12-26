//! Data loading layer.
//!
//! This module provides loaders for writing records to target systems.

pub mod basevn;
pub mod rate_limiter;
pub mod registry;
pub mod traits;

pub use basevn::BaseVnLoader;
pub use registry::LoaderRegistry;
pub use traits::{LoadResult, Loader, RecordFailure};
