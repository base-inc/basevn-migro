//! Validate command handler.

use std::path::Path;

use anyhow::Result;

use crate::config::load_config;

/// Runs the validate command.
///
/// # Arguments
///
/// * `config_path` - Path to the configuration file to validate
pub async fn run_validate(config_path: &Path) -> Result<()> {
    tracing::info!("Validating configuration: {}", config_path.display());

    // Attempt to load and validate configuration
    match load_config(config_path) {
        Ok(config) => {
            println!("✓ Configuration is valid");
            println!("\nJob Details:");
            println!("  ID: {}", config.job.id);
            println!("  Name: {}", config.job.name);
            if let Some(desc) = &config.job.description {
                println!("  Description: {}", desc);
            }

            // Show source info
            println!("\nSource:");
            match &config.source {
                crate::config::SourceConfig::Csv { path, .. } => {
                    println!("  Type: CSV");
                    println!("  Path: {}", path);
                }
                crate::config::SourceConfig::Excel { path, .. } => {
                    println!("  Type: Excel");
                    println!("  Path: {}", path);
                }
                crate::config::SourceConfig::RestApi { rest_api } => {
                    println!("  Type: REST API");
                    println!("  URL: {}", rest_api.url);
                }
            }

            // Show target info
            println!("\nTarget:");
            match &config.target {
                crate::config::TargetConfig::BaseVn { basevn } => {
                    println!("  Type: Base.vn");
                    println!("  App: {}", basevn.app);
                    println!("  Entity: {}", basevn.entity);
                }
            }

            // Show field mappings count
            println!(
                "\nField Mappings: {} configured",
                config.mapping.fields.len()
            );

            Ok(())
        }
        Err(e) => {
            println!("✗ Configuration validation failed");
            println!("\nError: {}", e);
            Err(e.into())
        }
    }
}
