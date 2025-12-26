//! Target configuration types.

use serde::{Deserialize, Serialize};

/// Target configuration enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TargetConfig {
    BaseVn {
        #[serde(flatten)]
        basevn: BaseVnConfig,
    },
}

/// Base.vn-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseVnConfig {
    /// Base URL.
    #[serde(default = "default_basevn_base_url")]
    pub base_url: String,

    /// Application name.
    pub app: String,

    /// Entity type.
    pub entity: String,

    /// Authentication configuration.
    pub auth: AuthConfig,

    /// Optional settings.
    #[serde(default)]
    pub options: BaseVnOptions,
}

fn default_basevn_base_url() -> String {
    "https://api.base.vn".to_string()
}

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Auth type.
    #[serde(rename = "type")]
    pub auth_type: AuthType,

    /// Access token (for access_token type).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// Authentication type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    AccessToken,
}

/// Base.vn API options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseVnOptions {
    /// Batch size (records per API call).
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Rate limit (requests per second).
    #[serde(default = "default_rate_limit")]
    pub rate_limit: usize,

    /// Request timeout (seconds).
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum retry attempts.
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
}

impl Default for BaseVnOptions {
    fn default() -> Self {
        Self {
            batch_size: default_batch_size(),
            rate_limit: default_rate_limit(),
            timeout: default_timeout(),
            max_retries: default_max_retries(),
        }
    }
}

fn default_batch_size() -> usize {
    100
}

fn default_rate_limit() -> usize {
    10
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> usize {
    3
}
