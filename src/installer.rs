//! Installation and project setup logic

use crate::patterns::generate_config_for_project;
use crate::types::Config;
use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

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
pub fn run_installer(config_path: &str) -> Result<()> {
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
pub fn detect_project_type() -> Result<String> {
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

/// Prints detailed instructions for integrating with Claude Code.
/// 
/// Shows multiple integration options including the /hooks command and manual
/// .claude/settings.json configuration. Uses const strings and format! for
/// better maintainability.
/// 
/// # Returns
/// * `Ok(())` - Instructions printed successfully
/// * `Err` - If current executable path cannot be determined
pub fn print_claude_integration_instructions() -> Result<()> {
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
    fn test_project_type_detection() {
        // Test detection returns a valid project type
        let result = detect_project_type();
        assert!(result.is_ok());
        let project_type = result.unwrap();
        assert!(!project_type.is_empty());
        
        // Should be one of the known types (checking actual return values)
        let known_types = ["Node.js", "Python", "Rust", "Go", "General"];
        assert!(known_types.contains(&project_type.as_str()));
    }
}