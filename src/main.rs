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
use tempfile::NamedTempFile;

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
    /// Execution tracking for suggestion effectiveness measurement
    #[serde(skip_serializing_if = "Option::is_none")]
    execution_history: Option<ExecutionHistory>,
}

/// Execution tracking for measuring suggestion effectiveness and learning.
/// 
/// Tracks command execution results, suggestion acceptance rates, and user behavior
/// patterns to enable dynamic confidence adjustment and never-suggest functionality.
#[derive(Debug, Deserialize, Serialize)]
struct ExecutionHistory {
    /// Record of command executions with results and metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    command_executions: Option<Vec<CommandExecution>>,
    /// Track suggestion effectiveness over time
    #[serde(skip_serializing_if = "Option::is_none")]
    suggestion_stats: Option<HashMap<String, SuggestionStats>>,
    /// Mapping correlation tracking for learning validation
    #[serde(skip_serializing_if = "Option::is_none")]
    mapping_correlations: Option<HashMap<String, MappingCorrelation>>,
    /// Track user overrides and corrections
    #[serde(skip_serializing_if = "Option::is_none")]
    user_overrides: Option<Vec<UserOverride>>,
}

/// Record of a single command execution with results and context.
/// 
/// Used to correlate suggestion effectiveness with actual command success rates
/// and enable dynamic confidence adjustment based on real-world outcomes.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct CommandExecution {
    /// The command that was executed
    command: String,
    /// Whether this command was suggested by the system
    was_suggested: bool,
    /// The original command if this was a suggested replacement
    original_command: Option<String>,
    /// Exit code of the command execution
    exit_code: Option<i32>,
    /// Whether the command succeeded (exit code 0)
    success: bool,
    /// Duration of execution in milliseconds
    duration_ms: Option<u64>,
    /// When the command was executed
    executed_at: DateTime<Utc>,
    /// Which mapping source provided this suggestion (if any)
    suggestion_source: Option<String>,
    /// Session ID for correlation with other events
    session_id: Option<String>,
}

/// Statistics for a specific command suggestion.
/// 
/// Tracks effectiveness metrics for individual command mappings to enable
/// confidence adjustment and never-suggest functionality.
#[derive(Debug, Deserialize, Serialize)]
struct SuggestionStats {
    /// Total number of times this suggestion was made
    times_suggested: u32,
    /// Number of times the user accepted the suggestion
    times_accepted: u32,
    /// Number of times the suggested command succeeded
    times_successful: u32,
    /// Number of times the user rejected or overrode the suggestion
    times_rejected: u32,
    /// Current effectiveness score (0.0 to 1.0)
    effectiveness_score: f32,
    /// When statistics were last updated
    last_updated: DateTime<Utc>,
}

/// Correlation tracking between original and suggested commands.
/// 
/// Measures the effectiveness of specific mapping relationships to enable
/// learning validation and automatic confidence adjustment.
#[derive(Debug, Deserialize, Serialize)]
struct MappingCorrelation {
    /// Original command pattern
    original_pattern: String,
    /// Suggested replacement command
    replacement_command: String,
    /// Success rate when suggestion is accepted
    success_rate: f32,
    /// Total number of executions tracked
    total_executions: u32,
    /// Number of successful executions
    successful_executions: u32,
    /// Confidence adjustment based on correlation data
    confidence_adjustment: f32,
    /// When correlation was last calculated
    last_calculated: DateTime<Utc>,
}

/// Record of a user override or correction.
/// 
/// Tracks when users manually change or reject suggestions to enable
/// automatic learning from corrections and never-suggest functionality.
#[derive(Debug, Deserialize, Serialize)]
struct UserOverride {
    /// The original suggested command
    suggested_command: String,
    /// The command the user actually ran (if detected)
    actual_command: Option<String>,
    /// Type of override (rejection, correction, manual_execution)
    override_type: String,
    /// When the override occurred
    occurred_at: DateTime<Utc>,
    /// Session ID for correlation
    session_id: Option<String>,
    /// Whether this should trigger never-suggest behavior
    should_never_suggest: bool,
}

