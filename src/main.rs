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
use chrono::{DateTime, Utc};

/// Enhanced configuration structure supporting both static and learned mappings.
/// 
/// This struct supports the evolution from simple static command mappings to an
/// intelligent learning system that can adapt based on user preferences and context.
#[derive(Debug, Deserialize, Serialize)]
struct Config {
    /// Legacy static command mappings (preserved for backwards compatibility)
    commands: HashMap<String, String>,
    /// Learned mappings with rich metadata and confidence tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    learned: Option<LearnedMappings>,
    /// Metadata about the learning system's state and statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    learning_meta: Option<LearningMetadata>,
    /// Confidence scores for specific command mappings
    #[serde(skip_serializing_if = "Option::is_none")]
    confidence_scores: Option<HashMap<String, f32>>,
    /// Commands that should never be suggested (user explicitly rejected)
    #[serde(skip_serializing_if = "Option::is_none")]
    never_suggest: Option<HashMap<String, String>>,
}

/// Learned command mappings organized by scope and context.
/// 
/// Provides a hierarchical system for storing learned preferences:
/// - Global: Universal preferences across all projects
/// - Project: Specific to the current project/directory
/// - Context: Conditional mappings based on project type or environment
#[derive(Debug, Deserialize, Serialize)]
struct LearnedMappings {
    /// Global mappings that apply across all projects
    #[serde(skip_serializing_if = "Option::is_none")]
    global: Option<HashMap<String, LearnedMapping>>,
    /// Project-specific mappings for the current working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    project: Option<HashMap<String, LearnedMapping>>,
    /// Context-specific mappings (e.g., react_projects, python_projects)
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<HashMap<String, HashMap<String, LearnedMapping>>>,
}

/// A single learned command mapping with rich metadata.
/// 
/// Contains all the information needed to make intelligent decisions about
/// command suggestions, including confidence, timing, and learning source.
#[derive(Debug, Deserialize, Serialize)]
struct LearnedMapping {
    /// The replacement command to suggest
    replacement: String,
    /// Confidence level (0.0 to 1.0) - higher means more certain
    confidence: f32,
    /// When this mapping was first learned
    learned_at: DateTime<Utc>,
    /// How this mapping was learned (user_preference, user_correction, etc.)
    learned_from: String,
    /// How many times this mapping has been suggested and accepted
    #[serde(skip_serializing_if = "Option::is_none")]
    usage_count: Option<u32>,
    /// Additional context about when this mapping applies
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
}

/// Metadata about the learning system's current state.
/// 
/// Tracks overall statistics and system state to enable analytics
/// and debugging of the learning behavior.
#[derive(Debug, Deserialize, Serialize)]
struct LearningMetadata {
    /// When the configuration was last updated
    last_updated: DateTime<Utc>,
    /// Total number of mappings learned across all scopes
    total_mappings_learned: u32,
    /// Number of mappings learned in the current session
    session_mappings: u32,
    /// Number of times user corrected or rejected suggestions
    user_corrections: u32,
    /// Configuration format version for migration support
    version: String,
}

/// Result of resolving a command against all available mappings.
/// 
/// Contains the final decision along with metadata about which source
/// provided the mapping and how confident we are in the suggestion.
#[derive(Debug)]
struct ResolvedMapping {
    /// The original command that was matched
    #[allow(dead_code)] // Will be used in future milestones
    original_command: String,
    /// The suggested replacement command
    replacement_command: String,
    /// Confidence level in this mapping (0.0 to 1.0)
    #[allow(dead_code)] // Will be used in future milestones
    confidence: f32,
    /// Which source provided this mapping (static, global, project, context, never_suggest)
    source: MappingSource,
    /// Human-readable reason for this suggestion
    reason: String,
}

/// Source of a command mapping for priority resolution.
/// 
/// Defines the hierarchy of mapping sources, with earlier variants
/// taking priority over later ones in conflict resolution.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum MappingSource {
    /// User explicitly rejected this mapping
    NeverSuggest,
    /// Project-specific learned mapping
    ProjectLearned,
    /// Context-specific learned mapping
    ContextLearned,
    /// Global learned mapping
    GlobalLearned,
    /// Static configuration mapping
    Static,
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

