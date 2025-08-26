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
        create_smart_config(config_path)?;
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

/// Creates a smart configuration file with project-specific command mappings.
/// Detects the project type and generates appropriate command mappings.
/// Directory aliases are provided as commented examples only.
/// 
/// # Arguments
/// * `config_path` - Path where to create the configuration file
/// 
/// # Returns
/// * `Ok(())` - Configuration created successfully
/// * `Err` - If file writing fails
fn create_smart_config(config_path: &str) -> Result<()> {
    // Detect project type
    let project_type = detect_project_type()?;
    println!("🔍 Detected project type: {project_type}");
    
    // Get project-specific command mappings
    let commands = get_commands_for_project_type(&project_type);
    
    // Create config structure with actual commands but empty directories
    let config = Config {
        commands,
        semantic_directories: std::collections::HashMap::new(), // Empty - will be comments only
        features: Default::default(),
    };
    
    // Generate TOML content
    let toml_content = toml::to_string_pretty(&config)
        .with_context(|| "Failed to serialize configuration to TOML")?;
    
    // Build the complete config with header and directory examples as comments
    let _project_name = get_project_name();
    let final_content = format!(r#"# Claude Hook Advisor Configuration
# Auto-generated for {project_type} project
# This file configures command mappings and semantic directory aliases
# for use with Claude Code integration.

{toml_content}
# Semantic directory aliases - natural language directory references
# Uncomment and customize these examples:
# docs = "~/Documents/Documentation"
# central_docs = "~/Documents/Documentation"
# project_docs = "~/Documents/Documentation/my-project"
# claude_docs = "~/Documents/Documentation/claude"
"#);
    
    fs::write(config_path, final_content)
        .with_context(|| format!("Failed to write config file: {config_path}"))?;
    
    println!("✅ Created smart configuration for {project_type} project");
    
    // Show what was configured
    if !config.commands.is_empty() {
        println!("📝 Command mappings configured:");
        for (from, to) in &config.commands {
            println!("   {from} → {to}");
        }
    } else {
        println!("📝 No specific command mappings for {project_type} - using general alternatives");
    }
    
    Ok(())
}

/// Detects the project type by examining files in the current directory.
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

/// Creates project-specific command mappings based on detected project type.
/// 
/// # Arguments
/// * `project_type` - The detected project type
/// 
/// # Returns
/// * `HashMap<String, String>` - Command mappings for the project
fn get_commands_for_project_type(project_type: &str) -> std::collections::HashMap<String, String> {
    let mut commands = std::collections::HashMap::new();
    
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
            // General project - modern CLI alternatives
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

/// Gets the current project name for variable substitution.
fn get_project_name() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|dir| dir.file_name().map(|name| name.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".to_string())
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
        config_content.push_str("project_docs = \"~/Documents/Documentation/my-project\"\n");
        config_content.push_str("claude_docs = \"~/Documents/Documentation/claude\"\n\n");
        needs_update = true;
        println!("✅ Added [semantic_directories] section with default aliases");
    }
    
    if !config_content.contains("[features]") {
        config_content.push_str("# Feature flags - enable/disable specific functionality\n");
        config_content.push_str("[features]\n");
        config_content.push_str("hashtag_search_advisory = true    # Enable hashtag search pattern guidance\n\n");
        needs_update = true;
        println!("✅ Added [features] section with hashtag search enabled");
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
    fn test_ensure_config_sections() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.toml");
        
        // Create a minimal config file missing features section
        fs::write(&config_path, r#"[commands]
npm = "bun"

[semantic_directories]
docs = "~/Documents/Documentation"
"#).unwrap();
        
        // Run ensure_config_sections
        ensure_config_sections(config_path.to_str().unwrap()).unwrap();
        
        // Check that features section was added
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[features]"));
        assert!(content.contains("hashtag_search_advisory = true"));
        assert!(content.contains("# Enable hashtag search pattern guidance"));
    }

    #[test]
    fn test_ensure_config_sections_no_update_needed() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.toml");
        
        // Create a complete config file
        fs::write(&config_path, r#"[commands]
npm = "bun"

[semantic_directories]
docs = "~/Documents/Documentation"

[features]
hashtag_search_advisory = true
"#).unwrap();
        
        let original_content = fs::read_to_string(&config_path).unwrap();
        
        // Run ensure_config_sections
        ensure_config_sections(config_path.to_str().unwrap()).unwrap();
        
        // Check that content is unchanged
        let updated_content = fs::read_to_string(&config_path).unwrap();
        assert_eq!(original_content, updated_content);
    }

    #[test] 
    fn test_create_example_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.toml");
        
        create_smart_config(config_path.to_str().unwrap()).unwrap();
        
        let content = fs::read_to_string(&config_path).unwrap();
        
        // Check that all required sections are present
        assert!(content.contains("[commands]"));
        assert!(content.contains("[semantic_directories]"));
        assert!(content.contains("[features]"));
        
        // Check that default aliases are present
        assert!(content.contains("docs = \"~/Documents/Documentation\""));
        assert!(content.contains("docs = \"~/Documents/Documentation\""));
        
        // Check that comments are present
        assert!(content.contains("# Claude Hook Advisor Configuration"));
        assert!(content.contains("# Uncomment and customize these examples:"));
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

[features]
hashtag_search_advisory = true
"#;
        fs::write(&config_path, existing_config).unwrap();
        
        ensure_config_sections(config_path.to_str().unwrap()).unwrap();
        
        let content = fs::read_to_string(&config_path).unwrap();
        
        // Should be unchanged since all sections already exist
        assert_eq!(content, existing_config);
    }
}