//! Claude Hook Advisor
//! 
//! A Rust CLI tool that integrates with Claude Code as a PreToolUse hook to suggest
//! better command alternatives based on project-specific preferences.
//! 
//! The tool reads `.claude-hook-advisor.toml` configuration files and uses word-boundary
//! regex matching to intercept Bash commands, providing suggestions for preferred
//! alternatives (e.g., suggesting `bun` instead of `npm` for Node.js projects).
//! 
//! # Usage
//! 
//! - `--hook`: Run as Claude Code hook (reads JSON from stdin)
//! - `--install`: Interactive setup for current project
//! - `--config <path>`: Use custom configuration file path

use anyhow::{Context, Result};
use clap::{Arg, Command};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// Configuration structure for command mappings.
/// 
/// Loaded from .claude-hook-advisor.toml files, this struct contains
/// the mapping from original commands to their preferred replacements.
#[derive(Debug, Deserialize, Serialize)]
struct Config {
    commands: HashMap<String, String>,
}

/// Input data received from Claude Code hook system.
/// 
/// This struct represents the JSON data sent to PreToolUse hooks,
/// containing information about the tool being invoked and its parameters.
#[derive(Debug, Deserialize)]
struct HookInput {
    #[allow(dead_code)]
    session_id: String,
    #[allow(dead_code)]
    transcript_path: String,
    #[allow(dead_code)]
    cwd: String,
    #[allow(dead_code)]
    hook_event_name: String,
    tool_name: String,
    tool_input: ToolInput,
}

/// Tool-specific input parameters from Claude Code.
/// 
/// Contains the actual command and optional description for Bash tool invocations.
#[derive(Debug, Deserialize)]
struct ToolInput {
    command: String,
    #[allow(dead_code)]
    description: Option<String>,
}

/// Response data sent back to Claude Code hook system.
/// 
/// This struct represents the JSON response that tells Claude Code whether
/// to block the command and provides suggestions or replacements.
#[derive(Debug, Serialize)]
struct HookOutput {
    decision: String,
    reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    replacement_command: Option<String>,
}