/// Loads configuration from a TOML file with migration support.
/// 
/// Handles both legacy format (simple commands hash) and enhanced format
/// (with learned mappings). Performs automatic migration when needed while
/// preserving backwards compatibility.
/// 
/// # Arguments
/// * `config_path` - Path to the .claude-hook-advisor.toml file
/// 
/// # Returns
/// * `Ok(Config)` - Loaded configuration or empty config if file not found
/// * `Err` - If file exists but cannot be read or parsed
fn load_config(config_path: &str) -> Result<Config> {
    if !Path::new(config_path).exists() {
        eprintln!("Warning: Config file '{config_path}' not found. No command mappings will be applied.");
        return Ok(create_empty_config());
    }

    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {config_path}"))?;

    // Try to parse as enhanced format first
    match toml::from_str::<Config>(&content) {
        Ok(config) => {
            // Validate and potentially migrate the configuration
            validate_and_migrate_config(config, config_path)
        }
        Err(enhanced_err) => {
            // Fall back to legacy format parsing
            #[derive(Deserialize)]
            struct LegacyConfig {
                commands: HashMap<String, String>,
            }
            
            match toml::from_str::<LegacyConfig>(&content) {
                Ok(legacy_config) => {
                    eprintln!("Info: Migrating legacy configuration format to enhanced format.");
                    Ok(migrate_legacy_config(legacy_config.commands))
                }
                Err(_legacy_err) => {
                    // If both parsing attempts fail, return the enhanced format error
                    // as it's more likely to be the intended format
                    Err(enhanced_err).with_context(|| {
                        format!("Failed to parse config file as either enhanced or legacy format: {config_path}")
                    })
                }
            }
        }
    }
}

/// Creates an empty configuration with proper defaults.
fn create_empty_config() -> Config {
    Config {
        commands: HashMap::new(),
        learned: None,
        learning_meta: Some(LearningMetadata {
            last_updated: Utc::now(),
            total_mappings_learned: 0,
            session_mappings: 0,
            user_corrections: 0,
            version: "0.2.0".to_string(),
        }),
        confidence_scores: None,
        never_suggest: None,
    }
}

/// Migrates a legacy configuration to the enhanced format.
/// 
/// Preserves all existing static command mappings while initializing
/// the new learned mapping structures for future use.
fn migrate_legacy_config(legacy_commands: HashMap<String, String>) -> Config {
    Config {
        commands: legacy_commands,
        learned: Some(LearnedMappings {
            global: Some(HashMap::new()),
            project: Some(HashMap::new()),
            context: Some(HashMap::new()),
        }),
        learning_meta: Some(LearningMetadata {
            last_updated: Utc::now(),
            total_mappings_learned: 0,
            session_mappings: 0,
            user_corrections: 0,
            version: "0.2.0".to_string(),
        }),
        confidence_scores: Some(HashMap::new()),
        never_suggest: Some(HashMap::new()),
    }
}

/// Validates and potentially migrates an enhanced configuration.
/// 
/// Ensures the configuration has all required fields initialized
/// and updates version information if needed.
fn validate_and_migrate_config(mut config: Config, _config_path: &str) -> Result<Config> {
    // Initialize missing optional fields with empty collections
    if config.learned.is_none() {
        config.learned = Some(LearnedMappings {
            global: Some(HashMap::new()),
            project: Some(HashMap::new()),
            context: Some(HashMap::new()),
        });
    }
    
    if config.learning_meta.is_none() {
        config.learning_meta = Some(LearningMetadata {
            last_updated: Utc::now(),
            total_mappings_learned: 0,
            session_mappings: 0,
            user_corrections: 0,
            version: "0.2.0".to_string(),
        });
    }
    
    if config.confidence_scores.is_none() {
        config.confidence_scores = Some(HashMap::new());
    }
    
    if config.never_suggest.is_none() {
        config.never_suggest = Some(HashMap::new());
    }

    // Update version if it's outdated (future migration logic can go here)
    if let Some(ref mut meta) = config.learning_meta {
        if meta.version != "0.2.0" {
            meta.version = "0.2.0".to_string();
            meta.last_updated = Utc::now();
        }
    }

    Ok(config)
}

