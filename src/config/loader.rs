//! Configuration file loading.

use std::path::Path;

use crate::error::ConfigError;

use super::JobConfig;

/// Loads job configuration from a YAML or JSON file.
///
/// # Arguments
///
/// * `path` - Path to the configuration file
///
/// # Returns
///
/// Parsed `JobConfig` or a `ConfigError`.
///
/// # Errors
///
/// Returns error if:
/// - File cannot be read
/// - File format is invalid (not YAML or JSON)
/// - Configuration validation fails
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<JobConfig, ConfigError> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(|source| ConfigError::FileRead {
        path: path.display().to_string(),
        source,
    })?;

    // Try parsing as YAML first (supports both YAML and JSON)
    let config: JobConfig = serde_yaml::from_str(&content)?;

    // Basic validation
    validate_config(&config)?;

    Ok(config)
}

/// Validates a job configuration.
fn validate_config(config: &JobConfig) -> Result<(), ConfigError> {
    // Validate job ID is not empty
    if config.job.id.is_empty() {
        return Err(ConfigError::Validation("job.id cannot be empty".into()));
    }

    // Validate job name is not empty
    if config.job.name.is_empty() {
        return Err(ConfigError::Validation("job.name cannot be empty".into()));
    }

    // Validate at least one field mapping exists
    if config.mapping.fields.is_empty() {
        return Err(ConfigError::Validation(
            "At least one field mapping is required".into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_yaml_config() {
        let yaml = r#"
version: "1"
job:
  id: "test-job"
  name: "Test Job"

source:
  type: csv
  path: "./test.csv"

target:
  type: basevn
  base_url: "https://api.base.vn"
  app: "hrm"
  entity: "employees"
  auth:
    type: access_token
    token: "test_token"

mapping:
  fields:
    - source: "name"
      target: "full_name"
      required: true
"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(yaml.as_bytes())
            .expect("Failed to write to temp file");

        let config = load_config(temp_file.path()).expect("Failed to load config");
        assert_eq!(config.job.id, "test-job");
        assert_eq!(config.job.name, "Test Job");
    }

    #[test]
    fn test_load_invalid_config_missing_job_id() {
        let yaml = r#"
version: "1"
job:
  id: ""
  name: "Test Job"

source:
  type: csv
  path: "./test.csv"

target:
  type: basevn
  app: "hrm"
  entity: "employees"
  auth:
    type: access_token
    token: "test_token"

mapping:
  fields:
    - source: "name"
      target: "full_name"
"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(yaml.as_bytes())
            .expect("Failed to write to temp file");

        let result = load_config(temp_file.path());
        assert!(result.is_err());
        if let Err(e) = result {
            matches!(e, ConfigError::Validation(_));
        }
    }
}
