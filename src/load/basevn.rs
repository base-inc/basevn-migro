//! Base.vn API loader implementation.

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value as JsonValue};
use std::time::Duration;
use tokio::time::sleep;

use crate::config::{AuthType, BaseVnConfig, TargetConfig};
use crate::core::{Record, Value};
use crate::error::{ConfigError, LoadError};

use super::rate_limiter::RateLimiter;
use super::traits::{LoadResult, Loader};

/// Base.vn API loader.
///
/// Loads records to Base.vn platform via REST API with:
/// - Access token authentication
/// - Rate limiting (token bucket)
/// - Exponential backoff retry
/// - Batch processing
pub struct BaseVnLoader {
    client: Client,
    rate_limiter: Option<RateLimiter>,
}

impl BaseVnLoader {
    /// Creates a new Base.vn loader.
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            rate_limiter: None,
        }
    }

    /// Converts a Record to JSON payload for Base.vn API.
    fn record_to_json(&self, record: &Record) -> JsonValue {
        let mut obj = serde_json::Map::new();

        for field in record.fields() {
            if let Some(value) = record.get(field) {
                let json_value = self.value_to_json(value);
                obj.insert(field.to_string(), json_value);
            }
        }

        JsonValue::Object(obj)
    }

    /// Converts a Value to JSON.
    fn value_to_json(&self, value: &Value) -> JsonValue {
        match value {
            Value::Null => JsonValue::Null,
            Value::Bool(b) => JsonValue::Bool(*b),
            Value::Number(n) => {
                if let Some(i) = serde_json::Number::from_f64(*n) {
                    JsonValue::Number(i)
                } else {
                    JsonValue::Null
                }
            }
            Value::String(s) => JsonValue::String(s.clone()),
            Value::Array(arr) => {
                let json_arr: Vec<JsonValue> = arr.iter().map(|v| self.value_to_json(v)).collect();
                JsonValue::Array(json_arr)
            }
            Value::Object(obj) => {
                let mut json_obj = serde_json::Map::new();
                for (k, v) in obj {
                    json_obj.insert(k.clone(), self.value_to_json(v));
                }
                JsonValue::Object(json_obj)
            }
        }
    }

    /// Sends a batch of records to Base.vn API with retry logic.
    async fn send_batch_with_retry(
        &self,
        config: &BaseVnConfig,
        records: &[Record],
    ) -> Result<LoadResult, LoadError> {
        let max_retries = config.options.max_retries;
        let mut attempt = 0;

        loop {
            match self.send_batch(config, records).await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    attempt += 1;

                    // Check if error is retryable
                    let is_retryable = matches!(
                        err,
                        LoadError::Network(_) | LoadError::RateLimit | LoadError::ApiRequest(_)
                    );

                    if !is_retryable || attempt >= max_retries {
                        return Err(err);
                    }

                    // Exponential backoff: 1s, 2s, 4s, 8s, ...
                    let delay_secs = 2_u64.pow((attempt - 1) as u32);
                    let delay = Duration::from_secs(delay_secs);

                    tracing::warn!(
                        "Batch load attempt {} failed, retrying in {:?}: {}",
                        attempt,
                        delay,
                        err
                    );

                    sleep(delay).await;

                    // If rate limit error, adjust rate limiter
                    if matches!(err, LoadError::RateLimit) {
                        if let Some(limiter) = &self.rate_limiter {
                            let current_rate = config.options.rate_limit;
                            let new_rate = (current_rate as f64 * 0.75) as usize;
                            limiter.adjust_rate(new_rate.max(1)).await;
                            tracing::info!("Adjusted rate limit to {} req/s", new_rate);
                        }
                    }
                }
            }
        }
    }

    /// Sends a batch of records to Base.vn API.
    async fn send_batch(
        &self,
        config: &BaseVnConfig,
        records: &[Record],
    ) -> Result<LoadResult, LoadError> {
        // Acquire rate limit token
        if let Some(limiter) = &self.rate_limiter {
            limiter.acquire().await;
        }

        // Convert records to JSON
        let payloads: Vec<JsonValue> = records.iter().map(|r| self.record_to_json(r)).collect();

        // Build API URL
        let url = format!("{}/api/{}/{}", config.base_url, config.app, config.entity);

        // Get auth token
        let token = config
            .auth
            .token
            .as_ref()
            .ok_or_else(|| LoadError::Auth("Access token not provided".into()))?;

        // Send request
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&json!({ "records": payloads }))
            .timeout(Duration::from_secs(config.options.timeout))
            .send()
            .await?;

        // Check status
        let status = response.status();

        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(LoadError::RateLimit);
        }

        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LoadError::Auth(format!(
                "Authentication failed: {}",
                error_text
            )));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LoadError::ApiRequest(format!(
                "API request failed with status {}: {}",
                status, error_text
            )));
        }

        // Parse response (assume all records succeeded for MVP)
        // In production, parse response to identify individual failures
        let mut result = LoadResult::new(records.len());
        result.success = records.len();

        Ok(result)
    }
}

