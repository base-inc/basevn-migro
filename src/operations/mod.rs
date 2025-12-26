//! Operations layer for ETL pipeline management.
//!
//! This module provides operational capabilities including:
//! - Progress tracking with real-time updates
//! - Audit logging in JSONL format
//! - Checkpointing for resume capability
//! - Sync modes (full/incremental)

pub mod audit;
pub mod checkpoint;
pub mod progress;
pub mod sync;

pub use audit::{AuditEvent, AuditLogger};
pub use checkpoint::{Checkpoint, CheckpointManager};
pub use progress::ProgressTracker;
pub use sync::{ConflictStrategy, SyncMode, SyncStrategy};
