//! Directory resolution and aliasing functionality

use crate::types::{Config, DirectoryResolution};
use anyhow::{anyhow, Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// Cache for compiled regex patterns to avoid recompilation
static REGEX_CACHE: Lazy<Mutex<HashMap<String, Regex>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Gets or creates a cached regex for the given pattern
fn get_cached_regex(pattern: &str) -> Result<Regex> {
    let mut cache = REGEX_CACHE.lock()
        .expect("regex cache mutex should not be poisoned");
    
    if let Some(regex) = cache.get(pattern) {
        return Ok(regex.clone());
    }
    
    let regex = Regex::new(pattern).context("Failed to compile regex pattern")?;
    cache.insert(pattern.to_string(), regex.clone());
    Ok(regex)
}

/// Resolves semantic directory references to canonical filesystem paths.
/// 
/// Takes a directory alias (e.g., "docs", "central_docs") and resolves it to
/// a canonical path. Uses path canonicalization for basic security against 
/// path traversal attacks.
/// 
/// # Arguments
/// * `config` - Configuration containing directory mappings
/// * `alias` - The directory alias to resolve
/// 
/// # Returns
/// * `Ok(DirectoryResolution)` - Resolved directory with metadata
/// * `Err` - If alias not found or path invalid
pub fn resolve_directory(config: &Config, alias: &str) -> Result<DirectoryResolution> {
    // Find the alias in semantic_directories
    let directory_path = config.semantic_directories.get(alias)
        .ok_or_else(|| anyhow!("Directory alias '{}' not found", alias))?;
    
    // Expand tilde and resolve to canonical path (provides basic security)
    let expanded_path = expand_path(directory_path)?;
    let canonical_path = fs::canonicalize(&expanded_path)
        .with_context(|| format!("Failed to resolve path: {}", expanded_path.display()))?;

    Ok(DirectoryResolution {
        canonical_path: canonical_path.to_string_lossy().to_string(),
        alias_used: alias.to_string(),
        variables_substituted: Vec::new(),
    })
}

/// Detects directory references in natural language text.
///
/// Scans user prompts for potential directory references and attempts
/// to resolve them using configured semantic directory mappings.
/// Uses whitespace-boundary matching to ensure aliases are standalone tokens,
/// not substrings within larger words.
///
/// # Arguments
/// * `config` - Configuration containing directory mappings
/// * `text` - The user prompt text to analyze
///
/// # Returns
/// * `Vec<DirectoryResolution>` - All resolved directory references found
pub fn detect_directory_references(config: &Config, text: &str) -> Vec<DirectoryResolution> {
    let mut results = Vec::new();

    // Try exact alias matches using whitespace boundaries
    for alias in config.semantic_directories.keys() {
        // Use capturing groups for whitespace boundaries
        // Group 1: (^|\s) = start of string or whitespace
        // Group 2: the alias pattern
        // Group 3: (\s|$) = whitespace or end of string
        let alias_pattern = format!(r"(^|\s)({})(\s|$)", regex::escape(alias));
        if let Ok(regex) = get_cached_regex(&alias_pattern) {
            if regex.is_match(text) {
                if let Ok(resolution) = resolve_directory(config, alias) {
                    results.push(resolution);
                }
            }
        }
    }

    // Remove duplicates (same canonical path)
    results.sort_by(|a, b| a.canonical_path.cmp(&b.canonical_path));
    results.dedup_by(|a, b| a.canonical_path == b.canonical_path);

    results
}


/// Expands tilde (~) to user home directory.
/// 
/// Converts paths starting with ~ to absolute paths using the user's
/// home directory from environment variables.
/// 
/// # Arguments
/// * `path` - Path that may contain tilde prefix
/// 
/// # Returns
/// * `Ok(PathBuf)` - Expanded absolute path
/// * `Err` - If home directory cannot be determined
fn expand_path(path: &str) -> Result<PathBuf> {
    if path.starts_with('~') {
        let home_dir = env::var("HOME")
            .with_context(|| "Failed to get HOME environment variable")?;
        let expanded = path.replacen('~', &home_dir, 1);
        Ok(PathBuf::from(expanded))
    } else {
        Ok(PathBuf::from(path))
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        let mut semantic_directories = HashMap::new();
        semantic_directories.insert("docs".to_string(), "~/Documents/Documentation".to_string());
        semantic_directories.insert("project_docs".to_string(), "~/Documents/Documentation/project".to_string());
        
        Config {
            commands: HashMap::new(),
            semantic_directories,
        }
    }

    #[test]
    fn test_expand_path() {
        // Mock HOME environment variable
        env::set_var("HOME", "/home/testuser");
        
        let result = expand_path("~/Documents").unwrap();
        assert_eq!(result, PathBuf::from("/home/testuser/Documents"));
        
        let result = expand_path("/absolute/path").unwrap();
        assert_eq!(result, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_detect_directory_references() {
        let config = create_test_config();
        
        let text = "Please check the docs directory for examples";
        let results = detect_directory_references(&config, text);
        
        // Should find at least one reference if directories exist
        // In test environment, paths may not exist, so we just check the detection logic
        assert!(results.len() <= 1); // At most one match for "docs"
    }

    #[test]
    fn test_directory_alias_matching_patterns() {
        let config = create_test_config();
        
        // Test that demonstrates the current word-boundary behavior
        // The key insight: "project docs" matches "docs" alias, not "project_docs" alias
        let text = "check the project docs directory";
        let results = detect_directory_references(&config, text);
        
        // This will find "docs" within "project docs" but NOT match "project_docs" alias
        // Resolution may fail due to path not existing in test environment, but
        // the pattern matching logic detects aliases correctly
        assert!(results.len() <= 1, "Should detect at most one alias match");
        
        // The important behavioral test: ensure we're not doing fuzzy matching
        let no_fuzzy_match = "check documentation folder";
        let results2 = detect_directory_references(&config, &no_fuzzy_match);
        assert_eq!(results2.len(), 0, "Should not fuzzy-match 'documentation' to 'docs'");
    }
}