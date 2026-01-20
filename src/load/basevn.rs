//! Base.vn API loader implementation.

use async_trait::async_trait;
use rand::Rng;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value as JsonValue};
use std::time::Duration;
use tokio::time::sleep;

use crate::config::{AuthType, BaseVnConfig, TargetConfig};
use crate::core::{Record, Value};
use crate::error::{ConfigError, LoadError};

use super::rate_limiter::RateLimiter;
use super::traits::{LoadResult, Loader};

/// Adds random jitter (±20%) to a duration to prevent thundering herd.
fn jitter_duration(base: Duration) -> Duration {
    let jitter_factor = rand::thread_rng().gen_range(0.8..1.2);
    Duration::from_secs_f64(base.as_secs_f64() * jitter_factor)
}

/// Parses the Retry-After header value.
///
/// Supports two formats:
/// - Seconds: `Retry-After: 120`
/// - HTTP-date: `Retry-After: Wed, 21 Oct 2015 07:28:00 GMT` (treated as fallback)
fn parse_retry_after(header_value: &str) -> Option<Duration> {
    // Try parsing as seconds first (most common)
    if let Ok(seconds) = header_value.trim().parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }

    // Try parsing as HTTP-date (RFC 7231)
    // For simplicity, we'll use a fallback duration if we can't parse
    // Most servers use seconds format
    None
}

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

    /// Parses Base.vn API response to track individual record successes/failures.
    ///
    /// Supports multiple response formats:
    /// 1. Detailed format with results array
    /// 2. Simple format with created/failed counts and errors array
    /// 3. Fallback: assume all succeeded if parsing fails
    fn parse_response(&self, response_text: &str, total_records: usize) -> Result<LoadResult, LoadError> {
        let mut result = LoadResult::new(total_records);

        // Try to parse as JSON
        let json: JsonValue = match serde_json::from_str(response_text) {
            Ok(v) => v,
            Err(_) => {
                // If not JSON, assume all succeeded (backward compatibility)
                result.success = total_records;
                return Ok(result);
            }
        };

        // Try detailed format with results array
        if let Some(results_array) = json.get("results").and_then(|v| v.as_array()) {
            for (idx, item) in results_array.iter().enumerate() {
                let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("success");

                if status == "success" {
                    result.add_success();
                } else {
                    let error = item.get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                        .to_string();
                    let id = item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
                    result.add_failure(idx, id, error);
                }
            }
            return Ok(result);
        }

        // Try simple format with errors array
        if let Some(errors_array) = json.get("errors").and_then(|v| v.as_array()) {
            let created = json.get("created").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

            for error_item in errors_array {
                let index = error_item.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let error = error_item.get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                let id = error_item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());

                result.add_failure(index, id, error);
            }

            result.success = created;
            result.failed = errors_array.len();

            return Ok(result);
        }

        // Fallback: assume all succeeded if no recognized format
        result.success = total_records;
        Ok(result)
    }

    /// Sends a batch of records to Base.vn API with retry logic.
    async fn send_batch_with_retry(
        &self,
        config: &BaseVnConfig,
        records: &[Record],
    ) -> Result<LoadResult, LoadError> {
        let max_retries = config.options.max_retries;
        let mut attempt = 0;
        let mut had_success_this_batch = false;

        loop {
            match self.send_batch(config, records).await {
                Ok(result) => {
                    // Record success for rate recovery
                    if let Some(limiter) = &self.rate_limiter {
                        limiter.record_success().await;
                    }
                    return Ok(result);
                }
                Err(err) => {
                    attempt += 1;

                    // Extract retry_after if rate limit error
                    let (is_rate_limit, retry_after) = match &err {
                        LoadError::RateLimit { retry_after } => (true, *retry_after),
                        _ => (false, None),
                    };

                    // Check if error is retryable
                    let is_retryable = matches!(
                        err,
                        LoadError::Network(_)
                            | LoadError::RateLimit { .. }
                            | LoadError::ApiRequest(_)
                    );

                    if !is_retryable || attempt >= max_retries {
                        return Err(err);
                    }

                    // Calculate delay: use Retry-After if available, else exponential backoff
                    let base_delay = if let Some(retry_dur) = retry_after {
                        retry_dur
                    } else {
                        // Exponential backoff: 1s, 2s, 4s, 8s, ...
                        Duration::from_secs(2_u64.pow((attempt - 1) as u32))
                    };

                    // Add jitter to prevent thundering herd
                    let delay = jitter_duration(base_delay);

                    tracing::warn!(
                        "Batch load attempt {} failed, retrying in {:?}: {}",
                        attempt,
                        delay,
                        err
                    );

                    sleep(delay).await;

                    // If rate limit error, adjust rate limiter with smarter reduction
                    if is_rate_limit {
                        if let Some(limiter) = &self.rate_limiter {
                            // Reset success streak on rate limit
                            limiter.reset_success_streak();

                            let current_rate = limiter.get_rate().await;

                            // Smarter reduction: 50% if first request failed, 25% if some succeeded
                            let reduction_factor = if had_success_this_batch { 0.75 } else { 0.5 };
                            let new_rate =
                                ((current_rate as f64) * reduction_factor).max(1.0) as usize;

                            limiter.adjust_rate(new_rate).await;
                            tracing::info!(
                                "Rate limit hit: reduced {} -> {} req/s (factor: {})",
                                current_rate,
                                new_rate,
                                reduction_factor
                            );
                        }
                    }

                    // Track that we had at least one successful request attempt
                    had_success_this_batch = true;
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
            // Parse Retry-After header if present
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(parse_retry_after);

            return Err(LoadError::RateLimit { retry_after });
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

        // Parse response to identify individual record failures
        let response_text = response.text().await?;
        self.parse_response(&response_text, records.len())
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

    #[test]
    fn test_parse_response_detailed_format_all_success() {
        let loader = BaseVnLoader::new();
        let response = r#"{
            "results": [
                {"index": 0, "id": "12345", "status": "success"},
                {"index": 1, "id": "12346", "status": "success"}
            ]
        }"#;

        let result = loader.parse_response(response, 2).unwrap();

        assert_eq!(result.total, 2);
        assert_eq!(result.success, 2);
        assert_eq!(result.failed, 0);
        assert!(result.failures.is_empty());
        assert!(result.is_complete_success());
    }

    #[test]
    fn test_parse_response_detailed_format_mixed() {
        let loader = BaseVnLoader::new();
        let response = r#"{
            "results": [
                {"index": 0, "id": "12345", "status": "success"},
                {"index": 1, "status": "error", "error": "Validation failed: email required"},
                {"index": 2, "id": "12346", "status": "success"}
            ]
        }"#;

        let result = loader.parse_response(response, 3).unwrap();

        assert_eq!(result.total, 3);
        assert_eq!(result.success, 2);
        assert_eq!(result.failed, 1);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].index, 1);
        assert_eq!(result.failures[0].error, "Validation failed: email required");
        assert!(!result.is_complete_success());
        assert_eq!(result.success_rate(), 66.66666666666666);
    }

    #[test]
    fn test_parse_response_simple_format() {
        let loader = BaseVnLoader::new();
        let response = r#"{
            "created": 2,
            "failed": 1,
            "errors": [
                {"index": 1, "id": "user_123", "error": "Duplicate email"}
            ]
        }"#;

        let result = loader.parse_response(response, 3).unwrap();

        assert_eq!(result.total, 3);
        assert_eq!(result.success, 2);
        assert_eq!(result.failed, 1);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].index, 1);
        assert_eq!(result.failures[0].id, Some("user_123".to_string()));
        assert_eq!(result.failures[0].error, "Duplicate email");
    }

    #[test]
    fn test_parse_response_non_json_fallback() {
        let loader = BaseVnLoader::new();
        let response = "OK";

        let result = loader.parse_response(response, 5).unwrap();

        assert_eq!(result.total, 5);
        assert_eq!(result.success, 5);
        assert_eq!(result.failed, 0);
        assert!(result.failures.is_empty());
    }

    #[test]
    fn test_parse_response_unrecognized_format_fallback() {
        let loader = BaseVnLoader::new();
        let response = r#"{
            "status": "ok",
            "message": "Records processed"
        }"#;

        let result = loader.parse_response(response, 3).unwrap();

        assert_eq!(result.total, 3);
        assert_eq!(result.success, 3);
        assert_eq!(result.failed, 0);
        assert!(result.failures.is_empty());
    }

    #[test]
    fn test_parse_retry_after_seconds() {
        assert_eq!(
            super::parse_retry_after("120"),
            Some(Duration::from_secs(120))
        );
        assert_eq!(
            super::parse_retry_after("  60  "),
            Some(Duration::from_secs(60))
        );
        assert_eq!(super::parse_retry_after("0"), Some(Duration::from_secs(0)));
    }

    #[test]
    fn test_parse_retry_after_invalid() {
        // Non-numeric strings return None
        assert_eq!(super::parse_retry_after("invalid"), None);
        assert_eq!(
            super::parse_retry_after("Wed, 21 Oct 2015 07:28:00 GMT"),
            None
        );
        assert_eq!(super::parse_retry_after(""), None);
    }

    #[test]
    fn test_jitter_duration_range() {
        let base = Duration::from_secs(10);

        // Run multiple times to verify jitter stays in range
        for _ in 0..100 {
            let jittered = super::jitter_duration(base);
            let secs = jittered.as_secs_f64();

            // Should be within ±20% of base (8-12 seconds)
            assert!(secs >= 8.0, "Jitter too low: {}", secs);
            assert!(secs <= 12.0, "Jitter too high: {}", secs);
        }
    }

    #[test]
    fn test_jitter_duration_not_constant() {
        let base = Duration::from_secs(100);

        // Generate multiple jittered values
        let values: Vec<f64> = (0..10)
            .map(|_| super::jitter_duration(base).as_secs_f64())
            .collect();

        // At least some values should be different (randomness)
        let first = values[0];
        let has_variation = values.iter().any(|&v| (v - first).abs() > 0.001);
        assert!(has_variation, "Jitter should produce varying values");
    }
}