/// Resolves command mappings using the enhanced priority system.
/// 
/// Searches through all available mapping sources (never_suggest, project, context,
/// global, static) and returns the highest-priority match. Includes confidence
/// threshold filtering to ensure only reliable suggestions are made.
/// 
/// # Arguments
/// * `config` - Enhanced configuration containing all mapping sources
/// * `command` - The bash command to check against mappings
/// 
/// # Returns
/// * `Ok(Some(ResolvedMapping))` - If a mapping is found above confidence threshold
/// * `Ok(None)` - If no mappings match or all are below confidence threshold
/// * `Err` - If regex compilation fails
fn resolve_command_mapping(config: &Config, command: &str) -> Result<Option<ResolvedMapping>> {
    const DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.7;
    
    // Check never_suggest first - these block all other suggestions
    if let Some(never_suggest) = &config.never_suggest {
        if let Some(blocked_replacement) = check_pattern_matches(never_suggest, command)? {
            return Ok(Some(ResolvedMapping {
                original_command: command.to_string(),
                replacement_command: blocked_replacement.suggested_command,
                confidence: 1.0, // Always confident about explicit rejections
                source: MappingSource::NeverSuggest,
                reason: format!("User explicitly rejected suggestion: {}", blocked_replacement.reason),
            }));
        }
    }

    // Check learned mappings in priority order: project -> context -> global
    if let Some(learned) = &config.learned {
        // Project-specific mappings (highest priority)
        if let Some(project_mappings) = &learned.project {
            if let Some(resolved) = resolve_learned_mapping(
                project_mappings, 
                command, 
                MappingSource::ProjectLearned,
                DEFAULT_CONFIDENCE_THRESHOLD
            )? {
                return Ok(Some(resolved));
            }
        }

        // Context-specific mappings
        if let Some(context_mappings) = &learned.context {
            for (context_name, mappings) in context_mappings {
                if let Some(resolved) = resolve_learned_mapping(
                    mappings,
                    command,
                    MappingSource::ContextLearned,
                    DEFAULT_CONFIDENCE_THRESHOLD
                )? {
                    // Add context information to the reason
                    let mut resolved = resolved;
                    resolved.reason = format!("{} (context: {})", resolved.reason, context_name);
                    return Ok(Some(resolved));
                }
            }
        }

        // Global learned mappings
        if let Some(global_mappings) = &learned.global {
            if let Some(resolved) = resolve_learned_mapping(
                global_mappings,
                command,
                MappingSource::GlobalLearned,
                DEFAULT_CONFIDENCE_THRESHOLD
            )? {
                return Ok(Some(resolved));
            }
        }
    }

    // Fall back to static mappings (legacy support)
    if let Some(pattern_match) = check_pattern_matches(&config.commands, command)? {
        return Ok(Some(ResolvedMapping {
            original_command: command.to_string(),
            replacement_command: pattern_match.suggested_command,
            confidence: 1.0, // Static mappings are always fully confident
            source: MappingSource::Static,
            reason: pattern_match.reason,
        }));
    }

    Ok(None)
}

/// Helper struct for pattern matching results.
struct PatternMatch {
    suggested_command: String,
    reason: String,
}

/// Checks a HashMap of command patterns against the input command.
/// 
/// Uses word-boundary regex matching to ensure exact command matches.
fn check_pattern_matches(
    patterns: &HashMap<String, String>, 
    command: &str
) -> Result<Option<PatternMatch>> {
    for (pattern, replacement) in patterns {
        // Create regex pattern to match the command at word boundaries
        let regex_pattern = format!(r"\b{}\b", regex::escape(pattern));
        let regex = Regex::new(&regex_pattern)?;

        if regex.is_match(command) {
            // Generate suggested replacement
            let suggested_command = regex.replace_all(command, replacement);
            let reason = format!(
                "Command '{pattern}' is mapped to use '{replacement}' instead. Try: {suggested_command}"
            );
            return Ok(Some(PatternMatch {
                suggested_command: suggested_command.to_string(),
                reason,
            }));
        }
    }
    Ok(None)
}

