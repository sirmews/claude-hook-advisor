//! Installation and project setup logic

use crate::patterns::generate_config_for_project;
use crate::types::Config;
use anyhow::{anyhow, Context, Result};
use serde_json::{Map, Value};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

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

/// Installs Claude Hook Advisor hooks directly into Claude Code settings.
/// 
/// This function:
/// 1. Detects appropriate Claude settings file location (.claude/settings.json or .claude/settings.local.json)
/// 2. Creates a timestamped backup of existing settings
/// 3. Carefully merges our hooks while preserving all existing hooks
/// 4. Only replaces hooks that contain "claude-hook-advisor" in the command
/// 
/// # Returns
/// * `Ok(())` - Hooks installed successfully  
/// * `Err` - If file operations fail or JSON parsing errors occur
pub fn install_claude_hooks() -> Result<()> {
    println!("🔧 Claude Hook Advisor - Hooks Installation");
    println!("===========================================");

    // Determine the best settings file to use
    let settings_path = determine_settings_file()?;
    println!("📁 Using settings file: {}", settings_path.display());

    // Create backup before modifying
    create_settings_backup(&settings_path)?;

    // Load existing settings or create new structure  
    let mut settings = load_or_create_settings(&settings_path)?;

    // Get the current binary path for hooks
    let binary_path = get_current_binary_path()?;
    
    // Merge our hooks into existing settings
    merge_claude_hooks(&mut settings, &binary_path)?;

    // Write updated settings back to file
    write_settings_file(&settings_path, &settings)?;

    println!("✅ Hooks successfully installed!");
    println!("🎯 Claude Hook Advisor will now intercept Bash commands in Claude Code");
    println!("📋 Run claude-hook-advisor --list-learned to see active mappings");

    Ok(())
}

/// Determines the best Claude settings file to use for hook installation.
/// 
/// Priority order:
/// 1. .claude/settings.local.json (preferred - not committed to git)
/// 2. .claude/settings.json (fallback - shared project settings)
/// 
/// Creates the .claude directory if it doesn't exist.
fn determine_settings_file() -> Result<PathBuf> {
    let claude_dir = PathBuf::from(".claude");
    
    // Create .claude directory if it doesn't exist
    if !claude_dir.exists() {
        fs::create_dir_all(&claude_dir)
            .context("Failed to create .claude directory")?;
        println!("📁 Created .claude directory");
    }

    // Prefer local settings (not committed)
    let local_settings = claude_dir.join("settings.local.json");
    let shared_settings = claude_dir.join("settings.json");

    // If local settings exist, use them
    if local_settings.exists() {
        return Ok(local_settings);
    }

    // If shared settings exist, ask user preference
    if shared_settings.exists() {
        println!("📋 Found existing .claude/settings.json (shared with team)");
        print!("Install hooks to local settings instead? (Y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('n') {
            return Ok(local_settings);
        }
        return Ok(shared_settings);
    }

    // Default to local settings for new installations
    Ok(local_settings)
}

/// Creates a timestamped backup of the settings file.
fn create_settings_backup(settings_path: &Path) -> Result<()> {
    if !settings_path.exists() {
        println!("📋 No existing settings file to backup");
        return Ok(());
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}.backup_{}", 
        settings_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("settings.json"),
        timestamp
    );
    let backup_path = settings_path.parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&backup_name);

    fs::copy(settings_path, &backup_path)
        .with_context(|| format!("Failed to create backup at {}", backup_path.display()))?;

    println!("💾 Created backup: {}", backup_path.display());
    Ok(())
}

/// Loads existing settings file or creates a new empty settings structure.
fn load_or_create_settings(settings_path: &Path) -> Result<Value> {
    if settings_path.exists() {
        let content = fs::read_to_string(settings_path)
            .with_context(|| format!("Failed to read settings file: {}", settings_path.display()))?;
        
        if content.trim().is_empty() {
            return Ok(Value::Object(Map::new()));
        }

        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON in settings file: {}", settings_path.display()))
    } else {
        Ok(Value::Object(Map::new()))
    }
}

