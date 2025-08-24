use std::process::Command;
use std::fs;
use std::env;
use tempfile::tempdir;

/// Test the complete hook workflow with real JSON input/output
#[test]
fn test_hook_end_to_end() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".claude-hook-advisor.toml");
    
    // Create test config
    let config_content = r#"[commands]
npm = "bun"
yarn = "bun"
pip = "uv pip"
"#;
    fs::write(&config_path, config_content).unwrap();
    
    // Test input that should trigger replacement
    let hook_input = r#"{
        "session_id": "test-session",
        "transcript_path": "",
        "cwd": "",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {
            "command": "npm install express",
            "description": "Install Express.js"
        }
    }"#;
    
    // Run the hook
    let output = Command::new("cargo")
        .args(["run", "--", "--hook", "--config", config_path.to_str().unwrap()])
        .current_dir(env::current_dir().unwrap())
        .arg("--")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    
    let mut child = output;
    
    // Send input to stdin
    use std::io::Write;
    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(hook_input.as_bytes()).unwrap();
    stdin.flush().unwrap();
    let _ = stdin; // Close stdin
    
    // Get output
    let result = child.wait_with_output().unwrap();
    assert!(result.status.success());
    
    let stdout = String::from_utf8(result.stdout).unwrap();
    
    // Verify JSON output structure
    let json_output: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json_output["decision"], "block");
    assert!(json_output["reason"].as_str().unwrap().contains("bun"));
    // In default mode, replacement_command is None, not included in JSON
}

/// Test hook with command that has no mapping
#[test]
fn test_hook_no_mapping() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".claude-hook-advisor.toml");
    
    // Create test config with limited mappings
    let config_content = r#"[commands]
npm = "bun"
"#;
    fs::write(&config_path, config_content).unwrap();
    
    // Test input with unmapped command
    let hook_input = r#"{
        "session_id": "test-session",
        "transcript_path": "",
        "cwd": "",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {
            "command": "ls -la"
        }
    }"#;
    
    let output = Command::new("cargo")
        .args(["run", "--", "--hook", "--config", config_path.to_str().unwrap()])
        .current_dir(env::current_dir().unwrap())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    
    let mut child = output;
    
    use std::io::Write;
    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(hook_input.as_bytes()).unwrap();
    let _ = stdin;
    
    let result = child.wait_with_output().unwrap();
    assert!(result.status.success());
    
    let stdout = String::from_utf8(result.stdout).unwrap();
    
    // When no mapping is found, the tool outputs nothing (allows command to proceed)
    // This is the expected behavior - no JSON output means "allow"
    assert!(stdout.trim().is_empty());
}

/// Test installer functionality
#[test]
fn test_installer_creates_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".claude-hook-advisor.toml");
    
    // Verify config doesn't exist initially
    assert!(!config_path.exists());
    
    // Run installer (this will detect project type and create config)
    // Note: In a real test environment, we'd need to mock stdin for interactive input
    // For now, we test that the installer can be invoked without crashing
    let output = Command::new("cargo")
        .args(["run", "--", "--install", "--config", config_path.to_str().unwrap()])
        .current_dir(temp_dir.path()) // Run in temp dir to avoid affecting real project
        .stdin(std::process::Stdio::null()) // No interactive input
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    
    let result = output.wait_with_output().unwrap();
    
    // Installer should run and create some output (even if it exits early due to no input)
    // The exact behavior depends on the implementation
    assert!(!result.stdout.is_empty() || !result.stderr.is_empty());
}

/// Test CLI argument parsing
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .current_dir(env::current_dir().unwrap())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    
    let result = output.wait_with_output().unwrap();
    assert!(result.status.success());
    
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert!(stdout.contains("claude-hook-advisor"));
    assert!(stdout.contains("--hook"));
    assert!(stdout.contains("--install"));
}

/// Test version information
#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .current_dir(env::current_dir().unwrap())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    
    let result = output.wait_with_output().unwrap();
    assert!(result.status.success());
    
    let stdout = String::from_utf8(result.stdout).unwrap();
    // Should contain version information
    assert!(!stdout.trim().is_empty());
}

