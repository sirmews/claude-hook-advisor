//! Configuration loading and management

use crate::types::Config;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Loads configuration from a TOML file.
/// 
/// If the config file doesn't exist, returns an empty configuration and logs
/// a warning to stderr. This allows the tool to work gracefully without config.
/// 
/// # Arguments
/// * `config_path` - Path to the .claude-hook-advisor.toml file
/// 
/// # Returns
/// * `Ok(Config)` - Loaded configuration or empty config if file not found
/// * `Err` - If file exists but cannot be read or parsed
pub fn load_config(config_path: &str) -> Result<Config> {
    if !Path::new(config_path).exists() {
        // Log warning to stderr when config file is not found
        eprintln!("Warning: Config file '{config_path}' not found. No command mappings will be applied.");
        return Ok(Config {
            commands: HashMap::new(),
            semantic_directories: HashMap::new(),
            features: Default::default(),
        });
    }

    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {config_path}"))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {config_path}"))?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading_missing_file() {
        // Test loading non-existent config file
        let result = load_config("non-existent-file.toml");
        assert!(result.is_ok()); // Should return empty config
        let config = result.unwrap();
        assert!(config.commands.is_empty());
    }
}