/// Learned command mappings organized by scope and context.
/// 
/// Provides a hierarchical system for storing learned preferences:
/// - Global: Universal preferences across all projects
/// - Project: Specific to the current project/directory
/// - Context: Conditional mappings based on project type or environment
#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
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
struct PreToolUseInput {
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

/// Input data received from Claude Code UserPromptSubmit hook.
/// 
/// This struct represents the JSON data sent when a user submits a prompt,
/// allowing us to parse natural language for learning signals.
#[derive(Debug, Deserialize)]
struct UserPromptSubmitInput {
    #[allow(dead_code)]
    session_id: String,
    #[allow(dead_code)]
    transcript_path: String,
    #[allow(dead_code)]
    cwd: String,
    #[allow(dead_code)]
    hook_event_name: String,
    prompt: String,
}

/// Input data received from Claude Code PostToolUse hook.
/// 
/// This struct represents the JSON data sent after a tool has been executed,
/// containing information about the tool execution results and metadata.
#[derive(Debug, Deserialize)]
struct PostToolUseInput {
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
    tool_response: ToolResult,
    #[allow(dead_code)]
    tool_use_id: Option<String>,
    #[allow(dead_code)]
    duration_ms: Option<u64>,
}

/// Tool execution result from Claude Code.
/// 
/// Contains the output, error information, and exit status from tool execution.
#[derive(Debug, Deserialize)]
struct ToolResult {
    #[allow(dead_code)]
    output: Option<String>,
    #[allow(dead_code)]
    error: Option<String>,
    exit_code: Option<i32>,
    #[allow(dead_code)]
    truncated: Option<bool>,
}

/// Generic hook input for auto-detection of hook types.
/// 
/// Used to parse the hook_event_name field first, then deserialize
/// into the appropriate specific hook input type.
#[derive(Debug, Deserialize)]
struct GenericHookInput {
    #[allow(dead_code)]
    session_id: String,
    #[allow(dead_code)]
    transcript_path: String,
    #[allow(dead_code)]
    cwd: String,
    hook_event_name: String,
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
        .arg(
            Arg::new("list-learned")
                .long("list-learned")
                .help("List all learned command mappings with statistics")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("reset-learning")
                .long("reset-learning")
                .help("Reset all learned mappings (keeps static configuration)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("confidence-report")
                .long("confidence-report")
                .help("Generate detailed confidence and effectiveness report")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("export-config")
                .long("export-config")
                .value_name("FILE")
                .help("Export learned configuration to file")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("import-config")
                .long("import-config")
                .value_name("FILE")
                .help("Import learned configuration from file")
                .action(clap::ArgAction::Set),
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();
    let replace_mode = matches.get_flag("replace");

    if matches.get_flag("hook") {
        run_as_hook(config_path, replace_mode)
    } else if matches.get_flag("install") {
        run_installer(config_path)
    } else if matches.get_flag("list-learned") {
        list_learned_mappings(config_path)
    } else if matches.get_flag("reset-learning") {
        reset_learning_data(config_path)
    } else if matches.get_flag("confidence-report") {
        generate_confidence_report(config_path)
    } else if let Some(export_file) = matches.get_one::<String>("export-config") {
        export_learned_config(config_path, export_file)
    } else if let Some(import_file) = matches.get_one::<String>("import-config") {
        import_learned_config(config_path, import_file)
    } else {
        println!("Claude Hook Advisor - Intelligent Command Suggestion System");
        println!("============================================================");
        println!();
        println!("Usage:");
        println!("  --hook                 Run as Claude Code hook (reads JSON from stdin)");
        println!("  --install              Set up configuration for this project");
        println!("  --list-learned         List all learned command mappings");
        println!("  --reset-learning       Reset all learned mappings");
        println!("  --confidence-report    Generate detailed effectiveness report");
        println!("  --export-config FILE   Export learned configuration");
        println!("  --import-config FILE   Import learned configuration");
        println!();
        println!("For integration with Claude Code, use --install or --hook");
        Ok(())
    }
}

/// Lists all learned command mappings with their statistics and metadata.
/// 
/// Displays learned mappings organized by scope (global, project, context) along with
/// confidence scores, usage statistics, and effectiveness information.
fn list_learned_mappings(config_path: &str) -> Result<()> {
    let config = load_config(config_path)?;
    
    println!("📚 Learned Command Mappings");
    println!("============================");
    println!();
    
    let mut total_mappings = 0;
    
    if let Some(ref learned) = config.learned {
        // Display global mappings
        if let Some(ref global) = learned.global {
            if !global.is_empty() {
                println!("🌍 Global Mappings:");
                for (original, mapping) in global {
                    print_mapping_info(original, mapping, "global");
                    total_mappings += 1;
                }
                println!();
            }
        }
        
        // Display project mappings
        if let Some(ref project) = learned.project {
            if !project.is_empty() {
                println!("📁 Project Mappings:");
                for (original, mapping) in project {
                    print_mapping_info(original, mapping, "project");
                    total_mappings += 1;
                }
                println!();
            }
        }
        
        // Display context mappings
        if let Some(ref context) = learned.context {
            for (context_name, mappings) in context {
                if !mappings.is_empty() {
                    println!("🏷️  Context Mappings ({context_name}):");
                    for (original, mapping) in mappings {
                        print_mapping_info(original, mapping, context_name);
                        total_mappings += 1;
                    }
                    println!();
                }
            }
        }
    }
    
    // Display summary statistics
    if total_mappings == 0 {
        println!("ℹ️  No learned mappings found. Use natural language like 'use bun instead of npm' to teach the system.");
    } else {
        println!("📊 Summary: {total_mappings} learned mappings");
        
        if let Some(ref meta) = config.learning_meta {
            println!("   Last updated: {}", meta.last_updated.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("   Total learned: {}", meta.total_mappings_learned);
        }
    }
    
    Ok(())
}

/// Prints detailed information about a single mapping.
fn print_mapping_info(original: &str, mapping: &LearnedMapping, scope: &str) {
    println!(
        "  {} → {} (confidence: {:.1}%, learned: {}, from: {})",
        original,
        mapping.replacement,
        mapping.confidence * 100.0,
        mapping.learned_at.format("%Y-%m-%d"),
        mapping.learned_from
    );
    
    if let Some(usage_count) = mapping.usage_count {
        println!("    Usage count: {usage_count} times");
    }
    
    if scope != "global" {
        println!("    Scope: {scope}");
    }
}

/// Resets all learned mapping data while preserving static configuration.
/// 
/// Prompts for confirmation before removing learned mappings, statistics,
/// and execution history. Keeps static command mappings intact.
fn reset_learning_data(config_path: &str) -> Result<()> {
    println!("🔄 Reset Learning Data");
    println!("=====================");
    println!();
    
    let mut config = load_config(config_path)?;
    
    // Count current learned mappings for confirmation
    let learned_count = count_learned_mappings(&config);
    
    if learned_count == 0 {
        println!("ℹ️  No learned data to reset.");
        return Ok(());
    }
    
    println!("⚠️  This will remove:");
    println!("   • {learned_count} learned command mappings");
    println!("   • All execution history and statistics");
    println!("   • All confidence scores and effectiveness data");
    println!();
    println!("Static command mappings from your configuration will be preserved.");
    println!();
    print!("Are you sure you want to reset all learning data? (y/N): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if !input.trim().to_lowercase().starts_with('y') {
        println!("Reset cancelled.");
        return Ok(());
    }
    
    // Reset all learning-related data
    config.learned = Some(LearnedMappings {
        global: Some(HashMap::new()),
        project: Some(HashMap::new()),
        context: Some(HashMap::new()),
    });
    config.execution_history = Some(ExecutionHistory {
        command_executions: Some(Vec::new()),
        suggestion_stats: Some(HashMap::new()),
        mapping_correlations: Some(HashMap::new()),
        user_overrides: Some(Vec::new()),
    });
    config.confidence_scores = Some(HashMap::new());
    config.never_suggest = Some(HashMap::new());
    
    // Update metadata
    if let Some(ref mut meta) = config.learning_meta {
        meta.total_mappings_learned = 0;
        meta.session_mappings = 0;
        meta.user_corrections = 0;
        meta.last_updated = Utc::now();
    }
    
    // Save the reset configuration
    save_config_atomic(config_path, &config)?;
    
    println!("✅ Learning data has been reset successfully.");
    println!("   The system will start learning from scratch while keeping your static configuration.");
    
    Ok(())
}

/// Counts the total number of learned mappings across all scopes.
fn count_learned_mappings(config: &Config) -> usize {
    let mut count = 0;
    
    if let Some(ref learned) = config.learned {
        count += learned.global.as_ref().map(|m| m.len()).unwrap_or(0);
        count += learned.project.as_ref().map(|m| m.len()).unwrap_or(0);
        count += learned.context.as_ref()
            .map(|contexts| contexts.values().map(|m| m.len()).sum())
            .unwrap_or(0);
    }
    
    count
}

/// Generates a detailed confidence and effectiveness report.
/// 
/// Analyzes learning performance, suggestion effectiveness, command execution
/// patterns, and provides insights for system optimization.
fn generate_confidence_report(config_path: &str) -> Result<()> {
    let config = load_config(config_path)?;
    
    println!("📊 Confidence and Effectiveness Report");
    println!("=====================================");
    println!();
    
    // System overview
    print_system_overview(&config);
    println!();
    
    // Mapping confidence analysis
    print_confidence_analysis(&config);
    println!();
    
    // Execution statistics
    print_execution_statistics(&config);
    println!();
    
    // Suggestion effectiveness
    print_suggestion_effectiveness(&config);
    println!();
    
    // Never-suggest analysis
    print_never_suggest_analysis(&config);
    
    Ok(())
}

/// Prints system overview statistics.
fn print_system_overview(config: &Config) {
    println!("🏠 System Overview");
    println!("------------------");
    
    let learned_count = count_learned_mappings(config);
    let static_count = config.commands.len();
    let never_suggest_count = config.never_suggest.as_ref().map(|m| m.len()).unwrap_or(0);
    
    println!("Total mappings: {} (learned: {}, static: {})", 
             learned_count + static_count, learned_count, static_count);
    println!("Never-suggest entries: {never_suggest_count}");
    
    if let Some(ref meta) = config.learning_meta {
        println!("Learning version: {}", meta.version);
        println!("Last updated: {}", meta.last_updated.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("Total learned: {}", meta.total_mappings_learned);
        println!("User corrections: {}", meta.user_corrections);
    }
    
    if let Some(ref history) = config.execution_history {
        let execution_count = history.command_executions.as_ref().map(|e| e.len()).unwrap_or(0);
        let stats_count = history.suggestion_stats.as_ref().map(|s| s.len()).unwrap_or(0);
        println!("Command executions tracked: {execution_count}");
        println!("Suggestion statistics: {stats_count}");
    }
}

/// Prints confidence analysis for learned mappings.
fn print_confidence_analysis(config: &Config) {
    println!("🎯 Confidence Analysis");
    println!("----------------------");
    
    let mut confidence_data = Vec::new();
    
    if let Some(ref learned) = config.learned {
        // Collect confidence scores from all mappings
        if let Some(ref global) = learned.global {
            for (cmd, mapping) in global {
                confidence_data.push((cmd.clone(), mapping.confidence, "global"));
            }
        }
        
        if let Some(ref project) = learned.project {
            for (cmd, mapping) in project {
                confidence_data.push((cmd.clone(), mapping.confidence, "project"));
            }
        }
        
        if let Some(ref context) = learned.context {
            for (context_name, mappings) in context {
                for (cmd, mapping) in mappings {
                    confidence_data.push((cmd.clone(), mapping.confidence, context_name));
                }
            }
        }
    }
    
    if confidence_data.is_empty() {
        println!("No learned mappings to analyze.");
        return;
    }
    
    // Sort by confidence (highest first)
    confidence_data.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    // Calculate statistics
    let _total = confidence_data.len();
    let high_confidence = confidence_data.iter().filter(|(_, conf, _)| *conf >= 0.8).count();
    let medium_confidence = confidence_data.iter().filter(|(_, conf, _)| *conf >= 0.5 && *conf < 0.8).count();
    let low_confidence = confidence_data.iter().filter(|(_, conf, _)| *conf < 0.5).count();
    
    println!("High confidence (≥80%): {high_confidence} mappings");
    println!("Medium confidence (50-79%): {medium_confidence} mappings");
    println!("Low confidence (<50%): {low_confidence} mappings");
    println!();
    
    // Show top 5 and bottom 5
    println!("🏆 Highest Confidence Mappings:");
    for (cmd, conf, scope) in confidence_data.iter().take(5) {
        println!("  {} ({:.1}%, {scope})", cmd, conf * 100.0);
    }
    
    if low_confidence > 0 {
        println!();
        println!("⚠️  Lowest Confidence Mappings:");
        for (cmd, conf, scope) in confidence_data.iter().rev().take(5) {
            println!("  {} ({:.1}%, {scope})", cmd, conf * 100.0);
        }
    }
}

/// Prints execution statistics and patterns.
fn print_execution_statistics(config: &Config) {
    println!("⚡ Execution Statistics");
    println!("----------------------");
    
    if let Some(ref history) = config.execution_history {
        if let Some(ref executions) = history.command_executions {
            if executions.is_empty() {
                println!("No command executions recorded yet.");
                return;
            }
            
            let total_executions = executions.len();
            let successful_executions = executions.iter().filter(|e| e.success).count();
            let suggested_executions = executions.iter().filter(|e| e.was_suggested).count();
            
            println!("Total executions: {total_executions}");
            println!("Successful executions: {} ({:.1}%)", 
                     successful_executions, 
                     (successful_executions as f32 / total_executions as f32) * 100.0);
            println!("Suggested command executions: {} ({:.1}%)",
                     suggested_executions,
                     (suggested_executions as f32 / total_executions as f32) * 100.0);
            
            // Calculate average execution time if available
            let timed_executions: Vec<_> = executions.iter()
                .filter_map(|e| e.duration_ms)
                .collect();
            
            if !timed_executions.is_empty() {
                let avg_duration = timed_executions.iter().sum::<u64>() as f32 / timed_executions.len() as f32;
                println!("Average execution time: {avg_duration:.1}ms");
            }
        }
    } else {
        println!("No execution history available.");
    }
}

/// Prints suggestion effectiveness analysis.
fn print_suggestion_effectiveness(config: &Config) {
    println!("🎯 Suggestion Effectiveness");
    println!("---------------------------");
    
    if let Some(ref history) = config.execution_history {
        if let Some(ref stats) = history.suggestion_stats {
            if stats.is_empty() {
                println!("No suggestion statistics available yet.");
                return;
            }
            
            let mut effectiveness_data: Vec<_> = stats.iter().collect();
            effectiveness_data.sort_by(|a, b| b.1.effectiveness_score.partial_cmp(&a.1.effectiveness_score).unwrap());
            
            let total_suggestions = stats.len();
            let highly_effective = stats.values().filter(|s| s.effectiveness_score >= 0.8).count();
            let moderately_effective = stats.values().filter(|s| s.effectiveness_score >= 0.5 && s.effectiveness_score < 0.8).count();
            let low_effective = stats.values().filter(|s| s.effectiveness_score < 0.5).count();
            
            println!("Total suggestion types: {total_suggestions}");
            println!("Highly effective (≥80%): {highly_effective} suggestions");
            println!("Moderately effective (50-79%): {moderately_effective} suggestions");
            println!("Low effectiveness (<50%): {low_effective} suggestions");
            println!();
            
            println!("🏆 Most Effective Suggestions:");
            for (mapping, stats) in effectiveness_data.iter().take(5) {
                println!("  {} (success: {:.1}%, attempts: {})", 
                         mapping, 
                         stats.effectiveness_score * 100.0,
                         stats.times_accepted);
            }
            
            if low_effective > 0 {
                println!();
                println!("⚠️  Least Effective Suggestions:");
                for (mapping, stats) in effectiveness_data.iter().rev().take(5) {
                    if stats.effectiveness_score < 0.8 {
                        println!("  {} (success: {:.1}%, attempts: {})", 
                                 mapping, 
                                 stats.effectiveness_score * 100.0,
                                 stats.times_accepted);
                    }
                }
            }
        }
    } else {
        println!("No suggestion effectiveness data available.");
    }
}

/// Prints never-suggest analysis.
fn print_never_suggest_analysis(config: &Config) {
    println!("🚫 Never-Suggest Analysis");
    println!("-------------------------");
    
    if let Some(ref never_suggest) = config.never_suggest {
        if never_suggest.is_empty() {
            println!("No never-suggest entries. This indicates suggestions are generally effective.");
        } else {
            println!("Never-suggest entries: {}", never_suggest.len());
            println!();
            println!("Blocked suggestions:");
            for (original, replacement) in never_suggest {
                println!("  {original} → {replacement} (automatically blocked due to poor performance)");
            }
        }
    } else {
        println!("Never-suggest functionality not initialized.");
    }
}

/// Exports learned configuration to a file.
/// 
/// Creates a portable configuration file containing only learned mappings,
/// statistics, and metadata that can be imported into other instances.
fn export_learned_config(config_path: &str, export_file: &str) -> Result<()> {
    let config = load_config(config_path)?;
    
    println!("📤 Exporting Learned Configuration");
    println!("==================================");
    
    // Create export structure with only learned data
    let export_config = Config {
        commands: HashMap::new(), // Don't export static commands
        learned: config.learned.clone(),
        learning_meta: config.learning_meta.clone(),
        confidence_scores: config.confidence_scores.clone(),
        never_suggest: config.never_suggest.clone(),
        execution_history: None, // Don't export execution history for privacy
    };
    
    let learned_count = count_learned_mappings(&export_config);
    
    if learned_count == 0 {
        println!("⚠️  No learned mappings to export.");
        return Ok(());
    }
    
    // Generate export content
    let export_content = toml::to_string_pretty(&export_config)
        .context("Failed to serialize export configuration")?;
    
    let header = format!(
        "# Claude Hook Advisor - Exported Learned Configuration\n\
         # Exported on: {}\n\
         # Contains {} learned mappings\n\
         # \n\
         # This file contains only learned preferences and can be imported\n\
         # into other claude-hook-advisor instances using --import-config\n\n",
        Utc::now().to_rfc3339(),
        learned_count
    );
    
    let full_content = format!("{header}{export_content}");
    
    // Write export file
    fs::write(export_file, &full_content)
        .with_context(|| format!("Failed to write export file: {export_file}"))?;
    
    println!("✅ Successfully exported {learned_count} learned mappings to: {export_file}");
    
    if let Some(ref meta) = export_config.learning_meta {
        println!("   Export includes {} total historical mappings", meta.total_mappings_learned);
    }
    
    Ok(())
}

/// Imports learned configuration from a file.
/// 
/// Merges learned mappings from an export file into the current configuration,
/// resolving conflicts by preferring existing mappings with higher confidence.
fn import_learned_config(config_path: &str, import_file: &str) -> Result<()> {
    println!("📥 Importing Learned Configuration");
    println!("==================================");
    
    // Load import file
    let import_content = fs::read_to_string(import_file)
        .with_context(|| format!("Failed to read import file: {import_file}"))?;
    
    let import_config: Config = toml::from_str(&import_content)
        .with_context(|| format!("Failed to parse import file: {import_file}"))?;
    
    let import_count = count_learned_mappings(&import_config);
    
    if import_count == 0 {
        println!("⚠️  No learned mappings found in import file.");
        return Ok(());
    }
    
    println!("Found {import_count} learned mappings in import file.");
    
    // Load current configuration
    let mut config = load_config(config_path)?;
    let original_count = count_learned_mappings(&config);
    
    // Merge configurations
    let merged_count = merge_learned_mappings(&mut config, &import_config)?;
    
    // Save merged configuration
    save_config_atomic(config_path, &config)?;
    
    let final_count = count_learned_mappings(&config);
    
    println!("✅ Import completed successfully!");
    println!("   Original mappings: {original_count}");
    println!("   Imported mappings: {import_count}");
    println!("   New mappings added: {merged_count}");
    println!("   Total mappings now: {final_count}");
    
    Ok(())
}

/// Merges learned mappings from import config into target config.
/// 
/// Returns the number of new mappings that were added.
fn merge_learned_mappings(target: &mut Config, source: &Config) -> Result<u32> {
    let mut added_count = 0;
    
    // Initialize target learned mappings if needed
    if target.learned.is_none() {
        target.learned = Some(LearnedMappings {
            global: Some(HashMap::new()),
            project: Some(HashMap::new()),
            context: Some(HashMap::new()),
        });
    }
    
    let target_learned = target.learned.as_mut().unwrap();
    
    if let Some(ref source_learned) = source.learned {
        // Merge global mappings
        if let Some(ref source_global) = source_learned.global {
            let target_global = target_learned.global.get_or_insert_with(HashMap::new);
            for (cmd, mapping) in source_global {
                if !target_global.contains_key(cmd) {
                    target_global.insert(cmd.clone(), mapping.clone());
                    added_count += 1;
                    eprintln!("📥 Added global mapping: {} → {}", cmd, mapping.replacement);
                }
            }
        }
        
        // Merge project mappings
        if let Some(ref source_project) = source_learned.project {
            let target_project = target_learned.project.get_or_insert_with(HashMap::new);
            for (cmd, mapping) in source_project {
                if !target_project.contains_key(cmd) {
                    target_project.insert(cmd.clone(), mapping.clone());
                    added_count += 1;
                    eprintln!("📥 Added project mapping: {} → {}", cmd, mapping.replacement);
                }
            }
        }
        
        // Merge context mappings
        if let Some(ref source_context) = source_learned.context {
            let target_context = target_learned.context.get_or_insert_with(HashMap::new);
            for (context_name, source_mappings) in source_context {
                let target_mappings = target_context.entry(context_name.clone()).or_default();
                for (cmd, mapping) in source_mappings {
                    if !target_mappings.contains_key(cmd) {
                        target_mappings.insert(cmd.clone(), mapping.clone());
                        added_count += 1;
                        eprintln!("📥 Added context mapping ({}): {} → {}", context_name, cmd, mapping.replacement);
                    }
                }
            }
        }
    }
    
    // Update metadata
    if let Some(ref mut target_meta) = target.learning_meta {
        target_meta.total_mappings_learned += added_count;
        target_meta.last_updated = Utc::now();
    }
    
    Ok(added_count)
}

/// Runs the application as a Claude Code hook with auto-detection of hook type.
/// 
/// Reads JSON input from stdin, detects the hook type based on hook_event_name,
/// and routes to the appropriate handler. Supports PreToolUse and UserPromptSubmit hooks.
/// 
/// # Arguments
/// * `config_path` - Path to the .claude-hook-advisor.toml configuration file
/// * `replace_mode` - If true, returns "replace" decision; if false, returns "block"
/// 
/// # Returns
/// * `Ok(())` - Hook processing completed successfully
/// * Process exits with JSON output if command should be blocked/replaced (PreToolUse only)
fn run_as_hook(config_path: &str, replace_mode: bool) -> Result<()> {
    // Read JSON input from stdin
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    // First parse to detect hook type
    let generic_input: GenericHookInput =
        serde_json::from_str(&buffer).context("Failed to parse hook input JSON")?;

    // Route to appropriate handler based on hook type
    match generic_input.hook_event_name.as_str() {
        "PreToolUse" => {
            let hook_input: PreToolUseInput = serde_json::from_str(&buffer)
                .context("Failed to parse PreToolUse input JSON")?;
            handle_pre_tool_use(hook_input, config_path, replace_mode)
        }
        "UserPromptSubmit" => {
            let hook_input: UserPromptSubmitInput = serde_json::from_str(&buffer)
                .context("Failed to parse UserPromptSubmit input JSON")?;
            handle_user_prompt_submit(hook_input, config_path)
        }
        "PostToolUse" => {
            let hook_input: PostToolUseInput = serde_json::from_str(&buffer)
                .context("Failed to parse PostToolUse input JSON")?;
            handle_post_tool_use(hook_input, config_path)
        }
        _ => {
            eprintln!("Warning: Unknown hook event type: {}", generic_input.hook_event_name);
            Ok(())
        }
    }
}

/// Handles PreToolUse hook events (formerly the main run_as_hook logic).
/// 
/// Loads configuration and checks if Bash commands should be blocked or replaced.
/// Only processes Bash commands; other tool types are ignored.
fn handle_pre_tool_use(hook_input: PreToolUseInput, config_path: &str, replace_mode: bool) -> Result<()> {
    // Read configuration
    let config = load_config(config_path)?;

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

/// Handles UserPromptSubmit hook events for natural language learning.
/// 
/// Parses user prompts for learning signals like "use X instead of Y" and
/// updates the configuration silently. Always allows the prompt to proceed.
fn handle_user_prompt_submit(hook_input: UserPromptSubmitInput, config_path: &str) -> Result<()> {
    // Create natural language parser
    let parser = match NaturalLanguageParser::new() {
        Ok(parser) => parser,
        Err(e) => {
            eprintln!("Warning: Failed to create natural language parser: {e}");
            return Ok(());
        }
    };
    
    // Extract mappings from the user prompt
    let mappings = parser.extract_mappings(&hook_input.prompt);
    
    if !mappings.is_empty() {
        // Load current configuration
        let mut config = load_config(config_path)?;
        
        // Apply each extracted mapping
        for mapping in mappings {
            // Add the learned mapping to the configuration
            if let Err(e) = add_learned_mapping(
                &mut config,
                &mapping.original,
                &mapping.replacement,
                &mapping.scope,
                mapping.confidence,
                &mapping.source,
            ) {
                eprintln!("Warning: Failed to add learned mapping: {e}");
                continue;
            }
            
            // Log the learning event for debugging
            eprintln!(
                "🧠 Learned: {} → {} (scope: {}, confidence: {:.1}%)",
                mapping.original,
                mapping.replacement,
                mapping.scope,
                mapping.confidence * 100.0
            );
        }
        
        // Save updated configuration atomically
        if let Err(e) = save_config_atomic(config_path, &config) {
            eprintln!("Warning: Failed to save learned mappings: {e}");
        }
    }
    
    // Always allow prompt to proceed normally
    Ok(())
}

/// Handles PostToolUse hook events for execution result validation and learning.
/// 
/// Analyzes tool execution results to update suggestion effectiveness, adjust
/// confidence scores, and detect user overrides. Only processes Bash commands.
fn handle_post_tool_use(hook_input: PostToolUseInput, config_path: &str) -> Result<()> {
    // Only process Bash commands
    if hook_input.tool_name != "Bash" {
        return Ok(());
    }

    // Load current configuration
    let mut config = load_config(config_path)?;
    
    // Create command execution record
    // Note: PostToolUse hooks only fire for successful tool completions in Claude Code
    // If exit_code is None, we assume success since the hook was triggered
    let actual_exit_code = hook_input.tool_response.exit_code.unwrap_or(0);
    let execution = CommandExecution {
        command: hook_input.tool_input.command.clone(),
        was_suggested: false, // Will be determined by correlation analysis
        original_command: None, // Will be populated if this was a suggestion
        exit_code: Some(actual_exit_code),
        success: actual_exit_code == 0,
        duration_ms: hook_input.duration_ms,
        executed_at: Utc::now(),
        suggestion_source: None, // Will be determined by correlation analysis
        session_id: Some(hook_input.session_id.clone()),
    };
    
    // Analyze execution and update tracking data
    if let Err(e) = analyze_and_update_execution(&mut config, execution) {
        eprintln!("Warning: Failed to analyze command execution: {e}");
    }
    
    // Save updated configuration atomically
    if let Err(e) = save_config_atomic(config_path, &config) {
        eprintln!("Warning: Failed to save execution tracking data: {e}");
    }
    
    // Always allow execution to complete normally (PostToolUse is observational)
    Ok(())
}

/// Analyzes command execution and updates tracking data for learning validation.
/// 
/// Determines if the executed command was suggested by the system, correlates it with
/// previous suggestions, updates effectiveness statistics, and adjusts confidence scores.
/// 
/// # Arguments
/// * `config` - Configuration to update (modified in place)
/// * `execution` - Command execution record to analyze
/// 
/// # Returns
/// * `Ok(())` - Analysis completed successfully
/// * `Err` - If analysis or updates fail
fn analyze_and_update_execution(config: &mut Config, mut execution: CommandExecution) -> Result<()> {
    // Initialize execution history if not present
    if config.execution_history.is_none() {
        config.execution_history = Some(ExecutionHistory {
            command_executions: Some(Vec::new()),
            suggestion_stats: Some(HashMap::new()),
            mapping_correlations: Some(HashMap::new()),
            user_overrides: Some(Vec::new()),
        });
    }
    
    // Determine if this command was suggested by analyzing it against known mappings
    // (do this before borrowing config mutably)
    let suggestion_info = determine_suggestion_correlation(config, &execution.command)?;
    let suggestion_info_clone = suggestion_info.clone();
    
    let history = config.execution_history.as_mut().unwrap();
    
    // Update execution record with suggestion information
    if let Some((original_cmd, source, replacement)) = suggestion_info {
        execution.was_suggested = true;
        execution.original_command = Some(original_cmd.clone());
        execution.suggestion_source = Some(source.clone());
        
        // Update suggestion statistics
        update_suggestion_stats(history, &original_cmd, &replacement, execution.success);
        
        // Update mapping correlations
        update_mapping_correlation(history, &original_cmd, &replacement, execution.success);
        
        // Log the correlation for debugging
        let status = if execution.success { "✅" } else { "❌" };
        eprintln!(
            "{} Command correlation: {} → {} (source: {}, exit_code: {:?})",
            status,
            original_cmd,
            replacement,
            source,
            execution.exit_code
        );
    } else {
        // Check for potential user overrides by comparing against recent suggestions
        detect_user_override(history, &execution);
    }
    
    // Add execution to history
    if let Some(ref mut executions) = history.command_executions {
        executions.push(execution.clone());
        
        // Keep only recent executions to prevent unbounded growth
        const MAX_EXECUTION_HISTORY: usize = 1000;
        if executions.len() > MAX_EXECUTION_HISTORY {
            executions.drain(0..executions.len() - MAX_EXECUTION_HISTORY);
        }
    }
    
    // Adjust confidence based on execution results (separate from history operations)
    if let Some((original_cmd, source, replacement)) = suggestion_info_clone {
        if let Err(e) = adjust_mapping_confidence(config, &original_cmd, &replacement, &source, execution.success) {
            eprintln!("Warning: Failed to adjust mapping confidence: {e}");
        }
    }
    
    // Perform periodic maintenance (confidence decay, never_suggest evaluation)
    // Only run this occasionally to avoid performance impact
    if should_run_maintenance(config) {
        if let Err(e) = perform_periodic_maintenance(config) {
            eprintln!("Warning: Failed to perform periodic maintenance: {e}");
        }
    }
    
    Ok(())
}

/// Determines if periodic maintenance should be run based on time and activity.
/// 
/// Runs maintenance at most once per day and only after significant activity.
fn should_run_maintenance(config: &Config) -> bool {
    const MAINTENANCE_INTERVAL_DAYS: i64 = 1;
    const MIN_EXECUTIONS_FOR_MAINTENANCE: usize = 10;
    
    // Check if enough time has passed since last maintenance
    let last_updated = config.learning_meta
        .as_ref()
        .map(|meta| meta.last_updated)
        .unwrap_or_else(Utc::now);
    
    let days_since_maintenance = (Utc::now() - last_updated).num_days();
    if days_since_maintenance < MAINTENANCE_INTERVAL_DAYS {
        return false;
    }
    
    // Check if there's been enough activity to warrant maintenance
    let execution_count = config.execution_history
        .as_ref()
        .and_then(|history| history.command_executions.as_ref())
        .map(|executions| executions.len())
        .unwrap_or(0);
    
    execution_count >= MIN_EXECUTIONS_FOR_MAINTENANCE
}

/// Determines if a command matches any known suggestion patterns.
/// 
/// Returns the original command, source, and replacement if a correlation is found.
fn determine_suggestion_correlation(
    config: &Config,
    executed_command: &str,
) -> Result<Option<(String, String, String)>> {
    // Check against all learned mappings to see if this execution matches a suggestion
    if let Some(learned) = &config.learned {
        // Check project mappings first (highest priority)
        if let Some(project_mappings) = &learned.project {
            if let Some(correlation) = find_command_correlation(project_mappings, executed_command)? {
                return Ok(Some((correlation.0, "project_learned".to_string(), correlation.1)));
            }
        }
        
        // Check global mappings
        if let Some(global_mappings) = &learned.global {
            if let Some(correlation) = find_command_correlation(global_mappings, executed_command)? {
                return Ok(Some((correlation.0, "global_learned".to_string(), correlation.1)));
            }
        }
        
        // Check context mappings
        if let Some(context_mappings) = &learned.context {
            for (context_name, mappings) in context_mappings {
                if let Some(correlation) = find_command_correlation(mappings, executed_command)? {
                    return Ok(Some((correlation.0, format!("context_{context_name}"), correlation.1)));
                }
            }
        }
    }
    
    // Check against static mappings
    if let Some(correlation) = find_command_correlation_static(&config.commands, executed_command)? {
        return Ok(Some((correlation.0, "static".to_string(), correlation.1)));
    }
    
    Ok(None)
}

/// Finds correlation between executed command and learned mappings.
fn find_command_correlation(
    mappings: &HashMap<String, LearnedMapping>,
    executed_command: &str,
) -> Result<Option<(String, String)>> {
    for (original_pattern, learned_mapping) in mappings {
        // Check if the executed command could be the result of applying this mapping
        let _regex_pattern = format!(r"\b{}\b", regex::escape(original_pattern));
        
        // If the executed command contains the replacement, it might be a suggestion result
        if executed_command.contains(&learned_mapping.replacement) {
            // Verify by checking if applying the mapping to some original would produce this result
            if let Some(reconstructed_original) = reconstruct_original_command(
                executed_command,
                original_pattern,
                &learned_mapping.replacement,
            ) {
                return Ok(Some((reconstructed_original, learned_mapping.replacement.clone())));
            }
        }
    }
    Ok(None)
}

/// Finds correlation between executed command and static mappings.
fn find_command_correlation_static(
    commands: &HashMap<String, String>,
    executed_command: &str,
) -> Result<Option<(String, String)>> {
    for (original_pattern, replacement) in commands {
        if executed_command.contains(replacement) {
            if let Some(reconstructed_original) = reconstruct_original_command(
                executed_command,
                original_pattern,
                replacement,
            ) {
                return Ok(Some((reconstructed_original, replacement.clone())));
            }
        }
    }
    Ok(None)
}

/// Attempts to reconstruct the original command that would produce the executed command.
fn reconstruct_original_command(
    executed_command: &str,
    original_pattern: &str,
    replacement: &str,
) -> Option<String> {
    // Simple reconstruction: replace the replacement back with the original pattern
    if executed_command.contains(replacement) {
        Some(executed_command.replace(replacement, original_pattern))
    } else {
        None
    }
}

/// Updates suggestion statistics for a command mapping.
fn update_suggestion_stats(
    history: &mut ExecutionHistory,
    original_command: &str,
    replacement_command: &str,
    success: bool,
) {
    let stats_key = format!("{original_command}→{replacement_command}");
    
    if history.suggestion_stats.is_none() {
        history.suggestion_stats = Some(HashMap::new());
    }
    
    let stats_map = history.suggestion_stats.as_mut().unwrap();
    let stats = stats_map.entry(stats_key).or_insert_with(|| SuggestionStats {
        times_suggested: 0,
        times_accepted: 1, // This execution represents an acceptance
        times_successful: if success { 1 } else { 0 },
        times_rejected: 0,
        effectiveness_score: if success { 1.0 } else { 0.0 },
        last_updated: Utc::now(),
    });
    
    stats.times_accepted += 1;
    if success {
        stats.times_successful += 1;
    }
    
    // Recalculate effectiveness score
    stats.effectiveness_score = if stats.times_accepted > 0 {
        stats.times_successful as f32 / stats.times_accepted as f32
    } else {
        0.0
    };
    
    stats.last_updated = Utc::now();
}

/// Updates mapping correlation data for learning validation.
fn update_mapping_correlation(
    history: &mut ExecutionHistory,
    original_pattern: &str,
    replacement_command: &str,
    success: bool,
) {
    if history.mapping_correlations.is_none() {
        history.mapping_correlations = Some(HashMap::new());
    }
    
    let correlations = history.mapping_correlations.as_mut().unwrap();
    let correlation = correlations.entry(original_pattern.to_string()).or_insert_with(|| {
        MappingCorrelation {
            original_pattern: original_pattern.to_string(),
            replacement_command: replacement_command.to_string(),
            success_rate: if success { 1.0 } else { 0.0 },
            total_executions: 1,
            successful_executions: if success { 1 } else { 0 },
            confidence_adjustment: 0.0,
            last_calculated: Utc::now(),
        }
    });
    
    correlation.total_executions += 1;
    if success {
        correlation.successful_executions += 1;
    }
    
    // Recalculate success rate and confidence adjustment
    correlation.success_rate = correlation.successful_executions as f32 / correlation.total_executions as f32;
    
    // Calculate confidence adjustment based on success rate and sample size
    let sample_weight = (correlation.total_executions as f32 / 10.0).min(1.0);
    correlation.confidence_adjustment = (correlation.success_rate - 0.7) * sample_weight * 0.1;
    
    correlation.last_calculated = Utc::now();
}

/// Adjusts mapping confidence based on execution results.
fn adjust_mapping_confidence(
    config: &mut Config,
    original_command: &str,
    replacement_command: &str,
    source: &str,
    success: bool,
) -> Result<()> {
    // Determine confidence adjustment based on success/failure
    let adjustment = if success { 0.05 } else { -0.10 }; // Success increases, failure decreases
    
    // Apply adjustment to the appropriate mapping
    match source {
        "global_learned" => {
            if let Some(ref mut learned) = config.learned {
                if let Some(ref mut global) = learned.global {
                    if let Some(mapping) = global.get_mut(original_command) {
                        mapping.confidence = (mapping.confidence + adjustment).clamp(0.0, 1.0);
                        eprintln!(
                            "📊 Confidence adjusted: {} → {} = {:.1}% ({})",
                            original_command,
                            replacement_command,
                            mapping.confidence * 100.0,
                            if success { "success" } else { "failure" }
                        );
                    }
                }
            }
        }
        "project_learned" => {
            if let Some(ref mut learned) = config.learned {
                if let Some(ref mut project) = learned.project {
                    if let Some(mapping) = project.get_mut(original_command) {
                        mapping.confidence = (mapping.confidence + adjustment).clamp(0.0, 1.0);
                        eprintln!(
                            "📊 Project confidence adjusted: {} → {} = {:.1}% ({})",
                            original_command,
                            replacement_command,
                            mapping.confidence * 100.0,
                            if success { "success" } else { "failure" }
                        );
                    }
                }
            }
        }
        source if source.starts_with("context_") => {
            let context_name = source.strip_prefix("context_").unwrap_or(source);
            if let Some(ref mut learned) = config.learned {
                if let Some(ref mut context) = learned.context {
                    if let Some(mappings) = context.get_mut(context_name) {
                        if let Some(mapping) = mappings.get_mut(original_command) {
                            mapping.confidence = (mapping.confidence + adjustment).clamp(0.0, 1.0);
                            eprintln!(
                                "📊 Context confidence adjusted: {} → {} = {:.1}% (context: {}, {})",
                                original_command,
                                replacement_command,
                                mapping.confidence * 100.0,
                                context_name,
                                if success { "success" } else { "failure" }
                            );
                        }
                    }
                }
            }
        }
        _ => {
            // Static mappings don't have adjustable confidence
            eprintln!(
                "📊 Static mapping executed: {} → {} ({})",
                original_command,
                replacement_command,
                if success { "success" } else { "failure" }
            );
        }
    }
    
    Ok(())
}

/// Detects potential user overrides by analyzing execution patterns.
fn detect_user_override(_history: &mut ExecutionHistory, execution: &CommandExecution) {
    // For now, just log unmatched executions for future analysis
    // In a more sophisticated implementation, we could compare against recent suggestions
    eprintln!(
        "🔍 Unmatched execution: {} (exit_code: {:?})",
        execution.command,
        execution.exit_code
    );
    
    // TODO: Implement sophisticated override detection by comparing against
    // recent PreToolUse suggestions that were blocked or rejected
}

/// Applies time-based confidence decay to prevent stale mappings from persisting.
/// 
/// Reduces confidence scores for mappings that haven't been used recently,
/// ensuring the system adapts to changing user preferences over time.
fn apply_confidence_decay(config: &mut Config) -> Result<()> {
    const DECAY_RATE: f32 = 0.02; // 2% decay per week
    const WEEKS_PER_DAY: f32 = 1.0 / 7.0;
    
    let now = Utc::now();
    
    if let Some(ref mut learned) = config.learned {
        // Apply decay to global mappings
        if let Some(ref mut global) = learned.global {
            for mapping in global.values_mut() {
                let days_since_learned = (now - mapping.learned_at).num_days() as f32;
                let decay_factor = (DECAY_RATE * days_since_learned * WEEKS_PER_DAY).min(0.3); // Max 30% decay
                mapping.confidence = (mapping.confidence - decay_factor).max(0.1); // Min 10% confidence
            }
        }
        
        // Apply decay to project mappings
        if let Some(ref mut project) = learned.project {
            for mapping in project.values_mut() {
                let days_since_learned = (now - mapping.learned_at).num_days() as f32;
                let decay_factor = (DECAY_RATE * days_since_learned * WEEKS_PER_DAY).min(0.3);
                mapping.confidence = (mapping.confidence - decay_factor).max(0.1);
            }
        }
        
        // Apply decay to context mappings
        if let Some(ref mut context) = learned.context {
            for mappings in context.values_mut() {
                for mapping in mappings.values_mut() {
                    let days_since_learned = (now - mapping.learned_at).num_days() as f32;
                    let decay_factor = (DECAY_RATE * days_since_learned * WEEKS_PER_DAY).min(0.3);
                    mapping.confidence = (mapping.confidence - decay_factor).max(0.1);
                }
            }
        }
    }
    
    Ok(())
}

/// Evaluates mapping effectiveness and moves low-performing suggestions to never_suggest.
/// 
/// Analyzes suggestion statistics to identify consistently failing mappings and
/// automatically adds them to the never_suggest list to prevent future suggestions.
fn evaluate_and_update_never_suggest(config: &mut Config) -> Result<()> {
    const MIN_ATTEMPTS: u32 = 5; // Minimum attempts before considering never_suggest
    const FAILURE_THRESHOLD: f32 = 0.3; // Below 30% success rate triggers never_suggest
    
    if let Some(ref history) = config.execution_history {
        if let Some(ref stats) = history.suggestion_stats {
            let mut never_suggest_candidates = Vec::new();
            
            for (mapping_key, suggestion_stats) in stats {
                // Only consider mappings with enough attempts
                if suggestion_stats.times_accepted >= MIN_ATTEMPTS {
                    // Check if effectiveness is below threshold
                    if suggestion_stats.effectiveness_score < FAILURE_THRESHOLD {
                        // Parse the mapping key (format: "original→replacement")
                        if let Some((original, replacement)) = mapping_key.split_once('→') {
                            never_suggest_candidates.push((original.to_string(), replacement.to_string()));
                            
                            eprintln!(
                                "🚫 Moving to never-suggest: {} → {} (effectiveness: {:.1}%, attempts: {})",
                                original,
                                replacement,
                                suggestion_stats.effectiveness_score * 100.0,
                                suggestion_stats.times_accepted
                            );
                        }
                    }
                }
            }
            
            // Add candidates to never_suggest
            if !never_suggest_candidates.is_empty() {
                if config.never_suggest.is_none() {
                    config.never_suggest = Some(HashMap::new());
                }
                
                // First, add to never_suggest
                {
                    let never_suggest = config.never_suggest.as_mut().unwrap();
                    for (original, replacement) in &never_suggest_candidates {
                        never_suggest.insert(original.clone(), replacement.clone());
                    }
                }
                
                // Then remove from learned mappings to prevent conflicts
                for (original, _) in never_suggest_candidates {
                    remove_from_learned_mappings(config, &original);
                }
            }
        }
    }
    
    Ok(())
}

/// Removes a mapping from all learned mapping categories.
/// 
/// Used when a mapping is moved to never_suggest to prevent conflicts.
fn remove_from_learned_mappings(config: &mut Config, original_command: &str) {
    if let Some(ref mut learned) = config.learned {
        // Remove from global mappings
        if let Some(ref mut global) = learned.global {
            global.remove(original_command);
        }
        
        // Remove from project mappings
        if let Some(ref mut project) = learned.project {
            project.remove(original_command);
        }
        
        // Remove from context mappings
        if let Some(ref mut context) = learned.context {
            for mappings in context.values_mut() {
                mappings.remove(original_command);
            }
        }
    }
}

/// Performs periodic maintenance on the configuration data.
/// 
/// Applies confidence decay, evaluates never_suggest candidates, and cleans up
/// old data to keep the system performant and accurate.
fn perform_periodic_maintenance(config: &mut Config) -> Result<()> {
    // Apply time-based confidence decay
    apply_confidence_decay(config)?;
    
    // Evaluate and update never_suggest based on performance
    evaluate_and_update_never_suggest(config)?;
    
    // Update learning metadata
    if let Some(ref mut meta) = config.learning_meta {
        meta.last_updated = Utc::now();
    }
    
    Ok(())
}

/// Saves configuration to file atomically using temp file + rename.
/// 
/// Creates a temporary file in the same directory, writes the configuration,
/// and then atomically renames it to the target path. This prevents corruption
/// from concurrent access or interrupted writes.
/// 
/// # Arguments
/// * `config_path` - Path where the configuration should be saved
/// * `config` - Configuration to save
/// 
/// # Returns
/// * `Ok(())` - Configuration saved successfully
/// * `Err` - If file operations fail or serialization fails
fn save_config_atomic(config_path: &str, config: &Config) -> Result<()> {
    let config_path = Path::new(config_path);
    let parent_dir = config_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    
    // Create temporary file in the same directory as the target
    let temp_file = NamedTempFile::new_in(parent_dir)
        .with_context(|| format!("Failed to create temporary file in directory: {parent_dir:?}"))?;
    
    // Generate configuration content with header
    let header = generate_config_header();
    let toml_content = toml::to_string_pretty(config)
        .context("Failed to serialize configuration to TOML")?;
    let full_content = format!("{header}\n{toml_content}");
    
    // Write to temporary file
    fs::write(temp_file.path(), &full_content)
        .context("Failed to write configuration to temporary file")?;
    
    // Atomically move temporary file to final location
    temp_file.persist(config_path)
        .with_context(|| format!("Failed to persist configuration file: {}", config_path.display()))?;
    
    Ok(())
}

/// Generates a header comment for configuration files.
/// 
/// Includes timestamp and warnings about learned sections being auto-managed.
fn generate_config_header() -> String {
    let timestamp = Utc::now().to_rfc3339();
    format!(
        "# Claude Hook Advisor Configuration\n\
         # Auto-updated by learning system\n\
         # Last updated: {timestamp}\n\
         # \n\
         # NOTE: The [learned] sections are managed automatically.\n\
         # You can safely edit [commands] but avoid manually editing learned mappings."
    )
}

/// Updates a configuration by adding a learned mapping.
/// 
/// Adds a new learned mapping to the appropriate section (global, project, or context)
/// and updates the learning metadata. Handles initialization of empty sections.
/// 
/// # Arguments
/// * `config` - Configuration to update (modified in place)
/// * `original` - Original command that should be replaced
/// * `replacement` - Replacement command to suggest
/// * `scope` - Where to store this mapping (global, project, context)
/// * `confidence` - Confidence score for this mapping (0.0 to 1.0)
/// * `source` - How this mapping was learned
/// 
/// # Returns
/// * `Ok(())` - Mapping added successfully
/// * `Err` - If the update fails
fn add_learned_mapping(
    config: &mut Config,
    original: &str,
    replacement: &str,
    scope: &str,
    confidence: f32,
    source: &str,
) -> Result<()> {
    // Initialize learned mappings if not present
    if config.learned.is_none() {
        config.learned = Some(LearnedMappings {
            global: Some(HashMap::new()),
            project: Some(HashMap::new()),
            context: Some(HashMap::new()),
        });
    }
    
    let learned = config.learned.as_mut().unwrap();
    let learned_mapping = LearnedMapping {
        replacement: replacement.to_string(),
        confidence,
        learned_at: Utc::now(),
        learned_from: source.to_string(),
        usage_count: Some(1),
        context: if scope != "global" { Some(scope.to_string()) } else { None },
    };
    
    // Add to appropriate scope
    match scope {
        "global" => {
            if learned.global.is_none() {
                learned.global = Some(HashMap::new());
            }
            learned.global.as_mut().unwrap().insert(original.to_string(), learned_mapping);
        }
        "project" => {
            if learned.project.is_none() {
                learned.project = Some(HashMap::new());
            }
            learned.project.as_mut().unwrap().insert(original.to_string(), learned_mapping);
        }
        scope if scope.starts_with("context:") => {
            // Context-specific mapping
            if learned.context.is_none() {
                learned.context = Some(HashMap::new());
            }
            let context_name = scope.strip_prefix("context:").unwrap_or(scope);
            learned.context.as_mut().unwrap()
                .entry(context_name.to_string())
                .or_default()
                .insert(original.to_string(), learned_mapping);
        }
        _ => {
            // Default to global for unknown scopes
            eprintln!("Warning: Unknown scope '{scope}', defaulting to global");
            if learned.global.is_none() {
                learned.global = Some(HashMap::new());
            }
            learned.global.as_mut().unwrap().insert(original.to_string(), learned_mapping);
        }
    }
    
    // Update learning metadata
    if let Some(ref mut meta) = config.learning_meta {
        meta.total_mappings_learned += 1;
        meta.session_mappings += 1;
        meta.last_updated = Utc::now();
    }
    
    Ok(())
}

/// Natural language parser for extracting command preferences from user text.
/// 
/// Uses regex patterns to identify learning signals in natural language like
/// "use X instead of Y", "I prefer X", "for this project use X", etc.
struct NaturalLanguageParser {
    patterns: Vec<LearningPattern>,
}

/// A single learning pattern with regex and extraction logic.
/// 
/// Each pattern can recognize a specific way users express command preferences
/// and extract the relevant command mapping with appropriate confidence.
struct LearningPattern {
    name: String,
    regex: Regex,
    confidence: f32,
    scope: String, // "global", "project", or "context"
}

/// Extracted command mapping from natural language.
/// 
/// Contains all the information needed to update the configuration
/// with a new learned preference.
#[derive(Debug, Clone)]
struct ExtractedMapping {
    original: String,
    replacement: String,
    scope: String,
    confidence: f32,
    source: String,
    #[allow(dead_code)]
    context: Option<String>,
}

impl NaturalLanguageParser {
    /// Creates a new parser with predefined learning patterns.
    fn new() -> Result<Self> {
        let patterns = vec![
            // Most specific patterns first (to avoid conflicts)
            
            // Always use instead: "always use X instead of Y"
            LearningPattern {
                name: "always_use_instead".to_string(),
                regex: Regex::new(r"(?i)\balways\s+use\s+([a-zA-Z][a-zA-Z0-9_-]*)\s+instead\s+of\s+([a-zA-Z][a-zA-Z0-9_-]*)\b")?,
                confidence: 0.95,
                scope: "global".to_string(),
            },
            
            // Always use: "always use X for Y"
            LearningPattern {
                name: "always_use_for".to_string(),
                regex: Regex::new(r"(?i)\balways\s+use\s+([a-zA-Z][a-zA-Z0-9_-]*)\s+for\s+([a-zA-Z][a-zA-Z0-9_-]*)\b")?,
                confidence: 0.95,
                scope: "global".to_string(),
            },
            
            // Project-specific with instead: "for this project, use X instead of Y"
            LearningPattern {
                name: "project_replacement".to_string(),
                regex: Regex::new(r"(?i)for\s+(?:this|the)\s+project,?\s+(?:let's |please )?use\s+([a-zA-Z][a-zA-Z0-9_-]*)\s+instead\s+of\s+([a-zA-Z][a-zA-Z0-9_-]*)")?,
                confidence: 0.92,
                scope: "project".to_string(),
            },
            
            // Project-specific: "for this project, use X"
            LearningPattern {
                name: "project_specific".to_string(),
                regex: Regex::new(r"(?i)for\s+(?:this|the)\s+project,?\s+(?:let's |please )?use\s+([a-zA-Z][a-zA-Z0-9_-]*)")?,
                confidence: 0.88,
                scope: "project".to_string(),
            },
            
            // Context-specific: "for React projects, use X"
            LearningPattern {
                name: "context_specific".to_string(),
                regex: Regex::new(r"(?i)for\s+([a-zA-Z][a-zA-Z0-9_]*)\s+projects?,?\s+(?:let's |please )?use\s+([a-zA-Z][a-zA-Z0-9_-]*)")?,
                confidence: 0.86,
                scope: "context".to_string(),
            },
            
            // Direct replacement: "use X instead of Y"
            LearningPattern {
                name: "direct_replacement".to_string(),
                regex: Regex::new(r"(?i)\b(?:let's |please |can we )?use\s+([a-zA-Z][a-zA-Z0-9_-]*)\s+instead\s+of\s+([a-zA-Z][a-zA-Z0-9_-]*)")?,
                confidence: 0.90,
                scope: "global".to_string(),
            },
            
            // Preference: "I prefer X over Y" or "I prefer X to Y"
            LearningPattern {
                name: "preference_statement".to_string(),
                regex: Regex::new(r"(?i)\bi\s+prefer\s+([a-zA-Z][a-zA-Z0-9_-]*)\s+(?:over|to)\s+([a-zA-Z][a-zA-Z0-9_-]*)")?,
                confidence: 0.85,
                scope: "global".to_string(),
            },
            
            // Simple replacement: "let's use X"
            LearningPattern {
                name: "simple_replacement".to_string(),
                regex: Regex::new(r"(?i)let's\s+use\s+([a-zA-Z][a-zA-Z0-9_-]*)")?,
                confidence: 0.70,
                scope: "global".to_string(),
            },
        ];
        
        Ok(Self { patterns })
    }
    
    /// Extracts command mappings from a text string.
    /// 
    /// Applies all patterns to the input text and returns any extracted mappings
    /// with their confidence scores and context information.
    fn extract_mappings(&self, text: &str) -> Vec<ExtractedMapping> {
        let mut mappings = Vec::new();
        let mut used_spans = Vec::new();
        
        for pattern in &self.patterns {
            for captures in pattern.regex.captures_iter(text) {
                let match_span = captures.get(0).unwrap();
                let start = match_span.start();
                let end = match_span.end();
                
                // Check if this match overlaps with any existing match
                let overlaps = used_spans.iter().any(|(used_start, used_end)| {
                    start < *used_end && end > *used_start
                });
                
                if !overlaps {
                    if let Some(mapping) = self.extract_mapping_from_captures(pattern, &captures) {
                        mappings.push(mapping);
                        used_spans.push((start, end));
                    }
                }
            }
        }
        
        mappings
    }
    
    /// Extracts a mapping from regex captures based on the pattern type.
    fn extract_mapping_from_captures(
        &self,
        pattern: &LearningPattern,
        captures: &regex::Captures,
    ) -> Option<ExtractedMapping> {
        match pattern.name.as_str() {
            "direct_replacement" | "preference_statement" | "project_replacement" | "always_use_for" | "always_use_instead" => {
                // Patterns with both original and replacement
                let replacement = captures.get(1)?.as_str().to_string();
                let original = captures.get(2)?.as_str().to_string();
                
                Some(ExtractedMapping {
                    original,
                    replacement,
                    scope: pattern.scope.clone(),
                    confidence: pattern.confidence,
                    source: "natural_language".to_string(),
                    context: None,
                })
            }
            "project_specific" => {
                // Pattern with only replacement - need to infer original from context
                let replacement = captures.get(1)?.as_str().to_string();
                let original = self.infer_original_from_replacement(&replacement)?;
                
                Some(ExtractedMapping {
                    original,
                    replacement,
                    scope: pattern.scope.clone(),
                    confidence: pattern.confidence,
                    source: "natural_language".to_string(),
                    context: Some("project_preference".to_string()),
                })
            }
            "context_specific" => {
                // Pattern with context and replacement
                let context = captures.get(1)?.as_str().to_string();
                let replacement = captures.get(2)?.as_str().to_string();
                let original = self.infer_original_from_replacement(&replacement)?;
                
                Some(ExtractedMapping {
                    original,
                    replacement,
                    scope: format!("context:{}", context.to_lowercase()),
                    confidence: pattern.confidence,
                    source: "natural_language".to_string(),
                    context: Some(context),
                })
            }
            "simple_replacement" => {
                // Simple "let's use X" - need more context to determine what to replace
                // For now, skip these unless we have more context
                None
            }
            _ => None,
        }
    }
    
    /// Infers what command should be replaced based on the replacement command.
    /// 
    /// Uses common knowledge about tool alternatives to guess the original command.
    fn infer_original_from_replacement(&self, replacement: &str) -> Option<String> {
        match replacement {
            "bun" => Some("npm".to_string()),
            "yarn" => Some("npm".to_string()),
            "pnpm" => Some("npm".to_string()),
            "bunx" => Some("npx".to_string()),
            "rg" => Some("grep".to_string()),
            "fd" => Some("find".to_string()),
            "bat" => Some("cat".to_string()),
            "eza" | "exa" => Some("ls".to_string()),
            "podman" => Some("docker".to_string()),
            "uv" => Some("pip".to_string()),
            _ => None,
        }
    }
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
            version: "0.3.0".to_string(),
        }),
        confidence_scores: None,
        never_suggest: None,
        execution_history: None,
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
            version: "0.3.0".to_string(),
        }),
        confidence_scores: Some(HashMap::new()),
        never_suggest: Some(HashMap::new()),
        execution_history: Some(ExecutionHistory {
            command_executions: Some(Vec::new()),
            suggestion_stats: Some(HashMap::new()),
            mapping_correlations: Some(HashMap::new()),
            user_overrides: Some(Vec::new()),
        }),
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
    
    if config.execution_history.is_none() {
        config.execution_history = Some(ExecutionHistory {
            command_executions: Some(Vec::new()),
            suggestion_stats: Some(HashMap::new()),
            mapping_correlations: Some(HashMap::new()),
            user_overrides: Some(Vec::new()),
        });
    }

    // Update version if it's outdated (future migration logic can go here)
    if let Some(ref mut meta) = config.learning_meta {
        if meta.version != "0.3.0" {
            meta.version = "0.3.0".to_string();
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
                "Tool preference: Use '{replacement}' instead of '{pattern}' for this project. Running: {suggested_command}"
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
            version: "0.3.0".to_string(),
        }),
        confidence_scores: None,
        never_suggest: None,
        execution_history: None,
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
            "command": "{} --hook",
            "timeout": 30
          }}
        ]
      }}
    ],
    "UserPromptSubmit": [
      {{
        "hooks": [
          {{
            "type": "command",
            "command": "{} --hook",
            "timeout": 10
          }}
        ]
      }}
    ],
    "PostToolUse": [
      {{
        "matcher": "Bash",
        "hooks": [
          {{
            "type": "command",
            "command": "{} --hook",
            "timeout": 15
          }}
        ]
      }}
    ]
  }}
}}"#;

    print!(
        r#"{HEADER}
  4. Add hook command: `{binary_path} --hook`
  5. Also add `UserPromptSubmit` and `PostToolUse` hooks with the same command
  6. Save to project settings

Option 2: Manual .claude/settings.json configuration
Add this to your .claude/settings.json (enables complete learning system):

{json_config}

Note: The triple-hook setup enables:
- PreToolUse: Blocks/suggests commands based on learned preferences
- UserPromptSubmit: Learns new preferences from natural language like "use bun instead of npm"
- PostToolUse: Validates suggestion effectiveness and adjusts confidence scores automatically

Additional CLI Commands:
- `{binary_path} --list-learned`: View all learned command mappings
- `{binary_path} --confidence-report`: Generate detailed effectiveness analysis
- `{binary_path} --reset-learning`: Clear all learned data (keeps static config)
- `{binary_path} --export-config file.toml`: Export learned preferences
- `{binary_path} --import-config file.toml`: Import learned preferences

"#,
        binary_path = binary_path,
        json_config = JSON_TEMPLATE.replacen("{}", &binary_path, 3)
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
            execution_history: None,
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
                version: "0.3.0".to_string(),
            }),
            confidence_scores: Some(HashMap::new()),
            never_suggest: Some(HashMap::new()),
            execution_history: None,
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
            execution_history: None,
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
            execution_history: None,
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

    #[test]
    fn test_natural_language_parser_creation() {
        let parser = NaturalLanguageParser::new();
        assert!(parser.is_ok());
        let parser = parser.unwrap();
        assert!(!parser.patterns.is_empty());
    }

    #[test]
    fn test_direct_replacement_pattern() {
        let parser = NaturalLanguageParser::new().unwrap();

        // Test "use X instead of Y" pattern
        let mappings = parser.extract_mappings("use bun instead of npm");
        assert_eq!(mappings.len(), 1);
        let mapping = &mappings[0];
        assert_eq!(mapping.original, "npm");
        assert_eq!(mapping.replacement, "bun");
        assert_eq!(mapping.scope, "global");
        assert_eq!(mapping.confidence, 0.90);

        // Test with variations
        let mappings = parser.extract_mappings("let's use yarn instead of npm");
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].original, "npm");
        assert_eq!(mappings[0].replacement, "yarn");

        // Test case insensitive
        let mappings = parser.extract_mappings("Use RG instead of GREP");
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].original, "GREP");
        assert_eq!(mappings[0].replacement, "RG");
    }

    #[test]
    fn test_preference_pattern() {
        let parser = NaturalLanguageParser::new().unwrap();

        // Test "I prefer X over Y" 
        let mappings = parser.extract_mappings("I prefer bun over npm");
        assert_eq!(mappings.len(), 1);
        let mapping = &mappings[0];
        assert_eq!(mapping.original, "npm");
        assert_eq!(mapping.replacement, "bun");
        assert_eq!(mapping.scope, "global");
        assert_eq!(mapping.confidence, 0.85);

        // Test "I prefer X to Y"
        let mappings = parser.extract_mappings("I prefer ripgrep to grep");
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].original, "grep");
        assert_eq!(mappings[0].replacement, "ripgrep");
    }

    #[test]
    fn test_project_specific_patterns() {
        let parser = NaturalLanguageParser::new().unwrap();

        // Test "for this project, use X"
        let mappings = parser.extract_mappings("for this project, use bun");
        assert_eq!(mappings.len(), 1);
        let mapping = &mappings[0];
        assert_eq!(mapping.original, "npm"); // Inferred from bun
        assert_eq!(mapping.replacement, "bun");
        assert_eq!(mapping.scope, "project");
        assert_eq!(mapping.confidence, 0.88);

        // Test "for this project, use X instead of Y"
        let mappings = parser.extract_mappings("for this project, use yarn instead of npm");
        assert_eq!(mappings.len(), 1);
        let mapping = &mappings[0];
        assert_eq!(mapping.original, "npm");
        assert_eq!(mapping.replacement, "yarn");
        assert_eq!(mapping.scope, "project");
        assert_eq!(mapping.confidence, 0.92);
    }

    #[test]
    fn test_context_specific_pattern() {
        let parser = NaturalLanguageParser::new().unwrap();

        // Test "for React projects, use X"
        let mappings = parser.extract_mappings("for React projects, use yarn");
        assert_eq!(mappings.len(), 1);
        let mapping = &mappings[0];
        assert_eq!(mapping.original, "npm"); // Inferred from yarn
        assert_eq!(mapping.replacement, "yarn");
        assert_eq!(mapping.scope, "context:react");
        assert_eq!(mapping.confidence, 0.86);
        assert_eq!(mapping.context, Some("React".to_string()));
    }

    #[test]
    fn test_always_use_pattern() {
        let parser = NaturalLanguageParser::new().unwrap();

        // Test "always use X for Y"
        let mappings = parser.extract_mappings("always use rg for grep");
        assert_eq!(mappings.len(), 1);
        let mapping = &mappings[0];
        assert_eq!(mapping.original, "grep");
        assert_eq!(mapping.replacement, "rg");
        assert_eq!(mapping.scope, "global");
        assert_eq!(mapping.confidence, 0.95);

        // Test "always use X instead of Y"
        let mappings = parser.extract_mappings("always use bun instead of npm");
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].original, "npm");
        assert_eq!(mappings[0].replacement, "bun");
    }

    #[test]
    fn test_original_inference() {
        let parser = NaturalLanguageParser::new().unwrap();

        // Test various tool replacements
        assert_eq!(parser.infer_original_from_replacement("bun"), Some("npm".to_string()));
        assert_eq!(parser.infer_original_from_replacement("yarn"), Some("npm".to_string()));
        assert_eq!(parser.infer_original_from_replacement("bunx"), Some("npx".to_string()));
        assert_eq!(parser.infer_original_from_replacement("rg"), Some("grep".to_string()));
        assert_eq!(parser.infer_original_from_replacement("fd"), Some("find".to_string()));
        assert_eq!(parser.infer_original_from_replacement("bat"), Some("cat".to_string()));
        assert_eq!(parser.infer_original_from_replacement("eza"), Some("ls".to_string()));
        assert_eq!(parser.infer_original_from_replacement("podman"), Some("docker".to_string()));
        assert_eq!(parser.infer_original_from_replacement("uv"), Some("pip".to_string()));
        
        // Unknown replacement should return None
        assert_eq!(parser.infer_original_from_replacement("unknown"), None);
    }

    #[test]
    fn test_multiple_patterns_in_text() {
        let parser = NaturalLanguageParser::new().unwrap();

        // Text with multiple learning signals
        let text = "I prefer bun over npm and for this project, use yarn instead of pnpm";
        let mappings = parser.extract_mappings(text);
        
        // Should extract multiple mappings
        assert_eq!(mappings.len(), 2);
        
        // First mapping: "I prefer bun over npm"
        let preference_mapping = mappings.iter().find(|m| m.replacement == "bun").unwrap();
        assert_eq!(preference_mapping.original, "npm");
        assert_eq!(preference_mapping.scope, "global");
        
        // Second mapping: "for this project, use yarn instead of pnpm"
        let project_mapping = mappings.iter().find(|m| m.replacement == "yarn").unwrap();
        assert_eq!(project_mapping.original, "pnpm");
        assert_eq!(project_mapping.scope, "project");
    }

    #[test]
    fn test_no_false_positives() {
        let parser = NaturalLanguageParser::new().unwrap();

        // Text that shouldn't trigger any patterns
        let non_matching_texts = vec![
            "I want to install some packages",
            "Let's run the tests",
            "The npm command is running",
            "We should use better tools",
            "This project needs yarn",
        ];

        for text in non_matching_texts {
            let mappings = parser.extract_mappings(text);
            assert!(mappings.is_empty(), "Unexpected mapping found for: '{}'", text);
        }
    }

    #[test]
    fn test_add_learned_mapping() {
        let mut config = create_empty_config();

        // Add a global mapping
        add_learned_mapping(&mut config, "npm", "bun", "global", 0.9, "test").unwrap();

        // Verify the mapping was added
        let learned = config.learned.as_ref().unwrap();
        let global_mappings = learned.global.as_ref().unwrap();
        assert!(global_mappings.contains_key("npm"));
        
        let mapping = global_mappings.get("npm").unwrap();
        assert_eq!(mapping.replacement, "bun");
        assert_eq!(mapping.confidence, 0.9);
        assert_eq!(mapping.learned_from, "test");

        // Verify metadata was updated
        let meta = config.learning_meta.as_ref().unwrap();
        assert_eq!(meta.total_mappings_learned, 1);
        assert_eq!(meta.session_mappings, 1);
    }

    #[test]
    fn test_atomic_config_save() {
        use tempfile::tempdir;
        
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.toml");
        let config_path_str = config_path.to_str().unwrap();

        let mut config = create_empty_config();
        add_learned_mapping(&mut config, "npm", "bun", "global", 0.9, "test").unwrap();

        // Save the configuration
        save_config_atomic(config_path_str, &config).unwrap();

        // Verify the file was created
        assert!(config_path.exists());

        // Load the configuration back
        let loaded_config = load_config(config_path_str).unwrap();
        
        // Verify the learned mapping was preserved
        let learned = loaded_config.learned.as_ref().unwrap();
        let global_mappings = learned.global.as_ref().unwrap();
        assert!(global_mappings.contains_key("npm"));
        
        let mapping = global_mappings.get("npm").unwrap();
        assert_eq!(mapping.replacement, "bun");
        assert_eq!(mapping.confidence, 0.9);
    }
    
    #[test]
    fn test_end_to_end_learning_workflow() {
        use tempfile::tempdir;
        
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.toml");
        let config_path_str = config_path.to_str().unwrap();

        // Step 1: Start with empty configuration
        let initial_config = create_empty_config();
        save_config_atomic(config_path_str, &initial_config).unwrap();

        // Step 2: Simulate UserPromptSubmit hook with learning signal
        let user_prompt_input = UserPromptSubmitInput {
            session_id: "test_session".to_string(),
            transcript_path: "/tmp/test".to_string(),
            cwd: "/tmp".to_string(),
            hook_event_name: "UserPromptSubmit".to_string(),
            prompt: "use bun instead of npm".to_string(),
        };
        
        // Process the user prompt (this should learn the mapping)
        handle_user_prompt_submit(user_prompt_input, config_path_str).unwrap();

        // Step 3: Simulate PreToolUse hook with npm command (structure defined for completeness)
        let _pre_tool_input = PreToolUseInput {
            session_id: "test_session".to_string(),
            transcript_path: "/tmp/test".to_string(),
            cwd: "/tmp".to_string(),
            hook_event_name: "PreToolUse".to_string(),
            tool_name: "Bash".to_string(),
            tool_input: ToolInput {
                command: "npm install".to_string(),
                description: None,
            },
        };

        // Load the updated config to verify learning occurred
        let updated_config = load_config(config_path_str).unwrap();
        
        // Verify the learned mapping was added
        let learned = updated_config.learned.as_ref().unwrap();
        let global_mappings = learned.global.as_ref().unwrap();
        assert!(global_mappings.contains_key("npm"));
        
        let mapping = global_mappings.get("npm").unwrap();
        assert_eq!(mapping.replacement, "bun");
        assert_eq!(mapping.confidence, 0.9); // From direct_replacement pattern
        assert_eq!(mapping.learned_from, "natural_language");

        // Step 4: Verify PreToolUse hook now suggests the learned command
        let result = check_command_mappings(&updated_config, "npm install").unwrap();
        assert!(result.is_some());
        let (suggestion, replacement) = result.unwrap();
        assert!(suggestion.contains("bun"));
        assert_eq!(replacement, "bun install");
    }
    
    #[test]
    fn test_project_specific_learning_workflow() {
        use tempfile::tempdir;
        
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-config.toml");
        let config_path_str = config_path.to_str().unwrap();

        // Start with empty configuration
        let initial_config = create_empty_config();
        save_config_atomic(config_path_str, &initial_config).unwrap();

        // Simulate project-specific learning: "for this project, use yarn instead of npm"
        let user_prompt_input = UserPromptSubmitInput {
            session_id: "test_session".to_string(),
            transcript_path: "/tmp/test".to_string(),
            cwd: "/tmp".to_string(),
            hook_event_name: "UserPromptSubmit".to_string(),
            prompt: "for this project, use yarn instead of npm".to_string(),
        };
        
        handle_user_prompt_submit(user_prompt_input, config_path_str).unwrap();

        // Load and verify the project-specific mapping was added
        let updated_config = load_config(config_path_str).unwrap();
        let learned = updated_config.learned.as_ref().unwrap();
        let project_mappings = learned.project.as_ref().unwrap();
        assert!(project_mappings.contains_key("npm"));
        
        let mapping = project_mappings.get("npm").unwrap();
        assert_eq!(mapping.replacement, "yarn");
        assert_eq!(mapping.confidence, 0.92); // From project_replacement pattern

        // Verify PreToolUse hook suggests the project-specific command
        let result = check_command_mappings(&updated_config, "npm start").unwrap();
        assert!(result.is_some());
        let (suggestion, replacement) = result.unwrap();
        assert!(suggestion.contains("yarn"));
        assert_eq!(replacement, "yarn start");
    }
}
