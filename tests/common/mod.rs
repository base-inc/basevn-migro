/// Common utilities for integration tests

use std::path::PathBuf;

/// Get the path to a fixture file
pub fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(filename)
}

/// Read fixture file contents
pub fn read_fixture(filename: &str) -> String {
    std::fs::read_to_string(fixture_path(filename))
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", filename, e))
}

/// Create a temporary directory for test outputs
pub fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Setup test environment
pub fn setup() {
    // Initialize tracing for tests (with filter to reduce noise)
    let _ = tracing_subscriber::fmt()
        .with_env_filter("basevn_migro=debug")
        .with_test_writer()
        .try_init();
}
