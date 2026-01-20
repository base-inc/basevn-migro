/// End-to-end pipeline integration tests
mod common;

use basevn_migro::config::JobConfig;
use basevn_migro::config::{
    FieldMap, MappingConfig, MissingRequiredBehavior, UnmappedFieldBehavior,
};
use basevn_migro::core::record::Record;
use basevn_migro::extract::registry::ExtractorRegistry;
use basevn_migro::transform::field_filter::{FieldFilter, FilterMode};
use basevn_migro::transform::field_mapping::FieldMapper;
use basevn_migro::transform::traits::Transformer;
use futures::StreamExt;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_full_pipeline_csv_to_transformed() {
    common::setup();

    let csv_path = common::fixture_path("test-employees.csv");
    let config = format!(
        r#"
version: "1"
job:
  id: "test-pipeline"
  name: "Test Pipeline"
source:
  type: csv
  path: "{}"
  options:
    has_headers: true
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
        csv_path.to_string_lossy().replace('\\', "/")
    );

    let config = serde_yaml::from_str::<JobConfig>(&config).expect("Failed to load config");

    // Extract
    let extractor_registry = ExtractorRegistry::default();
    let mut stream = extractor_registry
        .extract(&config.source)
        .await
        .expect("Failed to extract");

    let mut records: Vec<Record> = Vec::new();
    while let Some(result) = stream.next().await {
        records.push(result.expect("Failed to read record"));
    }

    assert_eq!(records.len(), 3, "Should extract 3 records");

    // Transform: Field Mapping
    let field_mapper = FieldMapper::from_fields(vec![
        FieldMap {
            source: "Full Name".to_string(),
            target: "full_name".to_string(),
            required: true,
            default: None,
        },
        FieldMap {
            source: "Email".to_string(),
            target: "email".to_string(),
            required: true,
            default: None,
        },
        FieldMap {
            source: "Department".to_string(),
            target: "department".to_string(),
            required: false,
            default: Some("General".to_string()),
        },
    ]);

    let mut mapped_records = Vec::new();
    for record in records {
        match field_mapper.transform(record) {
            Ok(transformed) => mapped_records.push(transformed),
            Err(e) => panic!("Mapping failed: {}", e),
        }
    }

    assert_eq!(
        mapped_records.len(),
        3,
        "All records should map successfully"
    );

    // Transform: Field Filter (remove sensitive fields)
    let field_filter =
        FieldFilter::remove(vec!["Internal ID".to_string(), "Password Hash".to_string()]);

    let mut final_records: Vec<Record> = Vec::new();
    for record in mapped_records {
        match field_filter.transform(record) {
            Ok(transformed) => final_records.push(transformed),
            Err(e) => panic!("Filtering failed: {}", e),
        }
    }

    // Verify transformations
    for record in &final_records {
        // Should have mapped fields
        assert!(record.get("full_name").is_some());
        assert!(record.get("email").is_some());
        assert!(record.get("department").is_some());

        // Should NOT have filtered fields
        assert!(record.get("Internal ID").is_none());
        assert!(record.get("Password Hash").is_none());
    }

    // Verify first record details
    let first = &final_records[0];
    assert_eq!(
        first.get("full_name").and_then(|v| v.as_string()),
        Some("Alice Test".to_string())
    );
    assert_eq!(
        first.get("email").and_then(|v| v.as_string()),
        Some("alice.test@example.com".to_string())
    );
}

#[tokio::test]
async fn test_pipeline_with_field_defaults() {
    common::setup();

    let csv_path = common::fixture_path("test-minimal.csv");
    let config = format!(
        r#"
version: "1"
job:
  id: "test"
  name: "Test"
source:
  type: csv
  path: "{}"
  options:
    has_headers: true
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
        csv_path.to_string_lossy().replace('\\', "/")
    );

    let config = serde_yaml::from_str::<JobConfig>(&config).expect("Failed to load config");

    // Extract
    let extractor_registry = ExtractorRegistry::default();
    let mut stream = extractor_registry
        .extract(&config.source)
        .await
        .expect("Failed to extract");

    let mut records: Vec<Record> = Vec::new();
    while let Some(result) = stream.next().await {
        records.push(result.expect("Failed to read record"));
    }

    // Transform with defaults
    let field_mapper = FieldMapper::from_fields(vec![
        FieldMap {
            source: "name".to_string(),
            target: "full_name".to_string(),
            required: true,
            default: None,
        },
        FieldMap {
            source: "email".to_string(),
            target: "email".to_string(),
            required: true,
            default: None,
        },
        FieldMap {
            source: "department".to_string(),
            target: "department".to_string(),
            required: false,
            default: Some("Unknown".to_string()),
        },
    ]);

    let mut transformed_records = Vec::new();
    for record in records {
        match field_mapper.transform(record) {
            Ok(transformed) => transformed_records.push(transformed),
            Err(e) => panic!("Transformation failed: {}", e),
        }
    }

    // All records should have department field with default value
    for record in &transformed_records {
        let dept = record.get("department").expect("Department should exist");
        // Should use default since source doesn't have this field
        assert_eq!(dept.as_string(), Some("Unknown".to_string()));
    }
}