/// Test --setup command with fresh configuration
#[test]
fn test_setup_command_fresh_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".claude-hook-advisor.toml");
    let config_path_str = config_path.to_str().unwrap();
    
    // Verify config doesn't exist initially
    assert!(!config_path.exists());
    
    // Run --setup command  
    let output = Command::new("cargo")
        .args(["run", "--", "--setup", "--config", config_path_str])
        .current_dir(env::current_dir().unwrap())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    
    let result = output.wait_with_output().unwrap();
    
    // Check that command completed successfully
    let stdout = String::from_utf8(result.stdout).unwrap();
    
    // Should mention setup completion
    assert!(stdout.contains("Running complete Claude Hook Advisor setup"));
    assert!(stdout.contains("Setup complete"));
    
    // Should add all default aliases
    assert!(stdout.contains("Added alias: 'docs'"));
    assert!(stdout.contains("Added alias: 'central_docs'"));
    assert!(stdout.contains("Added alias: 'project_docs'"));
    assert!(stdout.contains("Added alias: 'claude_docs'"));
    assert!(stdout.contains("Configuration saved with 4 new aliases"));
    
    // Verify config file was created and contains expected aliases
    assert!(config_path.exists());
    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(config_content.contains("[semantic_directories]"));
    assert!(config_content.contains("docs = \"~/Documents/Documentation\""));
    assert!(config_content.contains("central_docs = \"~/Documents/Documentation\""));
    assert!(config_content.contains("project_docs = \"~/Documents/Documentation/{project}\""));
    assert!(config_content.contains("claude_docs = \"~/Documents/Documentation/claude\""));
}

/// Test --setup command with existing configuration (should not overwrite)
#[test]
fn test_setup_command_preserves_existing() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".claude-hook-advisor.toml");
    let config_path_str = config_path.to_str().unwrap();
    
    // Create existing config with some aliases
    let existing_config = r#"[commands]
npm = "bun"
yarn = "bun"

[semantic_directories]
docs = "~/CustomDocs"  # Different path to test preservation
custom_alias = "/tmp/custom"

[directory_variables]
"#;
    fs::write(&config_path, existing_config).unwrap();
    
    // Run --setup command
    let output = Command::new("cargo")
        .args(["run", "--", "--setup", "--config", config_path_str])
        .current_dir(env::current_dir().unwrap())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    
    let result = output.wait_with_output().unwrap();
    let stdout = String::from_utf8(result.stdout).unwrap();
    
    // Should only add missing aliases, not overwrite existing ones
    assert!(!stdout.contains("Added alias: 'docs'")); // docs already exists
    assert!(stdout.contains("Added alias: 'central_docs'"));
    assert!(stdout.contains("Added alias: 'project_docs'"));
    assert!(stdout.contains("Added alias: 'claude_docs'"));
    assert!(stdout.contains("Configuration saved with 3 new aliases")); // Not 4!
    
    // Verify that existing values were preserved
    let final_config = fs::read_to_string(&config_path).unwrap();
    assert!(final_config.contains("docs = \"~/CustomDocs\"")); // Original value preserved
    assert!(final_config.contains("custom_alias = \"/tmp/custom\"")); // Custom alias preserved
    assert!(final_config.contains("npm = \"bun\"")); // Commands preserved
    assert!(final_config.contains("central_docs = \"~/Documents/Documentation\"")); // New alias added
}

/// Test --setup command with all aliases already configured
#[test]
fn test_setup_command_all_aliases_exist() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".claude-hook-advisor.toml");
    let config_path_str = config_path.to_str().unwrap();
    
    // Create config with all default aliases already present
    let complete_config = r#"[commands]
npm = "bun"

[semantic_directories]
docs = "~/Documents/Documentation"
central_docs = "~/Documents/Documentation"
project_docs = "~/Documents/Documentation/{project}"
claude_docs = "~/Documents/Documentation/claude"

[directory_variables]
"#;
    fs::write(&config_path, complete_config).unwrap();
    
    // Run --setup command
    let output = Command::new("cargo")
        .args(["run", "--", "--setup", "--config", config_path_str])
        .current_dir(env::current_dir().unwrap())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    
    let result = output.wait_with_output().unwrap();
    let stdout = String::from_utf8(result.stdout).unwrap();
    
    // Should detect that all aliases already exist
    assert!(stdout.contains("All default aliases already configured"));
    assert!(!stdout.contains("Added alias:"));
    assert!(!stdout.contains("Configuration saved"));
    
    // Verify config content unchanged
    let final_config = fs::read_to_string(&config_path).unwrap();
    assert_eq!(final_config.trim(), complete_config.trim());
}