/// Main entry point for the Claude Hook Advisor application.
/// 
/// Parses command-line arguments and dispatches to the appropriate mode:
/// - `--hook`: Run as a Claude Code PreToolUse hook (reads JSON from stdin)
/// - `--install`: Interactive installer to set up project configuration
/// - Default: Show usage information
fn main() -> Result<()> {
    let matches = Command::new("claude-hook-advisor")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Advises Claude Code on better command alternatives based on project preferences")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Path to configuration file")
                .default_value(".claude-hook-advisor.toml"),
        )
        .arg(
            Arg::new("hook")
                .long("hook")
                .help("Run as a Claude Code hook (reads JSON from stdin)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("replace")
                .long("replace")
                .help("Replace commands instead of blocking (experimental, not yet supported by Claude Code)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("install")
                .long("install")
                .help("Install and configure Claude Hook Advisor for this project")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();
    let replace_mode = matches.get_flag("replace");

    if matches.get_flag("hook") {
        run_as_hook(config_path, replace_mode)
    } else if matches.get_flag("install") {
        run_installer(config_path)
    } else {
        println!("Claude Hook Advisor");
        println!("Use --hook flag to run as a Claude Code hook");
        println!("Use --install flag to set up configuration for this project");
        Ok(())
    }
}

/// Runs the application as a Claude Code PreToolUse hook.
/// 
/// Reads JSON input from stdin containing hook event data, loads the project
/// configuration, and checks if the command should be blocked or replaced.
/// Only processes Bash commands; other tool types are ignored.
/// 
/// # Arguments
/// * `config_path` - Path to the .claude-hook-advisor.toml configuration file
/// * `replace_mode` - If true, returns "replace" decision; if false, returns "block"
/// 
/// # Returns
/// * `Ok(())` - Command allowed to proceed (no output)
/// * Process exits with JSON output if command should be blocked/replaced
fn run_as_hook(config_path: &str, replace_mode: bool) -> Result<()> {
    // Read configuration
    let config = load_config(config_path)?;

    // Read JSON input from stdin
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let hook_input: HookInput =
        serde_json::from_str(&buffer).context("Failed to parse hook input JSON")?;

    // Only process Bash commands
    if hook_input.tool_name != "Bash" {
        return Ok(());
    }

    let command = &hook_input.tool_input.command;

    // Check for command mappings
    if let Some((suggestion, replacement_cmd)) = check_command_mappings(&config, command)? {
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

    // No mappings found, allow command to proceed
    Ok(())
}

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
fn load_config(config_path: &str) -> Result<Config> {
    if !Path::new(config_path).exists() {
        // Log warning to stderr when config file is not found
        eprintln!("Warning: Config file '{config_path}' not found. No command mappings will be applied.");
        return Ok(Config {
            commands: HashMap::new(),
        });
    }

    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {config_path}"))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {config_path}"))?;

    Ok(config)
}

/// Checks if a command matches any configured mappings and generates suggestions.
/// 
/// Uses word-boundary regex matching to ensure exact command matches (e.g., "npm"
/// matches "npm install" but not "npm-check"). Returns the first matching pattern.
/// 
/// # Arguments
/// * `config` - Configuration containing command mappings
/// * `command` - The bash command to check against mappings
/// 
/// # Returns
/// * `Ok(Some((suggestion, replacement)))` - If a mapping is found
/// * `Ok(None)` - If no mappings match the command
/// * `Err` - If regex compilation fails
fn check_command_mappings(config: &Config, command: &str) -> Result<Option<(String, String)>> {
    for (pattern, replacement) in &config.commands {
        // Create regex pattern to match the command at word boundaries
        let regex_pattern = format!(r"\b{}\b", regex::escape(pattern));
        let regex = Regex::new(&regex_pattern)?;

        if regex.is_match(command) {
            // Generate suggested replacement
            let suggested_command = regex.replace_all(command, replacement);
            let suggestion = format!(
                "Command '{pattern}' is mapped to use '{replacement}' instead. Try: {suggested_command}"
            );
            return Ok(Some((suggestion, suggested_command.to_string())));
        }
    }

    Ok(None)
}

/// Interactive installer that sets up Claude Hook Advisor for a project.
/// 
/// Detects the project type, generates appropriate configuration, and provides
/// integration instructions for Claude Code. Prompts before overwriting existing configs.
/// 
/// # Arguments
/// * `config_path` - Path where the configuration file should be created
/// 
/// # Returns
/// * `Ok(())` - Installation completed successfully
/// * `Err` - If file operations fail or user cancels installation
fn run_installer(config_path: &str) -> Result<()> {
    println!("🚀 Claude Hook Advisor Installer");
    println!("==================================");

    // Check if config already exists
    if Path::new(config_path).exists() {
        println!("⚠️  Configuration file '{config_path}' already exists.");
        print!("Do you want to overwrite it? (y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("Installation cancelled.");
            return Ok(());
        }
    }

    // Detect project type and suggest appropriate config
    let project_type = detect_project_type()?;
    let config_content = generate_config_for_project(&project_type);

    // Write configuration file
    fs::write(config_path, &config_content)
        .with_context(|| format!("Failed to write config file: {config_path}"))?;

    println!("✅ Created configuration file: {config_path}");
    println!("📋 Configuration type: {project_type}");
    println!();

    // Show what was configured
    println!("📝 Command mappings configured:");
    let config: Config = toml::from_str(&config_content)?;
    for (from, to) in &config.commands {
        println!("   {from} → {to}");
    }
    println!();

    // Provide Claude Code integration instructions
    print_claude_integration_instructions()?;

    println!("🎉 Installation complete! Claude Hook Advisor is ready to use.");

    Ok(())
}

/// Detects the project type by examining files in the current directory.
/// 
/// Checks for common project indicators like package.json, Cargo.toml, etc.
/// Returns "General" as fallback if no specific project type is detected.
/// 
/// # Returns
/// * `Ok(String)` - Detected project type ("Node.js", "Python", "Rust", etc.)
/// * `Err` - If current directory cannot be accessed
fn detect_project_type() -> Result<String> {
    let current_dir = std::env::current_dir()?;

    // Check for various project indicators
    if current_dir.join("package.json").exists() {
        return Ok("Node.js".to_string());
    }

    if current_dir.join("requirements.txt").exists()
        || current_dir.join("pyproject.toml").exists()
        || current_dir.join("setup.py").exists()
    {
        return Ok("Python".to_string());
    }

    if current_dir.join("Cargo.toml").exists() {
        return Ok("Rust".to_string());
    }

    if current_dir.join("go.mod").exists() {
        return Ok("Go".to_string());
    }

    if current_dir.join("pom.xml").exists() || current_dir.join("build.gradle").exists() {
        return Ok("Java".to_string());
    }

    if current_dir.join("Dockerfile").exists() {
        return Ok("Docker".to_string());
    }

    Ok("General".to_string())
}

/// Generates a TOML configuration file for the specified project type.
/// 
/// Creates a properly formatted TOML file with project-specific command mappings
/// and common safety alternatives. Uses the toml crate for proper serialization.
/// 
/// # Arguments
/// * `project_type` - The type of project ("Node.js", "Python", "Rust", etc.)
/// 
/// # Returns
/// * `String` - Complete TOML configuration content with header and mappings
fn generate_config_for_project(project_type: &str) -> String {
    let commands = get_commands_for_project_type(project_type);
    let config = Config { commands };
    
    let header = format!(
        "# Claude Hook Advisor Configuration\n# Auto-generated for {project_type} project\n\n"
    );
    
    match toml::to_string_pretty(&config) {
        Ok(toml_content) => format!("{header}{toml_content}"),
        Err(_) => {
            // Fallback to basic config if serialization fails
            format!("{header}[commands]\n# Basic configuration\n")
        }
    }
}

/// Creates command mappings based on project type.
/// 
/// Returns a HashMap of command mappings tailored to the specific project type.
/// Includes both project-specific tools and common safety/modern alternatives.
/// 
/// # Arguments
/// * `project_type` - The type of project ("Node.js", "Python", "Rust", etc.)
/// 
/// # Returns
/// * `HashMap<String, String>` - Map from original commands to replacement commands
fn get_commands_for_project_type(project_type: &str) -> HashMap<String, String> {
    let mut commands = HashMap::new();
    
    match project_type {
        "Node.js" => {
            commands.insert("npm".to_string(), "bun".to_string());
            commands.insert("yarn".to_string(), "bun".to_string());
            commands.insert("pnpm".to_string(), "bun".to_string());
            commands.insert("npx".to_string(), "bunx".to_string());
            commands.insert("npm start".to_string(), "bun dev".to_string());
            commands.insert("npm test".to_string(), "bun test".to_string());
            commands.insert("npm run build".to_string(), "bun run build".to_string());
        }
        "Python" => {
            commands.insert("pip".to_string(), "uv pip".to_string());
            commands.insert("pip install".to_string(), "uv add".to_string());
            commands.insert("pip uninstall".to_string(), "uv remove".to_string());
            commands.insert("python".to_string(), "uv run python".to_string());
            commands.insert("python -m".to_string(), "uv run python -m".to_string());
        }
        "Rust" => {
            commands.insert("cargo check".to_string(), "cargo clippy".to_string());
            commands.insert("cargo test".to_string(), "cargo test -- --nocapture".to_string());
        }
        "Go" => {
            commands.insert("go run".to_string(), "go run -race".to_string());
            commands.insert("go test".to_string(), "go test -v".to_string());
        }
        "Java" => {
            commands.insert("mvn".to_string(), "./mvnw".to_string());
            commands.insert("gradle".to_string(), "./gradlew".to_string());
        }
        "Docker" => {
            commands.insert("docker".to_string(), "podman".to_string());
            commands.insert("docker-compose".to_string(), "podman-compose".to_string());
        }
        _ => {
            commands.insert("cat".to_string(), "bat".to_string());
            commands.insert("ls".to_string(), "eza".to_string());
            commands.insert("grep".to_string(), "rg".to_string());
            commands.insert("find".to_string(), "fd".to_string());
        }
    }
    
    // Add common safety and modern tool mappings for all project types
    commands.insert("curl".to_string(), "curl -L".to_string());
    commands.insert("rm".to_string(), "trash".to_string());
    commands.insert("rm -rf".to_string(), "echo 'Use trash command for safety'".to_string());
    
    commands
}

/// Prints detailed instructions for integrating with Claude Code.
/// 
/// Shows multiple integration options including the /hooks command and manual
/// .claude/settings.json configuration. Uses const strings and format! for
/// better maintainability.
/// 
/// # Returns
/// * `Ok(())` - Instructions printed successfully
/// * `Err` - If current executable path cannot be determined
fn print_claude_integration_instructions() -> Result<()> {
    let binary_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "claude-hook-advisor".to_string());

    const HEADER: &str = r#"🔧 Claude Code Integration Setup:
==================================

To integrate with Claude Code, you have several options:

Option 1: Using the /hooks command in Claude Code
  1. Run `/hooks` in Claude Code
  2. Select `PreToolUse`
  3. Add matcher: `Bash`"#;

    const JSON_TEMPLATE: &str = r#"{{
  "hooks": {{
    "PreToolUse": [
      {{
        "matcher": "Bash",
        "hooks": [
          {{
            "type": "command",
            "command": "{} --hook"
          }}
        ]
      }}
    ]
  }}
}}"#;

    print!(
        r#"{HEADER}
  4. Add hook command: `{binary_path} --hook`
  5. Save to project settings

Option 2: Manual .claude/settings.json configuration
Add this to your .claude/settings.json:

{json_config}

"#,
        binary_path = binary_path,
        json_config = JSON_TEMPLATE.replace("{}", &binary_path)
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_mapping() {
        let mut commands = HashMap::new();
        commands.insert("npm".to_string(), "bun".to_string());
        commands.insert("yarn".to_string(), "bun".to_string());
        commands.insert("npx".to_string(), "bunx".to_string());

        let config = Config { commands };

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

        // Test no mapping
        let result = check_command_mappings(&config, "ls -la").unwrap();
        assert!(result.is_none());
    }
}
