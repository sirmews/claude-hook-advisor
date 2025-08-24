//! CLI interface and main entry point

use crate::hooks::run_as_hook;
use crate::Config;
use anyhow::{Context, Result};
use clap::{Arg, Command};
use std::fs;
use std::path::Path;

/// Main entry point for the Claude Hook Advisor application.
/// 
/// Parses command-line arguments and dispatches to the appropriate mode:
/// - `--hook`: Run as a Claude Code PreToolUse hook (reads JSON from stdin)
/// - `--install`: Interactive installer to set up project configuration
/// - Default: Show usage information
pub fn run_cli() -> Result<()> {
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
                .help("Install Claude Hook Advisor: configure hooks and create/update config file")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("uninstall")
                .long("uninstall")
                .help("Remove Claude Hook Advisor hooks from Claude Code settings")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("install-hooks")
                .long("install-hooks")
                .help("Install hooks into Claude Code settings")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("uninstall-hooks")
                .long("uninstall-hooks")
                .help("Remove hooks from Claude Code settings")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("interactive-install")
                .long("interactive-install")
                .help("Run interactive installer with project type detection")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config")
        .expect("config argument has default value");
    let replace_mode = matches.get_flag("replace");

    if matches.get_flag("hook") {
        run_as_hook(config_path, replace_mode)
    } else if matches.get_flag("install") {
        run_smart_installation(config_path)
    } else if matches.get_flag("uninstall") {
        crate::installer::uninstall_claude_hooks()
    } else {
        println!("Claude Hook Advisor v{}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("Installation:");
        println!("  --install                 Install Claude Hook Advisor: configure hooks and create/update config file");
        println!();
        println!("Command Mapping:");
        println!("  --hook                    Run as a Claude Code hook");
        println!();
        println!("Configuration:");
        println!("  -c, --config <FILE>       Path to config file [default: .claude-hook-advisor.toml]");
        println!();
        println!("To configure directory aliases and command mappings, edit .claude-hook-advisor.toml directly.");
        Ok(())
    }
}


/// Smart installation that checks existing state and only makes necessary changes.
/// 
/// This function:
/// 1. Checks if hooks already exist - if so, skips hook installation
/// 2. Checks if config file exists - if not, creates it with examples
/// 3. If config exists, ensures required sections exist with commented examples
/// 
/// # Arguments
/// * `config_path` - Path to the configuration file
/// 
/// # Returns
/// * `Ok(())` - Installation completed successfully
/// * `Err` - If any installation step fails
fn run_smart_installation(config_path: &str) -> Result<()> {
    println!("🚀 Claude Hook Advisor Installation");
    println!("===================================\n");
    
    // Step 1: Check and install hooks if needed
    if hooks_already_exist()? {
        println!("✅ Hooks already installed in Claude Code settings");
    } else {
        println!("📋 Installing hooks into Claude Code settings...");
        crate::installer::install_claude_hooks()?;
        println!("✅ Hooks installed successfully");
    }
    
    // Step 2: Handle config file
    println!("\n📄 Checking configuration file...");
    if Path::new(config_path).exists() {
        println!("✅ Config file exists: {config_path}");
        ensure_config_sections(config_path)?;
    } else {
        println!("📝 Creating new config file: {config_path}");
        create_example_config(config_path)?;
    }
    
    println!("\n🎉 Installation complete! Claude Hook Advisor is ready to use.");
    println!("💡 You can now use semantic directory references in Claude Code conversations.");
    
    Ok(())
}

/// Checks if Claude Hook Advisor hooks are already installed in Claude Code settings.
/// 
/// # Returns
/// * `Ok(true)` - Hooks are already installed
/// * `Ok(false)` - Hooks are not installed
/// * `Err` - If settings file cannot be read or parsed
fn hooks_already_exist() -> Result<bool> {
    // Check for settings files in order of preference
    let local_settings = Path::new(".claude/settings.local.json");
    let shared_settings = Path::new(".claude/settings.json");
    
    let settings_path = if local_settings.exists() {
        local_settings
    } else if shared_settings.exists() {
        shared_settings
    } else {
        return Ok(false); // No settings file means no hooks
    };
    
    // Read and parse settings file
    let settings_content = fs::read_to_string(settings_path)
        .with_context(|| format!("Failed to read {}", settings_path.display()))?;
    
    let settings: serde_json::Value = serde_json::from_str(&settings_content)
        .with_context(|| "Failed to parse Claude settings JSON")?;
    
    // Check if our hooks exist
    if let Some(hooks) = settings.get("hooks").and_then(|h| h.as_object()) {
        // Check PreToolUse and UserPromptSubmit hooks
        for event_name in &["PreToolUse", "UserPromptSubmit"] {
            if let Some(event_hooks) = hooks.get(*event_name).and_then(|h| h.as_array()) {
                for hook_group in event_hooks {
                    if let Some(hooks_array) = hook_group.get("hooks").and_then(|h| h.as_array()) {
                        for hook in hooks_array {
                            if let Some(command) = hook.get("command").and_then(|c| c.as_str()) {
                                if command.contains("claude-hook-advisor") {
                                    return Ok(true);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(false)
}

/// Creates a new configuration file with examples and comments.
/// 
/// # Arguments
/// * `config_path` - Path where to create the configuration file
/// 
/// # Returns
/// * `Ok(())` - Configuration created successfully
/// * `Err` - If file writing fails
fn create_example_config(config_path: &str) -> Result<()> {
    let example_config = r#"# Claude Hook Advisor Configuration
# This file configures command mappings and semantic directory aliases
# for use with Claude Code integration.

# Command mappings - suggest alternatives when Claude Code runs these commands
[commands]
# npm = "bun"          # Suggest 'bun' instead of 'npm'
# yarn = "bun"         # Suggest 'bun' instead of 'yarn'
# npx = "bunx"         # Suggest 'bunx' instead of 'npx'
# grep = "rg"          # Suggest 'rg' (ripgrep) instead of 'grep'

# Semantic directory aliases - natural language directory references
[semantic_directories]
docs = "~/Documents/Documentation"
central_docs = "~/Documents/Documentation"
project_docs = "~/Documents/Documentation/{project}"
claude_docs = "~/Documents/Documentation/claude"

# Directory variables for path substitution
[directory_variables]
project = "claude-hook-advisor"    # Auto-detected from git repository name
user_home = "~"
"#;

    fs::write(config_path, example_config)
        .with_context(|| format!("Failed to write config file: {config_path}"))?;
    
    println!("✅ Created example configuration with default directory aliases");
    Ok(())
}

/// Ensures required sections exist in an existing config file.
/// 
/// # Arguments
/// * `config_path` - Path to the configuration file
/// 
/// # Returns
/// * `Ok(())` - Configuration updated successfully
/// * `Err` - If file operations fail
fn ensure_config_sections(config_path: &str) -> Result<()> {
    let mut config_content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {config_path}"))?;
    
    let mut needs_update = false;
    
    // Check and add missing sections
    if !config_content.contains("[commands]") {
        config_content.push_str("\n# Command mappings - suggest alternatives when Claude Code runs these commands\n");
        config_content.push_str("[commands]\n");
        config_content.push_str("# npm = \"bun\"          # Suggest 'bun' instead of 'npm'\n");
        config_content.push_str("# yarn = \"bun\"         # Suggest 'bun' instead of 'yarn'\n");
        config_content.push_str("# npx = \"bunx\"         # Suggest 'bunx' instead of 'npx'\n");
        config_content.push_str("# grep = \"rg\"          # Suggest 'rg' (ripgrep) instead of 'grep'\n\n");
        needs_update = true;
        println!("✅ Added [commands] section with examples");
    }
    
    if !config_content.contains("[semantic_directories]") {
        config_content.push_str("# Semantic directory aliases - natural language directory references\n");
        config_content.push_str("[semantic_directories]\n");
        config_content.push_str("docs = \"~/Documents/Documentation\"\n");
        config_content.push_str("central_docs = \"~/Documents/Documentation\"\n");
        config_content.push_str("project_docs = \"~/Documents/Documentation/{project}\"\n");
        config_content.push_str("claude_docs = \"~/Documents/Documentation/claude\"\n\n");
        needs_update = true;
        println!("✅ Added [semantic_directories] section with default aliases");
    }
    
    if !config_content.contains("[directory_variables]") {
        config_content.push_str("# Directory variables for path substitution\n");
        config_content.push_str("[directory_variables]\n");
        config_content.push_str("project = \"claude-hook-advisor\"    # Auto-detected from git repository name\n");
        config_content.push_str("user_home = \"~\"\n");
        needs_update = true;
        println!("✅ Added [directory_variables] section");
    }
    
    if needs_update {
        fs::write(config_path, config_content)
            .with_context(|| format!("Failed to update config file: {config_path}"))?;
        println!("💾 Configuration file updated");
    } else {
        println!("✅ All required sections already present");
    }
    
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use serde_json::json;
    
    // Helper function to run a test in a temporary directory
    fn with_temp_dir<F>(test: F) 
    where 
        F: FnOnce(),
    {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        
        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        // Run test with proper cleanup
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            test();
        }));
        
        // Always restore original directory
        std::env::set_current_dir(&original_dir).unwrap();
        
        // Re-panic if test panicked
        if let Err(err) = result {
            std::panic::resume_unwind(err);
        }
    }
    
    #[test]
    fn test_hooks_already_exist_no_settings_file() {
        with_temp_dir(|| {
            let result = hooks_already_exist().unwrap();
            assert!(!result, "Should return false when no settings files exist");
        });
    }
    
    #[test]
    fn test_hooks_already_exist_empty_settings() {
        with_temp_dir(|| {
            // Create .claude directory and empty settings file
            fs::create_dir_all(".claude").unwrap();
            let settings_content = json!({});
            fs::write(".claude/settings.local.json", serde_json::to_string_pretty(&settings_content).unwrap()).unwrap();
            
            let result = hooks_already_exist().unwrap();
            assert!(!result, "Should return false when settings file has no hooks");
        });
    }
    
    #[test]
    fn test_hooks_already_exist_with_our_hooks() {
        with_temp_dir(|| {
            // Create .claude directory and settings file with our hooks
            fs::create_dir_all(".claude").unwrap();
            let settings_content = json!({
                "hooks": {
                    "PreToolUse": [
                        {
                            "matcher": "Bash",
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": "claude-hook-advisor --hook"
                                }
                            ]
                        }
                    ]
                }
            });
            fs::write(".claude/settings.local.json", serde_json::to_string_pretty(&settings_content).unwrap()).unwrap();
            
            let result = hooks_already_exist().unwrap();
            assert!(result, "Should return true when our hooks are present");
        });
    }
    
    #[test]
    fn test_hooks_already_exist_with_other_hooks() {
        with_temp_dir(|| {
            // Create .claude directory and settings file with other hooks
            fs::create_dir_all(".claude").unwrap();
            let settings_content = json!({
                "hooks": {
                    "PreToolUse": [
                        {
                            "matcher": "Bash",
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": "some-other-tool --hook"
                                }
                            ]
                        }
                    ]
                }
            });
            fs::write(".claude/settings.local.json", serde_json::to_string_pretty(&settings_content).unwrap()).unwrap();
            
            let result = hooks_already_exist().unwrap();
            assert!(!result, "Should return false when only other hooks are present");
        });
    }
    
    #[test]
    fn test_hooks_already_exist_userprompsubmit_hooks() {
        with_temp_dir(|| {
            // Create .claude directory and settings file with UserPromptSubmit hooks
            fs::create_dir_all(".claude").unwrap();
            let settings_content = json!({
                "hooks": {
                    "UserPromptSubmit": [
                        {
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": "/path/to/claude-hook-advisor --hook"
                                }
                            ]
                        }
                    ]
                }
            });
            fs::write(".claude/settings.local.json", serde_json::to_string_pretty(&settings_content).unwrap()).unwrap();
            
            let result = hooks_already_exist().unwrap();
            assert!(result, "Should return true when UserPromptSubmit hooks are present");
        });
    }
    
    #[test]
    fn test_hooks_already_exist_prefers_local_settings() {
        with_temp_dir(|| {
            // Create .claude directory
            fs::create_dir_all(".claude").unwrap();
            
            // Create shared settings with our hooks
            let shared_settings = json!({
                "hooks": {
                    "PreToolUse": [
                        {
                            "matcher": "Bash",
                            "hooks": [
                                {
                                    "type": "command",
                                    "command": "claude-hook-advisor --hook"
                                }
                            ]
                        }
                    ]
                }
            });
            fs::write(".claude/settings.json", serde_json::to_string_pretty(&shared_settings).unwrap()).unwrap();
            
            // Create local settings without our hooks
            let local_settings = json!({});
            fs::write(".claude/settings.local.json", serde_json::to_string_pretty(&local_settings).unwrap()).unwrap();
            
            let result = hooks_already_exist().unwrap();
            assert!(!result, "Should check local settings first and return false when they don't have our hooks");
        });
    }
    
    #[test] 
    fn test_create_example_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.toml");
        
        create_example_config(config_path.to_str().unwrap()).unwrap();
        
        let content = fs::read_to_string(&config_path).unwrap();
        
        // Check that all required sections are present
        assert!(content.contains("[commands]"));
        assert!(content.contains("[semantic_directories]"));
        assert!(content.contains("[directory_variables]"));
        
        // Check that default aliases are present
        assert!(content.contains("docs = \"~/Documents/Documentation\""));
        assert!(content.contains("project_docs = \"~/Documents/Documentation/{project}\""));
        
        // Check that comments are present
        assert!(content.contains("# Claude Hook Advisor Configuration"));
        assert!(content.contains("# npm = \"bun\""));
    }
    
    #[test]
    fn test_ensure_config_sections_missing_sections() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.toml");
        
        // Create minimal config missing sections
        fs::write(&config_path, "# Minimal config\n").unwrap();
        
        ensure_config_sections(config_path.to_str().unwrap()).unwrap();
        
        let content = fs::read_to_string(&config_path).unwrap();
        
        // Check that all sections were added
        assert!(content.contains("[commands]"));
        assert!(content.contains("[semantic_directories]"));
        assert!(content.contains("[directory_variables]"));
        
        // Check that examples were added
        assert!(content.contains("docs = \"~/Documents/Documentation\""));
        assert!(content.contains("# npm = \"bun\""));
    }
    
    #[test]
    fn test_ensure_config_sections_all_sections_present() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.toml");
        
        let existing_config = r#"# Existing config
[commands]
npm = "bun"

[semantic_directories]
docs = "~/Documents"

[directory_variables]
project = "test"
"#;
        fs::write(&config_path, existing_config).unwrap();
        
        ensure_config_sections(config_path.to_str().unwrap()).unwrap();
        
        let content = fs::read_to_string(&config_path).unwrap();
        
        // Should be unchanged since all sections already exist
        assert_eq!(content, existing_config);
    }
}