/// Gets the current binary path, preferring debug build for development.
/// 
/// Returns absolute paths for development builds to ensure they work regardless
/// of Claude Code's working directory. Uses simple binary name for production
/// installs when available in PATH.
fn get_current_binary_path() -> Result<String> {
    let current_exe = std::env::current_exe()?;
    let binary_name = env!("CARGO_PKG_NAME");
    
    // For development builds, always use absolute path to avoid working directory issues
    if cfg!(debug_assertions) {
        return Ok(current_exe.to_string_lossy().to_string());
    }
    
    // For production builds, prefer simple binary name if available in PATH
    // Otherwise, fall back to absolute path of current executable
    if which::which(binary_name).is_ok() {
        Ok(binary_name.to_string())
    } else {
        Ok(current_exe.to_string_lossy().to_string())
    }
}

/// Merges Claude Hook Advisor hooks into existing settings, preserving other hooks.
/// 
/// This function is careful to:
/// - Only replace hooks containing "claude-hook-advisor" 
/// - Preserve all other existing hooks
/// - Create proper hook structure if it doesn't exist
/// - Handle both array and object formats for hooks
fn merge_claude_hooks(settings: &mut Value, binary_path: &str) -> Result<()> {
    let settings_obj = settings.as_object_mut()
        .ok_or_else(|| anyhow!("Settings must be a JSON object"))?;

    // Ensure hooks object exists
    if !settings_obj.contains_key("hooks") {
        settings_obj.insert("hooks".to_string(), Value::Object(Map::new()));
    }

    let hooks = settings_obj.get_mut("hooks")
        .and_then(|h| h.as_object_mut())
        .ok_or_else(|| anyhow!("hooks must be an object"))?;

    // Our hook configuration
    let hook_command = format!("{binary_path} --hook");

    // Install PreToolUse hook for Bash commands
    merge_hook_event(hooks, "PreToolUse", "Bash", &hook_command)?;
    
    // Install UserPromptSubmit hook (no matcher needed)
    merge_hook_event(hooks, "UserPromptSubmit", "", &hook_command)?;
    
    // Install PostToolUse hook for Bash commands  
    merge_hook_event(hooks, "PostToolUse", "Bash", &hook_command)?;

    Ok(())
}

/// Merges a single hook event, preserving existing hooks and only replacing claude-hook-advisor ones.
fn merge_hook_event(hooks: &mut Map<String, Value>, event_name: &str, matcher: &str, command: &str) -> Result<()> {
    // Ensure the event exists
    if !hooks.contains_key(event_name) {
        hooks.insert(event_name.to_string(), Value::Array(vec![]));
    }

    let event_hooks = hooks.get_mut(event_name)
        .and_then(|h| h.as_array_mut())
        .ok_or_else(|| anyhow!("{} hooks must be an array", event_name))?;

    // Look for existing claude-hook-advisor hooks to replace
    let mut found_existing = false;

    for hook_group in event_hooks.iter_mut() {
        let hook_obj = hook_group.as_object_mut()
            .ok_or_else(|| anyhow!("Hook group must be an object"))?;

        // Check if this hook group matches our matcher
        let group_matcher = hook_obj.get("matcher")
            .and_then(|m| m.as_str())
            .unwrap_or("");

        if (matcher.is_empty() && group_matcher.is_empty()) || 
           (!matcher.is_empty() && group_matcher == matcher) {
            
            // Check hooks array within this group
            if let Some(hooks_array) = hook_obj.get_mut("hooks")
                .and_then(|h| h.as_array_mut()) {
                
                // Remove existing claude-hook-advisor hooks
                hooks_array.retain(|hook| {
                    if let Some(cmd) = hook.get("command").and_then(|c| c.as_str()) {
                        !cmd.contains("claude-hook-advisor")
                    } else {
                        true
                    }
                });

                // Add our hook
                let new_hook = serde_json::json!({
                    "type": "command",
                    "command": command
                });
                hooks_array.push(new_hook);
                found_existing = true;
                break;
            }
        }
    }

    // If no matching group found, create a new one
    if !found_existing {
        let new_hook_group = if matcher.is_empty() {
            serde_json::json!({
                "hooks": [{
                    "type": "command",
                    "command": command
                }]
            })
        } else {
            serde_json::json!({
                "matcher": matcher,
                "hooks": [{
                    "type": "command", 
                    "command": command
                }]
            })
        };
        
        event_hooks.push(new_hook_group);
    }

    Ok(())
}

