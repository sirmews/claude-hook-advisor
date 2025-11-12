//! Hook processing logic

use crate::config::load_config;
use crate::directory::detect_directory_references;
use crate::history;
use crate::security::get_default_security_patterns;
use crate::types::{Config, HookInput, HookOutput, SecurityPattern};
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::Mutex;

/// Cache for compiled regex patterns to avoid recompilation
static REGEX_CACHE: Lazy<Mutex<HashMap<String, Regex>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Runs the application as a Claude Code hook for multiple event types.
/// 
/// Reads JSON input from stdin containing hook event data, loads the project
/// configuration, and processes based on the hook event type:
/// - PreToolUse: Command mapping and replacement suggestions
/// - UserPromptSubmit: Directory reference detection and learning
/// - PostToolUse: Command execution tracking and analysis
/// 
/// # Arguments
/// * `config_path` - Path to the .claude-hook-advisor.toml configuration file
/// * `replace_mode` - If true, returns "replace" decision; if false, returns "block"
/// 
/// # Returns
/// * `Ok(())` - Hook processing completed (may output to stdout)
/// * `Err` - If JSON parsing or configuration loading fails
pub fn run_as_hook(config_path: &str, replace_mode: bool) -> Result<()> {
    // Read configuration
    let config = load_config(config_path)?;

    // Read JSON input from stdin
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let hook_input: HookInput =
        serde_json::from_str(&buffer).context("Failed to parse hook input JSON")?;

    // Route to appropriate handler based on hook event type
    match hook_input.hook_event_name.as_str() {
        "PreToolUse" => handle_pre_tool_use(&config, &hook_input, replace_mode)?,
        "UserPromptSubmit" => handle_user_prompt_submit(&config, &hook_input)?,
        "PostToolUse" => handle_post_tool_use(&config, &hook_input)?,
        _ => {
            // Unknown hook event type, log warning and continue
            eprintln!("Warning: Unknown hook event type: {}", hook_input.hook_event_name);
        }
    }

    Ok(())
}

/// Handles PreToolUse hook events for command mapping and security checking.
///
/// Processes Bash commands for command mappings, and Edit/Write/MultiEdit tools
/// for security pattern detection. Logs Bash commands as "pending", and checks
/// for configured mappings or security issues.
///
/// # Arguments
/// * `config` - Configuration containing command mappings and security patterns
/// * `hook_input` - Hook input data from Claude Code
/// * `replace_mode` - Whether to replace or block commands
///
/// # Returns
/// * `Ok(())` - Processing completed (may exit process with JSON output)
/// * `Err` - If command mapping or security check fails
fn handle_pre_tool_use(config: &Config, hook_input: &HookInput, replace_mode: bool) -> Result<()> {
    let tool_name = hook_input.tool_name.as_deref();

    // Handle Bash commands
    if tool_name == Some("Bash") {
        return handle_bash_tool(config, hook_input, replace_mode);
    }

    // Handle file editing tools for security patterns
    if matches!(tool_name, Some("Edit") | Some("Write") | Some("MultiEdit")) {
        return handle_file_tool(config, hook_input);
    }

    Ok(())
}

/// Handles Bash tool for command mapping and replacement
fn handle_bash_tool(config: &Config, hook_input: &HookInput, replace_mode: bool) -> Result<()> {

    let Some(tool_input) = &hook_input.tool_input else {
        return Ok(());
    };

    let Some(command) = &tool_input.command else {
        return Ok(());
    };

    // Log command as pending if history tracking is enabled
    if let Some(history_config) = &config.command_history {
        if history_config.enabled {
            let log_path = expand_tilde(&history_config.log_file)?;

            // Initialize database connection
            if let Ok(conn) = history::init_database(&log_path) {
                // Create pending record
                let record = history::create_record(
                    &hook_input.session_id,
                    command,
                    None, // No exit code yet
                    hook_input.cwd.as_deref(),
                    false, // Not yet replaced
                    None,  // No original command yet
                    "pending",
                );

                // Log the pending command (ignore errors to not block execution)
                let _ = history::log_command(&conn, &record);
            }
        }
    }

    // Check for command mappings
    if let Some((suggestion, replacement_cmd)) = check_command_mappings(config, command)? {
        let output = if replace_mode {
            HookOutput {
                decision: "replace".to_string(),
                reason: format!("Command mapped: using '{replacement_cmd}' instead"),
                replacement_command: Some(replacement_cmd),
            }
        } else {
            HookOutput {
                decision: "block".to_string(),
                reason: suggestion,
                replacement_command: None,
            }
        };

        println!("{}", serde_json::to_string(&output)?);
        std::process::exit(0);
    }

    Ok(())
}