/// Resolves a learned mapping with confidence threshold filtering.
/// 
/// Checks learned mappings and only returns results that meet the confidence threshold.
fn resolve_learned_mapping(
    mappings: &HashMap<String, LearnedMapping>,
    command: &str,
    source: MappingSource,
    confidence_threshold: f32,
) -> Result<Option<ResolvedMapping>> {
    for (pattern, learned_mapping) in mappings {
        // Skip if confidence is below threshold
        if learned_mapping.confidence < confidence_threshold {
            continue;
        }

        // Create regex pattern to match the command at word boundaries
        let regex_pattern = format!(r"\b{}\b", regex::escape(pattern));
        let regex = Regex::new(&regex_pattern)?;

        if regex.is_match(command) {
            // Generate suggested replacement
            let suggested_command = regex.replace_all(command, &learned_mapping.replacement);
            let reason = format!(
                "Learned preference: '{}' -> '{}' (confidence: {:.1}%). Try: {}",
                pattern,
                learned_mapping.replacement,
                learned_mapping.confidence * 100.0,
                suggested_command
            );

            return Ok(Some(ResolvedMapping {
                original_command: command.to_string(),
                replacement_command: suggested_command.to_string(),
                confidence: learned_mapping.confidence,
                source,
                reason,
            }));
        }
    }
    Ok(None)
}