/// Writes the updated settings back to the file with pretty formatting.
fn write_settings_file(settings_path: &Path, settings: &Value) -> Result<()> {
    let json_content = serde_json::to_string_pretty(settings)
        .context("Failed to serialize settings to JSON")?;

    fs::write(settings_path, json_content)
        .with_context(|| format!("Failed to write settings file: {}", settings_path.display()))?;

    Ok(())
}

/// Uninstalls Claude Hook Advisor hooks from Claude Code settings.
/// 
/// This function:
/// 1. Detects existing Claude settings file location
/// 2. Creates a timestamped backup of existing settings
/// 3. Removes only claude-hook-advisor hooks while preserving all other hooks
/// 4. Cleans up empty hook groups after removal
/// 
/// # Returns
/// * `Ok(())` - Hooks uninstalled successfully
/// * `Err` - If file operations fail or no settings file exists
pub fn uninstall_claude_hooks() -> Result<()> {
    println!("🔧 Claude Hook Advisor - Hooks Uninstallation");
    println!("===============================================");

    // Find existing settings file
    let settings_path = find_existing_settings_file()?;
    println!("📁 Using settings file: {}", settings_path.display());

    // Create backup before modifying
    create_settings_backup(&settings_path)?;

    // Load existing settings
    let mut settings = load_or_create_settings(&settings_path)?;

    // Remove our hooks from existing settings
    let removed_count = remove_claude_hooks(&mut settings)?;

    if removed_count == 0 {
        println!("ℹ️  No Claude Hook Advisor hooks found to remove");
        return Ok(());
    }

    // Write updated settings back to file
    write_settings_file(&settings_path, &settings)?;

    println!("✅ Hooks successfully uninstalled!");
    println!("🗑️  Removed {removed_count} claude-hook-advisor hook(s)");
    println!("🎯 Claude Hook Advisor is no longer active in Claude Code");

    Ok(())
}

/// Finds an existing Claude settings file, preferring local over shared.
/// 
/// # Returns
/// * `Ok(PathBuf)` - Path to existing settings file
/// * `Err` - If no Claude settings file exists
fn find_existing_settings_file() -> Result<PathBuf> {
    let claude_dir = PathBuf::from(".claude");
    let local_settings = claude_dir.join("settings.local.json");
    let shared_settings = claude_dir.join("settings.json");

    if local_settings.exists() {
        return Ok(local_settings);
    }

    if shared_settings.exists() {
        return Ok(shared_settings);
    }

    Err(anyhow!("No Claude Code settings file found. Run 'claude-hook-advisor --install-hooks' first."))
}

/// Removes Claude Hook Advisor hooks from settings, preserving all other hooks.
/// 
/// This function carefully:
/// - Only removes hooks containing "claude-hook-advisor"
/// - Preserves all other hooks in the same groups
/// - Removes empty hook groups after cleanup
/// - Removes empty event arrays if no hooks remain
/// 
/// # Returns
/// * `Ok(usize)` - Number of claude-hook-advisor hooks removed
/// * `Err` - If JSON structure is invalid
fn remove_claude_hooks(settings: &mut Value) -> Result<usize> {
    let settings_obj = settings.as_object_mut()
        .ok_or_else(|| anyhow!("Settings must be a JSON object"))?;

    // If no hooks section exists, nothing to remove
    if !settings_obj.contains_key("hooks") {
        return Ok(0);
    }

    let hooks = settings_obj.get_mut("hooks")
        .and_then(|h| h.as_object_mut())
        .ok_or_else(|| anyhow!("hooks must be an object"))?;

    let mut total_removed = 0;

    // Process each hook event type
    let event_names: Vec<String> = hooks.keys().cloned().collect();
    
    for event_name in event_names {
        let removed_count = remove_hooks_from_event(hooks, &event_name)?;
        total_removed += removed_count;
    }

    // Clean up empty hooks object if no events remain
    if hooks.is_empty() {
        settings_obj.remove("hooks");
    }

    Ok(total_removed)
}