#[tokio::test]
async fn test_pipeline_field_filter_keep_mode() {
    common::setup();

    let csv_path = common::fixture_path("test-employees.csv");
    let config = format!(
        r#"
version: "1"
job:
  id: "test"
  name: "Test"
source:
  type: csv
  path: "{}"
  options:
    has_headers: true
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
        csv_path.to_string_lossy().replace('\\', "/")
    );

    let config = serde_yaml::from_str::<JobConfig>(&config).expect("Failed to load config");

    // Extract
    let extractor_registry = ExtractorRegistry::default();
    let mut stream = extractor_registry
        .extract(&config.source)
        .await
        .expect("Failed to extract");

    let mut records: Vec<Record> = Vec::new();
    while let Some(result) = stream.next().await {
        records.push(result.expect("Failed to read record"));
    }

    // Transform: Keep only specific fields
    let field_filter = FieldFilter::keep(vec![
        "Full Name".to_string(),
        "Email".to_string(),
        "Department".to_string(),
    ]);

    let mut transformed_records: Vec<Record> = Vec::new();
    for record in records {
        match field_filter.transform(record) {
            Ok(transformed) => transformed_records.push(transformed),
            Err(e) => panic!("Filtering failed: {}", e),
        }
    }

    // Verify only specified fields are kept
    for record in &transformed_records {
        assert_eq!(
            record.fields().len(),
            3,
            "Should only have 3 fields after keep filter"
        );
        assert!(record.get("Full Name").is_some());
        assert!(record.get("Email").is_some());
        assert!(record.get("Department").is_some());

        // Other fields should be removed
        assert!(record.get("Position").is_none());
        assert!(record.get("Start Date").is_none());
        assert!(record.get("Internal ID").is_none());
        assert!(record.get("Password Hash").is_none());
    }
}

#[tokio::test]
async fn test_pipeline_api_source() {
    common::setup();

    // Mock source API
    let source_server = MockServer::start().await;
    let response_body = serde_json::json!([
        {"id": 1, "name": "User 1", "email": "user1@example.com", "role": "admin"},
        {"id": 2, "name": "User 2", "email": "user2@example.com", "role": "user"}
    ]);

    Mock::given(method("GET"))
        .and(path("/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&source_server)
        .await;

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
        source_server.uri()
    );

    let config = serde_yaml::from_str::<JobConfig>(&config).expect("Failed to load config");

    // Extract
    let extractor_registry = ExtractorRegistry::default();
    let mut stream = extractor_registry
        .extract(&config.source)
        .await
        .expect("Failed to extract");

    let mut records: Vec<Record> = Vec::new();
    while let Some(result) = stream.next().await {
        records.push(result.expect("Failed to read record"));
    }

    assert_eq!(records.len(), 2);

    // Transform: Map and filter
    let field_mapper = FieldMapper::from_fields(vec![
        FieldMap {
            source: "name".to_string(),
            target: "full_name".to_string(),
            required: true,
            default: None,
        },
        FieldMap {
            source: "email".to_string(),
            target: "email".to_string(),
            required: true,
            default: None,
        },
    ]);

    let field_filter = FieldFilter::remove(vec!["id".to_string(), "role".to_string()]);

    let mut transformed_records: Vec<Record> = Vec::new();
    for record in records {
        let record = field_mapper.transform(record).expect("Mapping failed");
        let record = field_filter.transform(record).expect("Filtering failed");
        transformed_records.push(record);
    }

    // Verify transformation
    assert_eq!(transformed_records.len(), 2);
    for record in &transformed_records {
        assert!(record.get("full_name").is_some());
        assert!(record.get("email").is_some());
        assert!(record.get("id").is_none());
        assert!(record.get("role").is_none());
    }
}