impl Default for BaseVnLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Loader for BaseVnLoader {
    fn name(&self) -> &'static str {
        "basevn"
    }

    fn validate_config(&self, config: &TargetConfig) -> Result<(), ConfigError> {
        match config {
            TargetConfig::BaseVn { basevn } => {
                // Validate app name
                if basevn.app.is_empty() {
                    return Err(ConfigError::Validation(
                        "Base.vn app name cannot be empty".into(),
                    ));
                }

                // Validate entity name
                if basevn.entity.is_empty() {
                    return Err(ConfigError::Validation(
                        "Base.vn entity name cannot be empty".into(),
                    ));
                }

                // Validate auth
                if basevn.auth.auth_type == AuthType::AccessToken && basevn.auth.token.is_none() {
                    return Err(ConfigError::Validation(
                        "Access token is required for access_token auth type".into(),
                    ));
                }

                // Validate batch size
                if basevn.options.batch_size == 0 {
                    return Err(ConfigError::Validation("Batch size must be > 0".into()));
                }

                Ok(())
            }
        }
    }

    async fn initialize(&self, config: &TargetConfig) -> Result<(), LoadError> {
        match config {
            TargetConfig::BaseVn { basevn } => {
                // Initialize rate limiter (created in load_batch for MVP)
                tracing::info!(
                    "Initialized Base.vn loader for {}/{} with rate limit {} req/s",
                    basevn.app,
                    basevn.entity,
                    basevn.options.rate_limit
                );

                Ok(())
            }
        }
    }

    async fn load_batch(
        &self,
        records: Vec<Record>,
        config: &TargetConfig,
    ) -> Result<LoadResult, LoadError> {
        match config {
            TargetConfig::BaseVn { basevn } => {
                // Create rate limiter if not exists (workaround for &self limitation)
                let loader = if self.rate_limiter.is_none() {
                    let mut new_loader = BaseVnLoader::new();
                    new_loader.rate_limiter = Some(RateLimiter::new(basevn.options.rate_limit));
                    new_loader
                } else {
                    // Use existing (this branch won't be hit in current design, but kept for clarity)
                    BaseVnLoader {
                        client: self.client.clone(),
                        rate_limiter: self.rate_limiter.clone(),
                    }
                };

                loader.send_batch_with_retry(basevn, &records).await
            }
        }
    }

    async fn finalize(&self) -> Result<(), LoadError> {
        tracing::info!("Finalized Base.vn loader");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AuthConfig, BaseVnOptions};

    #[test]
    fn test_basevn_loader_creation() {
        let loader = BaseVnLoader::new();
        assert_eq!(loader.name(), "basevn");
    }

    #[test]
    fn test_validate_config_success() {
        let loader = BaseVnLoader::new();

        let config = TargetConfig::BaseVn {
            basevn: BaseVnConfig {
                base_url: "https://api.base.vn".into(),
                app: "test_app".into(),
                entity: "users".into(),
                auth: AuthConfig {
                    auth_type: AuthType::AccessToken,
                    token: Some("test_token".into()),
                },
                options: BaseVnOptions::default(),
            },
        };

        assert!(loader.validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_missing_token() {
        let loader = BaseVnLoader::new();

        let config = TargetConfig::BaseVn {
            basevn: BaseVnConfig {
                base_url: "https://api.base.vn".into(),
                app: "test_app".into(),
                entity: "users".into(),
                auth: AuthConfig {
                    auth_type: AuthType::AccessToken,
                    token: None,
                },
                options: BaseVnOptions::default(),
            },
        };

        assert!(loader.validate_config(&config).is_err());
    }

    #[test]
    fn test_record_to_json() {
        let loader = BaseVnLoader::new();

        let mut record = Record::new();
        record.insert("name", "John Doe");
        record.insert("age", "30");

        let json = loader.record_to_json(&record);

        assert!(json.is_object());
        assert_eq!(json["name"], JsonValue::String("John Doe".into()));
        assert_eq!(json["age"], JsonValue::String("30".into()));
    }
}