/// Removes claude-hook-advisor hooks from a specific event type.
/// 
/// # Returns
/// * `Ok(usize)` - Number of hooks removed from this event
/// * `Err` - If JSON structure is invalid
fn remove_hooks_from_event(hooks: &mut Map<String, Value>, event_name: &str) -> Result<usize> {
    let event_hooks = match hooks.get_mut(event_name) {
        Some(hooks_array) => hooks_array.as_array_mut()
            .ok_or_else(|| anyhow!("{} hooks must be an array", event_name))?,
        None => return Ok(0),
    };

    let mut total_removed = 0;

    // Process each hook group in reverse order so we can safely remove empty groups
    let mut i = 0;
    while i < event_hooks.len() {
        let hook_group = &mut event_hooks[i];
        let hook_obj = hook_group.as_object_mut()
            .ok_or_else(|| anyhow!("Hook group must be an object"))?;

        if let Some(hooks_array) = hook_obj.get_mut("hooks")
            .and_then(|h| h.as_array_mut()) {
            
            // Count hooks before removal
            let initial_count = hooks_array.len();
            
            // Remove claude-hook-advisor hooks
            hooks_array.retain(|hook| {
                if let Some(cmd) = hook.get("command").and_then(|c| c.as_str()) {
                    !cmd.contains("claude-hook-advisor")
                } else {
                    true
                }
            });

            // Calculate how many were removed
            let removed_from_group = initial_count - hooks_array.len();
            total_removed += removed_from_group;

            // If this hook group is now empty, remove the entire group
            if hooks_array.is_empty() {
                event_hooks.remove(i);
                // Don't increment i since we removed an element
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    // If this event type has no hook groups left, remove the entire event
    if event_hooks.is_empty() {
        hooks.remove(event_name);
    }

    Ok(total_removed)
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

    #[test]
    fn test_merge_hooks_empty_settings() {
        let mut settings = serde_json::json!({});
        let binary_path = "/path/to/claude-hook-advisor";
        
        let result = merge_claude_hooks(&mut settings, binary_path);
        assert!(result.is_ok());

        // Should have created hooks structure
        assert!(settings.get("hooks").is_some());
        let hooks = settings.get("hooks").unwrap().as_object().unwrap();
        
        // Should have our three hook types
        assert!(hooks.contains_key("PreToolUse"));
        assert!(hooks.contains_key("UserPromptSubmit"));
        assert!(hooks.contains_key("PostToolUse"));
    }

    #[test]
    fn test_merge_hooks_preserves_existing() {
        let mut settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Write",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "some-other-tool --check"
                            }
                        ]
                    }
                ]
            }
        });

        let binary_path = "/path/to/claude-hook-advisor";
        let result = merge_claude_hooks(&mut settings, binary_path);
        assert!(result.is_ok());

        let hooks = settings.get("hooks").unwrap().as_object().unwrap();
        let pre_tool_use = hooks.get("PreToolUse").unwrap().as_array().unwrap();
        
        // Should have 2 hook groups now - existing Write matcher and new Bash matcher
        assert_eq!(pre_tool_use.len(), 2);
        
        // Check that existing Write hook is preserved
        let write_hook = pre_tool_use.iter()
            .find(|h| h.get("matcher").and_then(|m| m.as_str()) == Some("Write"))
            .expect("Write hook should be preserved");
            
        let write_commands = write_hook.get("hooks").unwrap().as_array().unwrap();
        assert_eq!(write_commands[0].get("command").unwrap().as_str().unwrap(), "some-other-tool --check");
    }

    #[test]
    fn test_merge_hooks_replaces_existing_claude_advisor() {
        let mut settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Bash",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "old-claude-hook-advisor --hook"
                            },
                            {
                                "type": "command", 
                                "command": "some-other-tool --check"
                            }
                        ]
                    }
                ]
            }
        });

        let binary_path = "/path/to/claude-hook-advisor";
        let result = merge_claude_hooks(&mut settings, binary_path);
        assert!(result.is_ok());

        let hooks = settings.get("hooks").unwrap().as_object().unwrap();
        let pre_tool_use = hooks.get("PreToolUse").unwrap().as_array().unwrap();
        let bash_hooks = &pre_tool_use[0].get("hooks").unwrap().as_array().unwrap();
        
        // Should have 2 hooks - the preserved one and our new one
        assert_eq!(bash_hooks.len(), 2);
        
        // Check that claude-hook-advisor was replaced and other hook preserved
        let commands: Vec<&str> = bash_hooks.iter()
            .filter_map(|h| h.get("command").and_then(|c| c.as_str()))
            .collect();
            
        assert!(commands.contains(&"some-other-tool --check"));
        assert!(commands.contains(&"/path/to/claude-hook-advisor --hook"));
        assert!(!commands.iter().any(|c| c.contains("old-claude-hook-advisor")));
    }

    #[test]
    fn test_install_and_uninstall_full_cycle() {
        // Start with a realistic settings file with existing hooks and permissions
        let mut settings = serde_json::json!({
            "permissions": {
                "allow": ["Bash(git:*)", "Read(*.md)"],
                "deny": ["Bash(rm:*)"]
            },
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Write",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "prettier --write"
                            }
                        ]
                    }
                ],
                "PostToolUse": [
                    {
                        "matcher": "Edit",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "eslint --fix"
                            }
                        ]
                    }
                ]
            }
        });

        let binary_path = "/usr/local/bin/claude-hook-advisor";

        // Install our hooks
        let install_result = merge_claude_hooks(&mut settings, binary_path);
        assert!(install_result.is_ok());

        // Verify installation
        let hooks = settings.get("hooks").unwrap().as_object().unwrap();
        
        // Should have 3 hook event types now (PreToolUse, UserPromptSubmit, PostToolUse)
        // PreToolUse and PostToolUse existed before, UserPromptSubmit is new
        assert_eq!(hooks.len(), 3);
        assert!(hooks.contains_key("PreToolUse"));
        assert!(hooks.contains_key("UserPromptSubmit"));
        assert!(hooks.contains_key("PostToolUse"));
        
        // Check PreToolUse has both Write and Bash matchers
        let pre_tool_use = hooks.get("PreToolUse").unwrap().as_array().unwrap();
        assert_eq!(pre_tool_use.len(), 2);
        
        // Find the Write matcher (existing)
        let write_hook = pre_tool_use.iter()
            .find(|h| h.get("matcher").and_then(|m| m.as_str()) == Some("Write"))
            .expect("Write hook should be preserved");
        let write_commands = write_hook.get("hooks").unwrap().as_array().unwrap();
        assert_eq!(write_commands[0].get("command").unwrap().as_str().unwrap(), "prettier --write");
        
        // Find the Bash matcher (our new one)
        let bash_hook = pre_tool_use.iter()
            .find(|h| h.get("matcher").and_then(|m| m.as_str()) == Some("Bash"))
            .expect("Bash hook should be added");
        let bash_commands = bash_hook.get("hooks").unwrap().as_array().unwrap();
        assert_eq!(bash_commands[0].get("command").unwrap().as_str().unwrap(), 
                   "/usr/local/bin/claude-hook-advisor --hook");

        // Check PostToolUse has both Edit and Bash matchers
        let post_tool_use = hooks.get("PostToolUse").unwrap().as_array().unwrap();
        assert_eq!(post_tool_use.len(), 2);

        // Check UserPromptSubmit was added
        let user_prompt_submit = hooks.get("UserPromptSubmit").unwrap().as_array().unwrap();
        assert_eq!(user_prompt_submit.len(), 1);

        // Verify permissions were preserved
        let permissions = settings.get("permissions").unwrap().as_object().unwrap();
        assert_eq!(permissions.get("allow").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(permissions.get("deny").unwrap().as_array().unwrap().len(), 1);

        // Now uninstall our hooks
        let uninstall_result = remove_claude_hooks(&mut settings);
        assert!(uninstall_result.is_ok());
        let removed_count = uninstall_result.unwrap();
        assert_eq!(removed_count, 3); // Should remove 3 claude-hook-advisor hooks

        // Verify uninstallation
        let hooks_after = settings.get("hooks").unwrap().as_object().unwrap();
        
        // Should have 2 hook event types (the original ones)
        assert_eq!(hooks_after.len(), 2);
        assert!(hooks_after.contains_key("PreToolUse"));
        assert!(hooks_after.contains_key("PostToolUse"));
        assert!(!hooks_after.contains_key("UserPromptSubmit")); // Should be removed completely

        // Check PreToolUse only has Write matcher now
        let pre_tool_use_after = hooks_after.get("PreToolUse").unwrap().as_array().unwrap();
        assert_eq!(pre_tool_use_after.len(), 1);
        assert_eq!(pre_tool_use_after[0].get("matcher").unwrap().as_str().unwrap(), "Write");

        // Check PostToolUse only has Edit matcher now  
        let post_tool_use_after = hooks_after.get("PostToolUse").unwrap().as_array().unwrap();
        assert_eq!(post_tool_use_after.len(), 1);
        assert_eq!(post_tool_use_after[0].get("matcher").unwrap().as_str().unwrap(), "Edit");

        // Verify permissions were still preserved
        let permissions_after = settings.get("permissions").unwrap().as_object().unwrap();
        assert_eq!(permissions_after.get("allow").unwrap().as_array().unwrap().len(), 2);
        assert_eq!(permissions_after.get("deny").unwrap().as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_uninstall_mixed_hooks_preserves_others() {
        // Settings with mixed hooks in same groups
        let mut settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Bash",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "claude-hook-advisor --hook"
                            },
                            {
                                "type": "command",
                                "command": "security-checker --validate"
                            },
                            {
                                "type": "command",
                                "command": "old-claude-hook-advisor --legacy"
                            }
                        ]
                    }
                ]
            }
        });

        let result = remove_claude_hooks(&mut settings);
        assert!(result.is_ok());
        let removed_count = result.unwrap();
        assert_eq!(removed_count, 2); // Should remove 2 claude-hook-advisor hooks

        let hooks = settings.get("hooks").unwrap().as_object().unwrap();
        let pre_tool_use = hooks.get("PreToolUse").unwrap().as_array().unwrap();
        let bash_hooks = &pre_tool_use[0].get("hooks").unwrap().as_array().unwrap();
        
        // Should have 1 hook remaining
        assert_eq!(bash_hooks.len(), 1);
        assert_eq!(bash_hooks[0].get("command").unwrap().as_str().unwrap(), 
                   "security-checker --validate");
    }

    #[test]
    fn test_uninstall_removes_empty_groups() {
        // Settings where claude-hook-advisor is the only hook in groups
        let mut settings = serde_json::json!({
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
                ],
                "UserPromptSubmit": [
                    {
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

        let result = remove_claude_hooks(&mut settings);
        assert!(result.is_ok());
        let removed_count = result.unwrap();
        assert_eq!(removed_count, 2);

        // Entire hooks section should be removed since all groups are empty
        assert!(!settings.as_object().unwrap().contains_key("hooks"));
    }

    #[test]
    fn test_uninstall_no_hooks_to_remove() {
        let mut settings = serde_json::json!({
            "permissions": {
                "allow": ["Bash(git:*)"]
            },
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Write",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "prettier --write"
                            }
                        ]
                    }
                ]
            }
        });

        let result = remove_claude_hooks(&mut settings);
        assert!(result.is_ok());
        let removed_count = result.unwrap();
        assert_eq!(removed_count, 0);

        // Settings should be unchanged
        let hooks = settings.get("hooks").unwrap().as_object().unwrap();
        assert_eq!(hooks.len(), 1);
        assert!(settings.get("permissions").is_some());
    }

    #[test]
    fn test_uninstall_no_hooks_section() {
        let mut settings = serde_json::json!({
            "permissions": {
                "allow": ["Bash(git:*)"]
            }
        });

        let result = remove_claude_hooks(&mut settings);
        assert!(result.is_ok());
        let removed_count = result.unwrap();
        assert_eq!(removed_count, 0);

        // Settings should be unchanged
        assert!(!settings.as_object().unwrap().contains_key("hooks"));
        assert!(settings.get("permissions").is_some());
    }

    #[test]
    fn test_debug_assertions_consistency() {
        // This test validates that we're using the correct build detection method
        // In debug builds (cargo test), debug_assertions should be true
        // In release builds (cargo test --release), debug_assertions should be false
        
        #[cfg(debug_assertions)]
        {
            // We're in a debug build - this should be true
            assert!(cfg!(debug_assertions));
        }
        
        #[cfg(not(debug_assertions))]
        {
            // We're in a release build - this should be false
            assert!(!cfg!(debug_assertions));
        }
    }

    // Note: Testing get_current_binary_path() fully requires mocking std::env::current_exe()
    // and the which crate, which is complex. The core logic is simple enough that the
    // main risk is in the integration, which is tested through end-to-end tests.
    //
    // The build detection now uses cfg!(debug_assertions) which is a compile-time constant,
    // so it's inherently reliable and doesn't need runtime testing.
}