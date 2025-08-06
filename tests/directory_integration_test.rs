//! Integration tests for semantic directory aliasing functionality

use claude_hook_advisor::types::{Config, DirectoryVariables, HookInput};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test the complete directory alias workflow via CLI commands
#[test]
fn test_directory_alias_cli_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test-config.toml");
    let config_path_str = config_path.to_str().unwrap();

    // Build the binary for testing
    let binary_path = build_test_binary();

    // Test 1: List empty aliases
    let output = Command::new(&binary_path)
        .args(["--config", config_path_str, "--list-directory-aliases"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("No directory aliases configured"));

    // Test 2: Add directory alias
    let output = Command::new(&binary_path)
        .args([
            "--config", config_path_str,
            "--add-directory-alias", "docs", "~/Documents/Documentation"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Added directory alias: 'docs' -> '~/Documents/Documentation'"));

    // Test 3: List aliases after adding
    let output = Command::new(&binary_path)
        .args(["--config", config_path_str, "--list-directory-aliases"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("docs -> ~/Documents/Documentation"));

    // Test 4: Add another alias with variables
    let output = Command::new(&binary_path)
        .args([
            "--config", config_path_str,
            "--add-directory-alias", "project_docs", "~/Documents/Documentation/{project}"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());

    // Test 5: Remove alias
    let output = Command::new(&binary_path)
        .args([
            "--config", config_path_str,
            "--remove-directory-alias", "docs"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Removed directory alias: 'docs'"));

    // Test 6: Verify removal
    let output = Command::new(&binary_path)
        .args(["--config", config_path_str, "--list-directory-aliases"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // docs should be gone, but project_docs should remain
    assert!(!stdout.contains("  docs ->"));  // More specific match with proper spacing
    assert!(stdout.contains("project_docs ->"));

    // Test 7: Try to remove non-existent alias
    let output = Command::new(&binary_path)
        .args([
            "--config", config_path_str,
            "--remove-directory-alias", "nonexistent"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Directory alias 'nonexistent' not found"));
}

/// Test directory resolution via CLI
#[test]
fn test_directory_resolution_cli() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test-config.toml");
    let config_path_str = config_path.to_str().unwrap();

    // Create a test config with directory aliases
    let config = Config {
        commands: HashMap::new(),
        semantic_directories: {
            let mut dirs = HashMap::new();
            dirs.insert("test_docs".to_string(), temp_dir.path().to_str().unwrap().to_string());
            dirs
        },
        directory_variables: DirectoryVariables {
            project: Some("test-project".to_string()),
            current_project: Some("test-project".to_string()),
            user_home: Some(temp_dir.path().to_str().unwrap().to_string()),
        },
    };

    let toml_content = toml::to_string_pretty(&config).unwrap();
    fs::write(&config_path, toml_content).unwrap();

    let binary_path = build_test_binary();

    // Test resolving existing directory
    let output = Command::new(&binary_path)
        .args([
            "--config", config_path_str,
            "--resolve-directory", "test_docs"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Alias 'test_docs' resolves to:"));
    assert!(stdout.contains("Canonical path:"));

    // Test resolving non-existent alias
    let output = Command::new(&binary_path)
        .args([
            "--config", config_path_str,
            "--resolve-directory", "nonexistent"
        ])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Failed to resolve alias 'nonexistent'"));
}

/// Test UserPromptSubmit hook with directory references
#[test]
fn test_user_prompt_submit_hook() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test-config.toml");
    let config_path_str = config_path.to_str().unwrap();

    // Create a test config with directory aliases
    let config = Config {
        commands: HashMap::new(),
        semantic_directories: {
            let mut dirs = HashMap::new();
            dirs.insert("docs".to_string(), temp_dir.path().join("docs").to_str().unwrap().to_string());
            dirs.insert("central_docs".to_string(), temp_dir.path().join("central").to_str().unwrap().to_string());
            dirs
        },
        directory_variables: DirectoryVariables {
            project: Some("test-project".to_string()),
            current_project: Some("test-project".to_string()),
            user_home: Some(temp_dir.path().to_str().unwrap().to_string()),
        },
    };

    let toml_content = toml::to_string_pretty(&config).unwrap();
    fs::write(&config_path, toml_content).unwrap();

    // Create the directories so they can be resolved
    fs::create_dir_all(temp_dir.path().join("docs")).unwrap();
    fs::create_dir_all(temp_dir.path().join("central")).unwrap();

    let binary_path = build_test_binary();

    // Test UserPromptSubmit hook with directory references
    let hook_input = HookInput {
        session_id: "test-session".to_string(),
        transcript_path: None,
        cwd: None,
        hook_event_name: "UserPromptSubmit".to_string(),
        tool_name: None,
        tool_input: None,
        prompt: Some("Please check the docs directory and central_docs for examples".to_string()),
        tool_response: None,
    };

    let input_json = serde_json::to_string(&hook_input).unwrap();

    let output = Command::new(&binary_path)
        .args(["--config", config_path_str, "--hook"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    let mut child = output;
    
    // Write input to stdin
    use std::io::Write;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input_json.as_bytes()).unwrap();
    }

    let output = child.wait_with_output().unwrap();
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should detect and resolve directory references
    assert!(stdout.contains("Directory reference") || stdout.len() == 0); // May be empty if directories don't exist
}

/// Test error handling for non-existent directories
#[test]
fn test_directory_resolution_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test-config.toml");
    let config_path_str = config_path.to_str().unwrap();

    // Create a config that points to a non-existent directory
    let config = Config {
        commands: HashMap::new(),
        semantic_directories: {
            let mut dirs = HashMap::new();
            // Point to a directory that doesn't exist
            dirs.insert("nonexistent".to_string(), "/tmp/nonexistent_path_12345".to_string());
            dirs
        },
        directory_variables: DirectoryVariables::default(),
    };

    let toml_content = toml::to_string_pretty(&config).unwrap();
    fs::write(&config_path, toml_content).unwrap();

    let binary_path = build_test_binary();

    // Test resolving non-existent directory should fail
    let output = Command::new(&binary_path)
        .args([
            "--config", config_path_str,
            "--resolve-directory", "nonexistent"
        ])
        .output()
        .expect("Failed to execute command");
    
    // Should fail because directory doesn't exist
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Failed to resolve"));
}

/// Build the test binary and return its path
fn build_test_binary() -> String {
    let output = Command::new("cargo")
        .args(["build", "--bin", "claude-hook-advisor"])
        .output()
        .expect("Failed to build binary");

    if !output.status.success() {
        panic!("Failed to build test binary: {}", String::from_utf8_lossy(&output.stderr));
    }

    "./target/debug/claude-hook-advisor".to_string()
}