/// Gets the list of enabled security patterns by merging defaults with overrides.
///
/// Default patterns are enabled unless explicitly disabled in the config.
fn get_enabled_security_patterns(config: &Config) -> Vec<SecurityPattern> {
    let defaults = get_default_security_patterns();

    defaults
        .into_iter()
        .filter(|pattern| {
            // Check if pattern is explicitly disabled
            !matches!(
                config.security_pattern_overrides.get(&pattern.rule_name),
                Some(false)
            )
        })
        .collect()
}

/// Handles file editing tools (Edit/Write/MultiEdit) for security pattern detection.
///
/// Checks file paths and content against configured security patterns to warn
/// about potential security vulnerabilities before files are modified.
///
/// # Arguments
/// * `config` - Configuration containing security pattern overrides
/// * `hook_input` - Hook input data containing file editing parameters
///
/// # Returns
/// * `Ok(())` - Processing completed (may exit process with blocking decision)
/// * `Err` - If security pattern check fails
fn handle_file_tool(config: &Config, hook_input: &HookInput) -> Result<()> {
    // Get enabled security patterns (defaults with overrides applied)
    let security_patterns = get_enabled_security_patterns(config);

    let Some(tool_input) = &hook_input.tool_input else {
        return Ok(());
    };

    let Some(file_path) = &tool_input.file_path else {
        return Ok(());
    };

    // Extract content to check based on tool type
    let content = extract_content_from_tool_input(hook_input.tool_name.as_deref(), tool_input);

    // Check for security pattern matches
    if let Some((rule_name, reminder)) = check_security_patterns(&security_patterns, file_path, &content)? {
        // Check if we've already shown this warning in this session
        if should_show_warning(&hook_input.session_id, file_path, &rule_name)? {
            // Mark warning as shown
            mark_warning_shown(&hook_input.session_id, file_path, &rule_name)?;

            // Output blocking decision with security reminder
            let output = HookOutput {
                decision: "block".to_string(),
                reason: reminder,
                replacement_command: None,
            };

            println!("{}", serde_json::to_string(&output)?);
            std::process::exit(0);
        }
    }

    Ok(())
}