/// Legacy wrapper function for backwards compatibility.
/// 
/// Maintains the existing function signature while using the new resolution system.
fn check_command_mappings(config: &Config, command: &str) -> Result<Option<(String, String)>> {
    match resolve_command_mapping(config, command)? {
        Some(resolved) => {
            // Handle never_suggest case - return None to allow command
            if resolved.source == MappingSource::NeverSuggest {
                return Ok(None);
            }
            Ok(Some((resolved.reason, resolved.replacement_command)))
        }
        None => Ok(None),
    }
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
    let config = Config {
        commands,
        learned: None,
        learning_meta: Some(LearningMetadata {
            last_updated: Utc::now(),
            total_mappings_learned: 0,
            session_mappings: 0,
            user_corrections: 0,
            version: "0.2.0".to_string(),
        }),
        confidence_scores: None,
        never_suggest: None,
    };
    
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

        let config = Config {
            commands,
            learned: None,
            learning_meta: None,
            confidence_scores: None,
            never_suggest: None,
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

        // Test no mapping
        let result = check_command_mappings(&config, "ls -la").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_legacy_config_migration() {
        // Test that legacy format can be loaded directly (optional fields make it compatible)
        let legacy_toml = r#"
[commands]
npm = "bun"
yarn = "bun"
"#;
        
        let legacy_config: Result<crate::Config, _> = toml::from_str(legacy_toml);
        assert!(legacy_config.is_ok()); // Should succeed with enhanced struct (optional fields)
        
        let loaded_config = legacy_config.unwrap();
        // Legacy format should have commands but no learned data initially
        assert_eq!(loaded_config.commands.get("npm"), Some(&"bun".to_string()));
        assert_eq!(loaded_config.commands.get("yarn"), Some(&"bun".to_string()));
        assert!(loaded_config.learned.is_none());
        assert!(loaded_config.learning_meta.is_none());
        
        // Test the validation/migration function that would be called by load_config
        let migrated = validate_and_migrate_config(loaded_config, "test.toml").unwrap();
        
        // After validation, all optional fields should be initialized
        assert!(migrated.learned.is_some());
        assert!(migrated.learning_meta.is_some());
        assert!(migrated.confidence_scores.is_some());
        assert!(migrated.never_suggest.is_some());
    }

    #[test]
    fn test_enhanced_config_resolution() {
        let mut commands = HashMap::new();
        commands.insert("npm".to_string(), "bun".to_string());

        let mut global_learned = HashMap::new();
        global_learned.insert("grep".to_string(), LearnedMapping {
            replacement: "rg".to_string(),
            confidence: 0.95,
            learned_at: Utc::now(),
            learned_from: "user_preference".to_string(),
            usage_count: Some(10),
            context: None,
        });

        let mut project_learned = HashMap::new();
        project_learned.insert("npm".to_string(), LearnedMapping {
            replacement: "yarn".to_string(),
            confidence: 0.90,
            learned_at: Utc::now(),
            learned_from: "project_preference".to_string(),
            usage_count: Some(5),
            context: Some("react_project".to_string()),
        });

        let config = Config {
            commands,
            learned: Some(LearnedMappings {
                global: Some(global_learned),
                project: Some(project_learned),
                context: None,
            }),
            learning_meta: Some(LearningMetadata {
                last_updated: Utc::now(),
                total_mappings_learned: 2,
                session_mappings: 0,
                user_corrections: 0,
                version: "0.2.0".to_string(),
            }),
            confidence_scores: Some(HashMap::new()),
            never_suggest: Some(HashMap::new()),
        };

        // Test priority resolution: project should override static
        let result = resolve_command_mapping(&config, "npm install").unwrap();
        assert!(result.is_some());
        let resolved = result.unwrap();
        assert_eq!(resolved.replacement_command, "yarn install");
        assert_eq!(resolved.source, MappingSource::ProjectLearned);

        // Test global learned mapping
        let result = resolve_command_mapping(&config, "grep pattern").unwrap();
        assert!(result.is_some());
        let resolved = result.unwrap();
        assert_eq!(resolved.replacement_command, "rg pattern");
        assert_eq!(resolved.source, MappingSource::GlobalLearned);
    }

    #[test]
    fn test_confidence_filtering() {
        let mut global_learned = HashMap::new();
        
        // High confidence mapping (should be suggested)
        global_learned.insert("grep".to_string(), LearnedMapping {
            replacement: "rg".to_string(),
            confidence: 0.95,
            learned_at: Utc::now(),
            learned_from: "user_preference".to_string(),
            usage_count: Some(10),
            context: None,
        });
        
        // Low confidence mapping (should be filtered out)
        global_learned.insert("cat".to_string(), LearnedMapping {
            replacement: "bat".to_string(),
            confidence: 0.30, // Below 0.7 threshold
            learned_at: Utc::now(),
            learned_from: "experimental".to_string(),
            usage_count: Some(1),
            context: None,
        });

        let config = Config {
            commands: HashMap::new(),
            learned: Some(LearnedMappings {
                global: Some(global_learned),
                project: None,
                context: None,
            }),
            learning_meta: None,
            confidence_scores: None,
            never_suggest: None,
        };

        // High confidence should be suggested
        let result = resolve_command_mapping(&config, "grep pattern").unwrap();
        assert!(result.is_some());

        // Low confidence should be filtered out
        let result = resolve_command_mapping(&config, "cat file.txt").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_never_suggest_blocking() {
        let mut commands = HashMap::new();
        commands.insert("docker".to_string(), "podman".to_string());

        let mut never_suggest = HashMap::new();
        never_suggest.insert("docker".to_string(), "podman".to_string());

        let config = Config {
            commands,
            learned: None,
            learning_meta: None,
            confidence_scores: None,
            never_suggest: Some(never_suggest),
        };

        // never_suggest should block the command (return None in legacy wrapper)
        let result = check_command_mappings(&config, "docker run").unwrap();
        assert!(result.is_none());

        // But the full resolution should show the never_suggest result
        let result = resolve_command_mapping(&config, "docker run").unwrap();
        assert!(result.is_some());
        let resolved = result.unwrap();
        assert_eq!(resolved.source, MappingSource::NeverSuggest);
    }
}
