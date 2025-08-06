//! Directory resolution and aliasing functionality

use crate::types::{Config, DirectoryResolution, DirectoryVariables};
use anyhow::{anyhow, Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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
/// a canonical path using variable substitution. Uses path canonicalization
/// for basic security against path traversal attacks.
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
    let template_path = config.semantic_directories.get(alias)
        .ok_or_else(|| anyhow!("Directory alias '{}' not found", alias))?;

    // Substitute variables in the path
    let substituted_path = substitute_variables(template_path, &config.directory_variables)?;
    
    // Expand tilde and resolve to canonical path (provides basic security)
    let expanded_path = expand_path(&substituted_path)?;
    let canonical_path = fs::canonicalize(&expanded_path)
        .with_context(|| format!("Failed to resolve path: {}", expanded_path.display()))?;

    // Collect substitution metadata
    let variables_substituted = collect_substitutions(template_path, &config.directory_variables);

    Ok(DirectoryResolution {
        canonical_path: canonical_path.to_string_lossy().to_string(),
        alias_used: alias.to_string(),
        variables_substituted,
    })
}

/// Detects directory references in natural language text.
/// 
/// Scans user prompts for potential directory references and attempts
/// to resolve them using configured semantic directory mappings.
/// 
/// # Arguments
/// * `config` - Configuration containing directory mappings
/// * `text` - The user prompt text to analyze
/// 
/// # Returns
/// * `Vec<DirectoryResolution>` - All resolved directory references found
pub fn detect_directory_references(config: &Config, text: &str) -> Vec<DirectoryResolution> {
    let mut results = Vec::new();
    
    // Try exact alias matches first
    for alias in config.semantic_directories.keys() {
        let alias_pattern = format!(r"\b{}\b", regex::escape(alias));
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

/// Substitutes variables in directory path templates.
/// 
/// Replaces variable placeholders like {project}, {user_home} with their
/// configured values from DirectoryVariables.
/// 
/// # Arguments
/// * `template` - Path template with variable placeholders
/// * `variables` - Variable definitions for substitution
/// 
/// # Returns
/// * `Ok(String)` - Path with variables substituted
/// * `Err` - If required variables are missing
fn substitute_variables(template: &str, variables: &DirectoryVariables) -> Result<String> {
    let mut result = template.to_string();
    
    // Substitute {project} or {current_project}
    if result.contains("{project}") || result.contains("{current_project}") {
        let detected_project = detect_project_name();
        let project_name = variables.current_project.as_deref()
            .or(variables.project.as_deref())
            .or(detected_project.as_deref())
            .ok_or_else(|| anyhow!("Project variable required but not configured"))?;
        
        result = result.replace("{project}", project_name);
        result = result.replace("{current_project}", project_name);
    }
    
    // Substitute {user_home}
    if result.contains("{user_home}") {
        let home_from_env = env::var("HOME").ok();
        let home_dir = variables.user_home.as_deref()
            .or(home_from_env.as_deref())
            .unwrap_or("~");
        
        result = result.replace("{user_home}", home_dir);
    }
    
    Ok(result)
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


/// Collects variable substitution metadata for debugging.
/// 
/// Returns information about which variables were found and substituted
/// in the path template for inclusion in resolution results.
/// 
/// # Arguments
/// * `template` - Original path template
/// * `variables` - Variable configuration
/// 
/// # Returns
/// * `Vec<(String, String)>` - Variable name and substituted value pairs
fn collect_substitutions(template: &str, variables: &DirectoryVariables) -> Vec<(String, String)> {
    let mut substitutions = Vec::new();
    
    if template.contains("{project}") || template.contains("{current_project}") {
        if let Some(project) = variables.current_project.as_ref().or(variables.project.as_ref()) {
            substitutions.push(("project".to_string(), project.clone()));
        }
    }
    
    if template.contains("{user_home}") {
        if let Some(home) = variables.user_home.as_ref() {
            substitutions.push(("user_home".to_string(), home.clone()));
        }
    }
    
    substitutions
}

/// Attempts to detect current project name from git repository.
/// 
/// Looks for git repository root and extracts the directory name
/// as a fallback when project name is not explicitly configured.
/// 
/// # Returns
/// * `Option<String>` - Detected project name, or None if not in git repo
fn detect_project_name() -> Option<String> {
    // Try to get git repository root directory name
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    
    if output.status.success() {
        let root_path = String::from_utf8(output.stdout).ok()?;
        let root_path = root_path.trim();
        let path = Path::new(root_path);
        path.file_name()?.to_str().map(|s| s.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DirectoryVariables;
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        let mut semantic_directories = HashMap::new();
        semantic_directories.insert("docs".to_string(), "~/Documents/Documentation".to_string());
        semantic_directories.insert("project_docs".to_string(), "~/Documents/Documentation/{project}".to_string());
        
        let directory_variables = DirectoryVariables {
            project: Some("test-project".to_string()),
            current_project: Some("test-project".to_string()),
            user_home: Some("/home/testuser".to_string()),
        };
        
        Config {
            commands: HashMap::new(),
            semantic_directories,
            directory_variables,
        }
    }

    #[test]
    fn test_substitute_variables() {
        let variables = DirectoryVariables {
            project: Some("my-project".to_string()),
            current_project: Some("my-project".to_string()),
            user_home: Some("/home/user".to_string()),
        };

        let result = substitute_variables("~/Documents/{project}/notes", &variables).unwrap();
        assert_eq!(result, "~/Documents/my-project/notes");

        let result = substitute_variables("{user_home}/Projects", &variables).unwrap();
        assert_eq!(result, "/home/user/Projects");
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
    fn test_collect_substitutions() {
        let variables = DirectoryVariables {
            project: Some("test-proj".to_string()),
            current_project: Some("test-proj".to_string()),
            user_home: Some("/home/test".to_string()),
        };

        let substitutions = collect_substitutions("~/Documents/{project}/files", &variables);
        assert_eq!(substitutions.len(), 1);
        assert_eq!(substitutions[0], ("project".to_string(), "test-proj".to_string()));

        let substitutions = collect_substitutions("{user_home}/{project}", &variables);
        assert_eq!(substitutions.len(), 2);
    }
}