/// Extracts content to check from tool input based on tool type
fn extract_content_from_tool_input(tool_name: Option<&str>, tool_input: &crate::types::ToolInput) -> String {
    match tool_name {
        Some("Write") => tool_input.content.clone().unwrap_or_default(),
        Some("Edit") => tool_input.new_string.clone().unwrap_or_default(),
        Some("MultiEdit") => {
            if let Some(edits) = &tool_input.edits {
                edits
                    .iter()
                    .map(|edit| edit.new_string.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

/// Handles UserPromptSubmit hook events for directory reference detection.
/// 
/// Analyzes user prompts for semantic directory references and outputs
/// resolved canonical paths to help Claude Code understand directory context.
/// 
/// # Arguments
/// * `config` - Configuration containing directory mappings
/// * `hook_input` - Hook input data containing user prompt
/// 
/// # Returns
/// * `Ok(())` - Processing completed (may output directory resolutions)
/// * `Err` - If directory resolution fails
fn handle_user_prompt_submit(config: &Config, hook_input: &HookInput) -> Result<()> {
    let Some(prompt) = &hook_input.prompt else {
        return Ok(());
    };

    // Detect directory references in the prompt
    let directory_refs = detect_directory_references(config, prompt);
    
    if !directory_refs.is_empty() {
        // Output directory resolutions as plain text (not JSON for UserPromptSubmit)
        for resolution in directory_refs {
            println!("Directory reference '{}' resolved to: {}", 
                resolution.alias_used, 
                resolution.canonical_path
            );
            
            if !resolution.variables_substituted.is_empty() {
                println!("  Variables substituted: {:?}", resolution.variables_substituted);
            }
        }
    }

    Ok(())
}

/// Handles PostToolUse hook events for command execution tracking.
///
/// Updates the status of pending commands to "success" when they complete.
/// This only fires for successful commands, so any pending commands that
/// remain after execution are implicitly failed.
///
/// # Arguments
/// * `config` - Configuration for tracking settings
/// * `hook_input` - Hook input data containing execution results
///
/// # Returns
/// * `Ok(())` - Processing completed (may output analytics)
/// * `Err` - If execution tracking fails
fn handle_post_tool_use(config: &Config, hook_input: &HookInput) -> Result<()> {
    let Some(tool_name) = &hook_input.tool_name else {
        return Ok(());
    };

    let Some(tool_response) = &hook_input.tool_response else {
        return Ok(());
    };

    // Only track Bash command executions
    if tool_name != "Bash" {
        return Ok(());
    }

    // Check if command history is enabled
    let history_config = match &config.command_history {
        Some(cfg) if cfg.enabled => cfg,
        _ => return Ok(()), // History disabled, skip logging
    };

    // Get command details
    let Some(tool_input) = &hook_input.tool_input else {
        return Ok(());
    };

    let Some(command) = &tool_input.command else {
        return Ok(());
    };

    // Expand tilde in log file path
    let log_path = expand_tilde(&history_config.log_file)?;

    // Initialize database connection
    let conn = history::init_database(&log_path)
        .context("Failed to initialize command history database")?;

    // Update the pending command to success
    let updated = history::update_command_status(
        &conn,
        &hook_input.session_id,
        command,
        "success",
        tool_response.exit_code,
    )
    .context("Failed to update command status")?;

    // If no pending command was found to update, this might be a command
    // that wasn't logged in PreToolUse (e.g., if hooks were just enabled)
    // In that case, create a new record directly as success
    if !updated {
        let record = history::create_record(
            &hook_input.session_id,
            command,
            tool_response.exit_code,
            hook_input.cwd.as_deref(),
            false,
            None,
            "success",
        );

        history::log_command(&conn, &record)
            .context("Failed to log command to history")?;
    }

    Ok(())
}

/// Expands tilde (~) in file paths to the user's home directory
fn expand_tilde(path: &str) -> Result<PathBuf> {
    if path.starts_with("~/") {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
        Ok(PathBuf::from(path.replacen("~", &home, 1)))
    } else {
        Ok(PathBuf::from(path))
    }
}

/// Gets or creates a cached regex for the given pattern
fn get_cached_regex(pattern: &str) -> Result<Regex> {
    let mut cache = REGEX_CACHE.lock()
        .expect("regex cache mutex should not be poisoned");
    
    if let Some(regex) = cache.get(pattern) {
        return Ok(regex.clone());
    }
    
    let regex = Regex::new(pattern)?;
    cache.insert(pattern.to_string(), regex.clone());
    Ok(regex)
}

/// Checks if a file path or content matches any security patterns.
///
/// Security patterns can match based on:
/// 1. File path glob patterns (e.g., ".github/workflows/*.yml")
/// 2. Content substring matching (e.g., "eval(", "dangerouslySetInnerHTML")
///
/// Returns the first matching pattern.
///
/// # Arguments
/// * `patterns` - List of security patterns to check
/// * `file_path` - The file path being edited
/// * `content` - The content being written/edited
///
/// # Returns
/// * `Ok(Some((rule_name, reminder)))` - If a pattern matches
/// * `Ok(None)` - If no patterns match
/// * `Err` - If pattern matching fails
fn check_security_patterns(patterns: &[SecurityPattern], file_path: &str, content: &str) -> Result<Option<(String, String)>> {
    // Normalize file path by removing leading slashes
    let normalized_path = file_path.trim_start_matches('/');

    for pattern in patterns {
        // Check path-based patterns using glob matching
        if let Some(path_pattern) = &pattern.path_pattern {
            if glob_match(path_pattern, normalized_path)? {
                return Ok(Some((pattern.rule_name.clone(), pattern.reminder.clone())));
            }
        }

        // Check content-based patterns
        if !pattern.content_substrings.is_empty() && !content.is_empty() {
            for substring in &pattern.content_substrings {
                if content.contains(substring) {
                    return Ok(Some((pattern.rule_name.clone(), pattern.reminder.clone())));
                }
            }
        }
    }

    Ok(None)
}

/// Checks if a file path matches a glob pattern
fn glob_match(pattern: &str, path: &str) -> Result<bool> {
    // Simple glob matching supporting * and **
    let regex_pattern = pattern
        .replace(".", r"\.")
        .replace("**", "DOUBLE_STAR")
        .replace("*", "[^/]*")
        .replace("DOUBLE_STAR", ".*");

    let regex = get_cached_regex(&format!("^{}$", regex_pattern))?;
    Ok(regex.is_match(path))
}

/// Checks if we should show a warning for the given session, file, and rule.
///
/// Returns true if the warning hasn't been shown yet in this session.
fn should_show_warning(session_id: &str, file_path: &str, rule_name: &str) -> Result<bool> {
    let state_file = get_security_state_file(session_id)?;

    if !state_file.exists() {
        return Ok(true);
    }

    let content = std::fs::read_to_string(&state_file)?;
    let shown_warnings: std::collections::HashSet<String> =
        serde_json::from_str(&content).unwrap_or_default();

    let warning_key = format!("{}-{}", file_path, rule_name);
    Ok(!shown_warnings.contains(&warning_key))
}

/// Marks a warning as shown for the given session, file, and rule.
fn mark_warning_shown(session_id: &str, file_path: &str, rule_name: &str) -> Result<()> {
    let state_file = get_security_state_file(session_id)?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = state_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Load existing warnings
    let mut shown_warnings: std::collections::HashSet<String> = if state_file.exists() {
        let content = std::fs::read_to_string(&state_file)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        std::collections::HashSet::new()
    };

    // Add new warning
    let warning_key = format!("{}-{}", file_path, rule_name);
    shown_warnings.insert(warning_key);

    // Save updated warnings
    let content = serde_json::to_string(&shown_warnings)?;
    std::fs::write(&state_file, content)?;

    Ok(())
}

/// Gets the path to the security state file for a given session
fn get_security_state_file(session_id: &str) -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .context("HOME environment variable not set")?;

    Ok(PathBuf::from(format!(
        "{}/.claude/security_warnings_state_{}.json",
        home, session_id
    )))
}

/// Checks if a command matches any configured mappings and generates suggestions.
///
/// Only matches the primary command at the start of the line (e.g., "npm" matches
/// "npm install" but NOT "my-npm-tool" or "npx npm"). This ensures command mappings
/// only apply to the main command being executed, not subcommands or arguments.
/// Returns the first matching pattern. Uses cached regex compilation for better performance.
///
/// # Arguments
/// * `config` - Configuration containing command mappings
/// * `command` - The bash command to check against mappings
///
/// # Returns
/// * `Ok(Some((suggestion, replacement)))` - If a mapping is found
/// * `Ok(None)` - If no mappings match the command
/// * `Err` - If regex compilation fails
pub fn check_command_mappings(config: &Config, command: &str) -> Result<Option<(String, String)>> {
    for (pattern, replacement) in &config.commands {
        // Create regex pattern that only matches at start of line
        // ^ = start of string (primary command position)
        // Group 1: the pattern to match
        // Group 2: (\s|$) = followed by whitespace or end of string
        // This ensures only the primary command is matched, not subcommands
        let regex_pattern = format!(r"^({})(\s|$)", regex::escape(pattern));
        let regex = get_cached_regex(&regex_pattern)?;

        if regex.is_match(command) {
            // Generate suggested replacement, preserving trailing whitespace
            let suggested_command = regex.replace_all(command, |caps: &regex::Captures| {
                format!("{}{}", replacement, &caps[2])
            });
            let suggestion = format!(
                "Command '{pattern}' is mapped to use '{replacement}' instead. Try: {suggested_command}"
            );
            return Ok(Some((suggestion, suggested_command.to_string())));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_command_mapping() {
        let mut commands = HashMap::new();
        commands.insert("npm".to_string(), "bun".to_string());
        commands.insert("yarn".to_string(), "bun".to_string());
        commands.insert("npx".to_string(), "bunx".to_string());

        let config = Config {
            commands,
            semantic_directories: HashMap::new(),
            command_history: None,
            security_pattern_overrides: HashMap::new(),
        };

        // Test npm mapping
        let result = check_command_mappings(&config, "npm install").unwrap();
        assert!(result.is_some());
        let (suggestion, replacement) = result.unwrap();
        assert!(suggestion.contains("bun install"));
        assert_eq!(replacement, "bun install");

        // Test yarn mapping
        let result = check_command_mappings(&config, "yarn start").unwrap();
        assert!(result.is_some());
        let (suggestion, replacement) = result.unwrap();
        assert!(suggestion.contains("bun start"));
        assert_eq!(replacement, "bun start");
    }

    #[test]
    fn test_command_mapping_edge_cases() {
        let mut commands = HashMap::new();
        commands.insert("npm".to_string(), "bun".to_string());
        let config = Config {
            commands,
            semantic_directories: HashMap::new(),
            command_history: None,
            security_pattern_overrides: HashMap::new(),
        };

        // Test whitespace boundaries - "npm" in "my-npm-tool" should NOT match
        // because it's not a standalone token (no whitespace separation)
        let result = check_command_mappings(&config, "my-npm-tool install").unwrap();
        assert!(result.is_none(), "npm in 'my-npm-tool' should NOT match");

        // Test empty command
        let result = check_command_mappings(&config, "").unwrap();
        assert!(result.is_none());

        // Test command with multiple spaces - should preserve spacing
        let result = check_command_mappings(&config, "npm   install   --verbose").unwrap();
        assert!(result.is_some());
        let (_, replacement) = result.unwrap();
        assert_eq!(replacement, "bun   install   --verbose");

        // Test npm NOT at start should NOT match (only matches primary command)
        let result = check_command_mappings(&config, "run npm").unwrap();
        assert!(result.is_none(), "'npm' in 'run npm' should NOT match (not primary command)");

        // Test npm-like substring should NOT match
        let result = check_command_mappings(&config, "npmc install").unwrap();
        assert!(result.is_none(), "'npmc' should NOT match 'npm'");

        // Test command by itself (no args)
        let result = check_command_mappings(&config, "npm").unwrap();
        assert!(result.is_some());
        let (_, replacement) = result.unwrap();
        assert_eq!(replacement, "bun");
    }

    #[test]
    fn test_command_mapping_prevents_false_positives() {
        let mut commands = HashMap::new();
        commands.insert("RM".to_string(), "rm -i".to_string());
        let config = Config {
            commands,
            semantic_directories: HashMap::new(),
            command_history: None,
            security_pattern_overrides: HashMap::new(),
        };

        // Test exact match
        let result = check_command_mappings(&config, "RM file.txt").unwrap();
        assert!(result.is_some());
        let (_, replacement) = result.unwrap();
        assert_eq!(replacement, "rm -i file.txt");

        // Test should NOT match when RM is part of a larger word
        let result = check_command_mappings(&config, "RMm file.txt").unwrap();
        assert!(result.is_none(), "'RMm' should NOT match 'RM'");

        // Test should NOT match when RM has prefix
        let result = check_command_mappings(&config, "gitRM file.txt").unwrap();
        assert!(result.is_none(), "'gitRM' should NOT match 'RM'");

        // Test should NOT match when RM is a subcommand (not at start)
        let result = check_command_mappings(&config, "git RM file.txt").unwrap();
        assert!(result.is_none(), "'RM' in 'git RM' should NOT match (not primary command)");

        // Test should NOT match when RM has hyphen prefix
        let result = check_command_mappings(&config, "git-RM file.txt").unwrap();
        assert!(result.is_none(), "'git-RM' should NOT match 'RM'");

        // Test should NOT match when RM has hyphen suffix
        let result = check_command_mappings(&config, "RM-tool file.txt").unwrap();
        assert!(result.is_none(), "'RM-tool' should NOT match 'RM'");
    }

    #[test]
    fn test_hook_output_serialization() {
        // Test blocking output
        let output = HookOutput {
            decision: "block".to_string(),
            reason: "Test reason".to_string(),
            replacement_command: Some("test command".to_string()),
        };
        
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"decision\":\"block\""));
        assert!(json.contains("\"reason\":\"Test reason\""));
        assert!(json.contains("\"replacement_command\":\"test command\""));

        // Test allowing output (no replacement)
        let output = HookOutput {
            decision: "allow".to_string(),
            reason: "No mapping found".to_string(),
            replacement_command: None,
        };
        
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"decision\":\"allow\""));
        assert!(json.contains("\"reason\":\"No mapping found\""));
        // Should not include replacement_command field when None due to serde skip
        assert!(!json.contains("replacement_command"));
    }
}