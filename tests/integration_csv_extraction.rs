/// Integration tests for CSV extraction
mod common;

use basevn_migro::config::JobConfig;
use basevn_migro::core::record::Record;
use basevn_migro::extract::registry::ExtractorRegistry;
use futures::StreamExt;

#[tokio::test]
async fn test_csv_extraction_with_headers() {
    common::setup();

    let csv_path = common::fixture_path("test-employees.csv");
    // Convert path to string, replacing backslashes with forward slashes for cross-platform compatibility
    let csv_path_str = csv_path.to_string_lossy().replace('\\', "/");
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
    delimiter: ","
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
        csv_path_str
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

    assert_eq!(records.len(), 3, "Should extract 3 employee records");

    // Validate first record
    let first = &records[0];
    assert_eq!(
        first.get("Full Name").and_then(|v| v.as_string()),
        Some("Alice Test".to_string())
    );
    assert_eq!(
        first.get("Email").and_then(|v| v.as_string()),
        Some("alice.test@example.com".to_string())
    );
    assert_eq!(
        first.get("Department").and_then(|v| v.as_string()),
        Some("Engineering".to_string())
    );
}

#[tokio::test]
async fn test_csv_extraction_field_count() {
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
    let registry = ExtractorRegistry::default();

    let mut stream = registry
        .extract(&config.source)
        .await
        .expect("Failed to extract");

    let mut records: Vec<Record> = Vec::new();
    while let Some(result) = stream.next().await {
        records.push(result.expect("Failed to read record"));
    }

    // All records should have same number of fields
    for record in &records {
        assert_eq!(record.fields().len(), 7, "Each record should have 7 fields");
    }
}

#[tokio::test]
async fn test_csv_extraction_minimal() {
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
    let registry = ExtractorRegistry::default();

    let mut stream = registry
        .extract(&config.source)
        .await
        .expect("Failed to extract");

    let mut records: Vec<Record> = Vec::new();
    while let Some(result) = stream.next().await {
        records.push(result.expect("Failed to read record"));
    }

    assert_eq!(records.len(), 2, "Should extract 2 minimal records");

    // Validate fields exist
    for record in &records {
        assert!(record.get("name").is_some());
        assert!(record.get("email").is_some());
    }
}

#[tokio::test]
async fn test_csv_estimate_count() {
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
    let registry = ExtractorRegistry::default();

    let estimate = registry
        .estimate_count(&config.source)
        .await
        .expect("Failed to estimate count");

    assert!(estimate.is_some(), "CSV should provide count estimate");
    assert_eq!(estimate.unwrap(), 3, "Estimate should be 3 records");
}
