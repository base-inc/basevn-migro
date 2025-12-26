//! Job configuration structure.

use serde::{Deserialize, Serialize};

use super::{MappingConfig, SourceConfig, TargetConfig};

/// Complete job configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    /// Schema version.
    #[serde(default = "default_version")]
    pub version: String,

    /// Job metadata.
    pub job: JobMetadata,

    /// Source configuration.
    pub source: SourceConfig,

    /// Target configuration.
    pub target: TargetConfig,

    /// Field mapping configuration.
    pub mapping: MappingConfig,

    /// Sync behavior.
    #[serde(default)]
    pub sync: SyncConfig,

    /// Retry configuration.
    #[serde(default)]
    pub retry: RetryConfig,

    /// Logging configuration.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Audit configuration.
    #[serde(default)]
    pub audit: AuditConfig,

    /// Checkpoint configuration.
    #[serde(default)]
    pub checkpoint: CheckpointConfig,
}

fn default_version() -> String {
    "1".to_string()
}

/// Job metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    /// Unique job identifier.
    pub id: String,

    /// Human-readable name.
    pub name: String,

    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Sync mode configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Sync mode: full or incremental.
    #[serde(default = "default_sync_mode")]
    pub mode: SyncMode,

    /// Incremental sync options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental: Option<IncrementalConfig>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            mode: SyncMode::Full,
            incremental: None,
        }
    }
}

fn default_sync_mode() -> SyncMode {
    SyncMode::Full
}

/// Sync mode enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncMode {
    Full,
    Incremental,
}

/// Incremental sync configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalConfig {
    /// Field used to identify existing records.
    pub key_field: String,

    /// Conflict resolution strategy.
    #[serde(default = "default_conflict")]
    pub conflict: ConflictStrategy,

    /// Optional state file path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_file: Option<String>,
}

fn default_conflict() -> ConflictStrategy {
    ConflictStrategy::Skip
}

/// Conflict resolution strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStrategy {
    Skip,
    Update,
    Error,
}

/// Retry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts.
    #[serde(default = "default_max_attempts")]
    pub max_attempts: usize,

    /// Retry strategy.
    #[serde(default = "default_retry_strategy")]
    pub strategy: RetryStrategy,

    /// Initial delay in milliseconds.
    #[serde(default = "default_initial_delay")]
    pub initial_delay_ms: u64,

    /// Maximum delay in milliseconds.
    #[serde(default = "default_max_delay")]
    pub max_delay_ms: u64,

    /// Multiplier for exponential backoff.
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: default_max_attempts(),
            strategy: default_retry_strategy(),
            initial_delay_ms: default_initial_delay(),
            max_delay_ms: default_max_delay(),
            multiplier: default_multiplier(),
        }
    }
}

fn default_max_attempts() -> usize {
    3
}

fn default_retry_strategy() -> RetryStrategy {
    RetryStrategy::Exponential
}

fn default_initial_delay() -> u64 {
    1000
}

fn default_max_delay() -> u64 {
    30000
}

fn default_multiplier() -> f64 {
    2.0
}

/// Retry strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RetryStrategy {
    Fixed,
    Exponential,
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level.
    #[serde(default = "default_log_level")]
    pub level: LogLevel,

    /// Log format.
    #[serde(default = "default_log_format")]
    pub format: LogFormat,

    /// Optional log file path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            file: None,
        }
    }
}

fn default_log_level() -> LogLevel {
    LogLevel::Info
}

fn default_log_format() -> LogFormat {
    LogFormat::Text
}

/// Log level enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Log format enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Text,
    Json,
}

/// Audit configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging.
    #[serde(default = "default_audit_enabled")]
    pub enabled: bool,

    /// Audit log file path.
    #[serde(default = "default_audit_file")]
    pub file: String,

    /// Include full record data in audit logs.
    #[serde(default)]
    pub include_record_data: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: default_audit_enabled(),
            file: default_audit_file(),
            include_record_data: false,
        }
    }
}

fn default_audit_enabled() -> bool {
    true
}

fn default_audit_file() -> String {
    "./logs/audit.jsonl".to_string()
}

/// Checkpoint configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    /// Enable checkpointing.
    #[serde(default = "default_checkpoint_enabled")]
    pub enabled: bool,

    /// Checkpoint file path.
    #[serde(default = "default_checkpoint_file")]
    pub file: String,

    /// Checkpoint interval (records).
    #[serde(default = "default_checkpoint_interval")]
    pub interval: usize,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            enabled: default_checkpoint_enabled(),
            file: default_checkpoint_file(),
            interval: default_checkpoint_interval(),
        }
    }
}

fn default_checkpoint_enabled() -> bool {
    true
}

fn default_checkpoint_file() -> String {
    "./.migro/checkpoint.json".to_string()
}

fn default_checkpoint_interval() -> usize {
    100
}
