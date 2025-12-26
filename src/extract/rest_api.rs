//! REST API extractor implementation.

use async_trait::async_trait;
use futures::stream;
use reqwest::Client;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

use crate::config::{PaginationType, RestApiOptions, SourceConfig};
use crate::core::{Record, Value};
use crate::error::{ConfigError, ExtractError};

use super::traits::{Extractor, RecordStream};

/// REST API extractor.
///
/// Extracts records from REST APIs with support for pagination.
pub struct RestApiExtractor {
    client: Client,
}

impl RestApiExtractor {
    /// Creates a new REST API extractor.
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Converts JSON value to our Value type.
    fn json_to_value(json: &JsonValue) -> Value {
        match json {
            JsonValue::Null => Value::Null,
            JsonValue::Bool(b) => Value::Bool(*b),
            JsonValue::Number(n) => {
                if let Some(f) = n.as_f64() {
                    Value::Number(f)
                } else {
                    Value::Number(n.as_i64().unwrap_or(0) as f64)
                }
            }
            JsonValue::String(s) => Value::String(s.clone()),
            JsonValue::Array(arr) => Value::Array(arr.iter().map(Self::json_to_value).collect()),
            JsonValue::Object(obj) => {
                let map: HashMap<String, Value> = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::json_to_value(v)))
                    .collect();
                Value::Object(map)
            }
        }
    }

    /// Extracts records array from JSON response using JSONPath-like simple path.
    fn extract_records_from_response(
        json: &JsonValue,
        path: &str,
    ) -> Result<Vec<JsonValue>, ExtractError> {
        if path.is_empty() {
            // If no path specified, assume root is the array
            if let JsonValue::Array(arr) = json {
                return Ok(arr.clone());
            } else {
                return Err(ExtractError::ApiRequest(
                    "Response is not an array and no response_path specified".into(),
                ));
            }
        }

        // Simple path extraction (e.g., "data" or "result.items")
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = json;

        for part in parts {
            match current {
                JsonValue::Object(obj) => {
                    current = obj.get(part).ok_or_else(|| {
                        ExtractError::ApiRequest(format!(
                            "Path component '{}' not found in response",
                            part
                        ))
                    })?;
                }
                _ => {
                    return Err(ExtractError::ApiRequest(format!(
                        "Cannot navigate path '{}' - not an object",
                        part
                    )))
                }
            }
        }

        // Final value should be an array
        match current {
            JsonValue::Array(arr) => Ok(arr.clone()),
            _ => Err(ExtractError::ApiRequest(
                "Final path value is not an array".into(),
            )),
        }
    }

    /// Fetches a single page of data from the API.
    async fn fetch_page(
        &self,
        options: &RestApiOptions,
        page_params: HashMap<String, String>,
    ) -> Result<JsonValue, ExtractError> {
        let mut request = match options.method.to_uppercase().as_str() {
            "GET" => self.client.get(&options.url),
            "POST" => {
                let mut req = self.client.post(&options.url);
                if let Some(body) = &options.body {
                    req = req.body(body.clone());
                }
                req
            }
            method => {
                return Err(ExtractError::ApiRequest(format!(
                    "Unsupported HTTP method: {}",
                    method
                )))
            }
        };

        // Add headers
        for (key, value) in &options.headers {
            request = request.header(key, value);
        }

        // Add query params (base + pagination)
        let mut all_params = options.query_params.clone();
        all_params.extend(page_params);
        request = request.query(&all_params);

        // Execute request
        let response = request
            .send()
            .await
            .map_err(|e| ExtractError::ApiRequest(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExtractError::ApiRequest(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let json: JsonValue = response.json().await.map_err(|e| {
            ExtractError::ApiRequest(format!("Failed to parse JSON response: {}", e))
        })?;

        Ok(json)
    }
}

impl Default for RestApiExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Extractor for RestApiExtractor {
    fn name(&self) -> &'static str {
        "rest_api"
    }

    fn supported_types(&self) -> &[&'static str] {
        &["rest_api", "api", "http"]
    }

    fn validate_config(&self, config: &SourceConfig) -> Result<(), ConfigError> {
        match config {
            SourceConfig::RestApi { rest_api } => {
                if rest_api.url.is_empty() {
                    return Err(ConfigError::Validation("API URL cannot be empty".into()));
                }

                // Validate pagination config if present
                if let Some(pagination) = &rest_api.pagination {
                    if pagination.response_path.is_empty() {
                        return Err(ConfigError::Validation(
                            "Pagination response_path cannot be empty".into(),
                        ));
                    }
                }

                Ok(())
            }
            _ => Err(ConfigError::Validation(
                "Invalid config type for REST API extractor".into(),
            )),
        }
    }

    async fn extract(&self, config: &SourceConfig) -> Result<RecordStream, ExtractError> {
        let rest_api = match config {
            SourceConfig::RestApi { rest_api } => rest_api.clone(),
            _ => {
                return Err(ExtractError::UnsupportedType(
                    "REST API extractor requires REST API source config".into(),
                ))
            }
        };

        let mut all_records = Vec::new();

        // Check if pagination is configured
        if let Some(pagination) = &rest_api.pagination {
            // Paginated extraction
            match pagination.page_type {
                PaginationType::Offset => {
                    let mut offset = 0;
                    let offset_param = pagination
                        .offset_param
                        .as_ref()
                        .ok_or_else(|| {
                            ExtractError::ApiRequest(
                                "offset_param required for offset pagination".into(),
                            )
                        })?
                        .clone();

                    loop {
                        let mut params = HashMap::new();
                        params.insert(pagination.limit_param.clone(), pagination.limit.to_string());
                        params.insert(offset_param.clone(), offset.to_string());

                        let response = self.fetch_page(&rest_api, params).await?;
                        let records = Self::extract_records_from_response(
                            &response,
                            &pagination.response_path,
                        )?;

                        if records.is_empty() {
                            break;
                        }

                        let batch_size = records.len();
                        for (idx, json_record) in records.into_iter().enumerate() {
                            if let JsonValue::Object(obj) = json_record {
                                let mut record = Record::new();
                                record.add_metadata("source_offset", (offset + idx).to_string());

                                for (key, value) in obj {
                                    record.insert(key, Self::json_to_value(&value));
                                }

                                all_records.push(Ok(record));
                            }
                        }

                        offset += batch_size;

                        // If we got fewer records than the limit, we've reached the end
                        if batch_size < pagination.limit {
                            break;
                        }
                    }
                }
                PaginationType::Page => {
                    let mut page = 1;
                    let page_param = pagination
                        .page_param
                        .as_ref()
                        .ok_or_else(|| {
                            ExtractError::ApiRequest(
                                "page_param required for page pagination".into(),
                            )
                        })?
                        .clone();

                    loop {
                        let mut params = HashMap::new();
                        params.insert(pagination.limit_param.clone(), pagination.limit.to_string());
                        params.insert(page_param.clone(), page.to_string());

                        let response = self.fetch_page(&rest_api, params).await?;
                        let records = Self::extract_records_from_response(
                            &response,
                            &pagination.response_path,
                        )?;

                        if records.is_empty() {
                            break;
                        }

                        let batch_size = records.len();
                        for json_record in records {
                            if let JsonValue::Object(obj) = json_record {
                                let mut record = Record::new();
                                record.add_metadata("source_page", page.to_string());

                                for (key, value) in obj {
                                    record.insert(key, Self::json_to_value(&value));
                                }

                                all_records.push(Ok(record));
                            }
                        }

                        page += 1;

                        if batch_size < pagination.limit {
                            break;
                        }
                    }
                }
                PaginationType::Cursor => {
                    // Cursor-based pagination not fully implemented yet
                    // Would need to extract cursor from response and use it for next request
                    return Err(ExtractError::ApiRequest(
                        "Cursor-based pagination not yet implemented".into(),
                    ));
                }
            }
        } else {
            // Single request, no pagination
            let response = self.fetch_page(&rest_api, HashMap::new()).await?;

            // Try to extract as array
            let records = if let JsonValue::Array(arr) = response {
                arr
            } else if let JsonValue::Object(obj) = &response {
                // Try common response paths
                if let Some(JsonValue::Array(arr)) = obj.get("data") {
                    arr.clone()
                } else if let Some(JsonValue::Array(arr)) = obj.get("results") {
                    arr.clone()
                } else if let Some(JsonValue::Array(arr)) = obj.get("items") {
                    arr.clone()
                } else {
                    return Err(ExtractError::ApiRequest(
                        "Response is not an array and no pagination configured".into(),
                    ));
                }
            } else {
                return Err(ExtractError::ApiRequest(
                    "Response is not an object or array".into(),
                ));
            };

            for json_record in records {
                if let JsonValue::Object(obj) = json_record {
                    let mut record = Record::new();

                    for (key, value) in obj {
                        record.insert(key, Self::json_to_value(&value));
                    }

                    all_records.push(Ok(record));
                }
            }
        }

        Ok(Box::pin(stream::iter(all_records)))
    }

    async fn estimate_count(&self, _config: &SourceConfig) -> Result<Option<u64>, ExtractError> {
        // Cannot estimate count for API without fetching
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_api_extractor_creation() {
        let extractor = RestApiExtractor::new();
        assert_eq!(extractor.name(), "rest_api");
        assert!(extractor.supported_types().contains(&"api"));
    }

    #[test]
    fn test_json_to_value_conversion() {
        let json = serde_json::json!({
            "name": "John",
            "age": 30,
            "active": true,
            "tags": ["a", "b"]
        });

        let value = RestApiExtractor::json_to_value(&json);
        if let Value::Object(obj) = value {
            assert_eq!(obj.get("name"), Some(&Value::String("John".into())));
            assert_eq!(obj.get("age"), Some(&Value::Number(30.0)));
            assert_eq!(obj.get("active"), Some(&Value::Bool(true)));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_extract_records_from_response() {
        let json = serde_json::json!({
            "data": [
                {"id": 1, "name": "John"},
                {"id": 2, "name": "Jane"}
            ]
        });

        let records = RestApiExtractor::extract_records_from_response(&json, "data").unwrap();
        assert_eq!(records.len(), 2);
    }
}
