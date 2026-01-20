/// Integration tests for REST API extraction
mod common;

use basevn_migro::config::JobConfig;
use basevn_migro::core::record::Record;
use basevn_migro::extract::registry::ExtractorRegistry;
use futures::StreamExt;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_rest_api_extraction_simple() {
    common::setup();

    // Start mock server
    let mock_server = MockServer::start().await;

    // Setup mock response
    let response_body = serde_json::json!([
        {
            "id": 1,
            "name": "Alice API",
            "email": "alice@api.com",
            "role": "developer"
        },
        {
            "id": 2,
            "name": "Bob API",
            "email": "bob@api.com",
            "role": "designer"
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    // Create config with mock server URL
    let config = format!(
        r#"
version: "1"
job:
  id: "test"
  name: "Test"
source:
  type: restapi
  url: "{}/users"
  method: "GET"
target:
  type: basevn
  base_url: "http://localhost"
  app: "test"
  entity: "test"
  auth:
    type: access_token
    token: "test"
mapping:
  fields:
    - source: "test"
      target: "test"
      required: false
"#,
        mock_server.uri()
    );

    let config = serde_yaml::from_str::<JobConfig>(&config).expect("Failed to load config");
    let registry = ExtractorRegistry::default();

    let mut stream = registry
        .extract(&config.source)
        .await
        .expect("Failed to extract");

    let mut records: Vec<Record> = Vec::new();
    while let Some(result) = stream.next().await {
        records.push(result.expect("Failed to read record"));
    }

    assert_eq!(records.len(), 2, "Should extract 2 API records");

    // Validate first record
    let first = &records[0];
    assert_eq!(
        first.get("name").and_then(|v| v.as_string()),
        Some("Alice API".to_string())
    );
    assert_eq!(
        first.get("email").and_then(|v| v.as_string()),
        Some("alice@api.com".to_string())
    );
}

#[tokio::test]
async fn test_rest_api_extraction_with_nested_path() {
    common::setup();

    let mock_server = MockServer::start().await;

    // Response with nested data path
    let response_body = serde_json::json!({
        "status": "success",
        "data": {
            "items": [
                {"id": 1, "title": "Item 1"},
                {"id": 2, "title": "Item 2"},
                {"id": 3, "title": "Item 3"}
            ]
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let config = format!(
        r#"
version: "1"
job:
  id: "test"
  name: "Test"
source:
  type: restapi
  url: "{}/api/items"
  method: "GET"
  pagination:
    type: offset
    limit_param: "limit"
    offset_param: "offset"
    limit: 100
    response_path: "data.items"
target:
  type: basevn
  base_url: "http://localhost"
  app: "test"
  entity: "test"
  auth:
    type: access_token
    token: "test"
mapping:
  fields:
    - source: "test"
      target: "test"
      required: false
"#,
        mock_server.uri()
    );

    let config = serde_yaml::from_str::<JobConfig>(&config).expect("Failed to load config");
    let registry = ExtractorRegistry::default();

    let mut stream = registry
        .extract(&config.source)
        .await
        .expect("Failed to extract");

    let mut records: Vec<Record> = Vec::new();
    while let Some(result) = stream.next().await {
        records.push(result.expect("Failed to read record"));
    }

    assert_eq!(records.len(), 3, "Should extract 3 items from nested path");
}

#[tokio::test]
async fn test_rest_api_extraction_with_headers() {
    common::setup();

    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!([
        {"id": 1, "secret": "protected"}
    ]);

    Mock::given(method("GET"))
        .and(path("/protected"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let config = format!(
        r#"
version: "1"
job:
  id: "test"
  name: "Test"
source:
  type: restapi
  url: "{}/protected"
  method: "GET"
  headers:
    Authorization: "Bearer test-token"
    X-Custom-Header: "test-value"
target:
  type: basevn
  base_url: "http://localhost"
  app: "test"
  entity: "test"
  auth:
    type: access_token
    token: "test"
mapping:
  fields:
    - source: "test"
      target: "test"
      required: false
"#,
        mock_server.uri()
    );

    let config = serde_yaml::from_str::<JobConfig>(&config).expect("Failed to load config");
    let registry = ExtractorRegistry::default();

    let mut stream = registry
        .extract(&config.source)
        .await
        .expect("Failed to extract");

    let mut records: Vec<Record> = Vec::new();
    while let Some(result) = stream.next().await {
        records.push(result.expect("Failed to read record"));
    }

    assert_eq!(records.len(), 1, "Should extract protected resource");
}

#[tokio::test]
async fn test_rest_api_extraction_offset_pagination() {
    common::setup();

    let mock_server = MockServer::start().await;

    // First page
    let page1 = serde_json::json!([
        {"id": 1, "name": "Item 1"},
        {"id": 2, "name": "Item 2"}
    ]);

    // Second page
    let page2 = serde_json::json!([
        {"id": 3, "name": "Item 3"},
        {"id": 4, "name": "Item 4"}
    ]);

    // Empty third page (signals end)
    let page3 = serde_json::json!([]);

    Mock::given(method("GET"))
        .and(path("/items"))
        .respond_with(move |req: &wiremock::Request| {
            let query = req.url.query().unwrap_or("");
            if query.contains("offset=0") || !query.contains("offset") {
                ResponseTemplate::new(200).set_body_json(&page1)
            } else if query.contains("offset=2") {
                ResponseTemplate::new(200).set_body_json(&page2)
            } else {
                ResponseTemplate::new(200).set_body_json(&page3)
            }
        })
        .mount(&mock_server)
        .await;

    let config = format!(
        r#"
version: "1"
job:
  id: "test"
  name: "Test"
source:
  type: restapi
  url: "{}/items"
  method: "GET"
  pagination:
    type: offset
    limit_param: "limit"
    offset_param: "offset"
    limit: 2
    response_path: ""
target:
  type: basevn
  base_url: "http://localhost"
  app: "test"
  entity: "test"
  auth:
    type: access_token
    token: "test"
mapping:
  fields:
    - source: "test"
      target: "test"
      required: false
"#,
        mock_server.uri()
    );

    let config = serde_yaml::from_str::<JobConfig>(&config).expect("Failed to load config");
    let registry = ExtractorRegistry::default();

    let mut stream = registry
        .extract(&config.source)
        .await
        .expect("Failed to extract");

    let mut records: Vec<Record> = Vec::new();
    while let Some(result) = stream.next().await {
        records.push(result.expect("Failed to read record"));
    }

    assert_eq!(
        records.len(),
        4,
        "Should extract all paginated records (2 pages)"
    );
}
