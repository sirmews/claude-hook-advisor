//! Type definitions for Claude Hook Advisor

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration structure for command mappings and directory aliasing.
///
/// Loaded from .claude-hook-advisor.toml files, this struct contains
/// the mapping from original commands to their preferred replacements
/// and semantic directory aliases for natural language references.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub commands: HashMap<String, String>,
    #[serde(default)]
    pub semantic_directories: HashMap<String, String>,
    #[serde(default)]
    pub command_history: Option<CommandHistoryConfig>,
    #[serde(default)]
    pub security_pattern_overrides: HashMap<String, bool>,
}

/// Configuration for command history tracking
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommandHistoryConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_history_path")]
    pub log_file: String,
}

/// Security pattern for detecting risky code patterns in file edits.
///
/// Patterns can match based on file path patterns (glob-style) or content substrings.
/// When a match is found, the specified reminder is shown to Claude.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecurityPattern {
    pub rule_name: String,
    #[serde(default)]
    pub path_pattern: Option<String>,
    #[serde(default)]
    pub content_substrings: Vec<String>,
    pub reminder: String,
}

fn default_true() -> bool {
    true
}

fn default_history_path() -> String {
    "~/.claude-hook-advisor/bash-history.db".to_string()
}

/// Input data received from Claude Code hook system.
/// 
/// This struct represents the JSON data sent from different hook events,
/// containing information about the tool being invoked and its parameters.
#[derive(Debug, Deserialize, Serialize)]
pub struct HookInput {
    #[allow(dead_code)]
    pub session_id: String,
    #[allow(dead_code)]
    pub transcript_path: Option<String>,
    #[allow(dead_code)]
    pub cwd: Option<String>,
    pub hook_event_name: String,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<ToolInput>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub tool_response: Option<ToolResponse>,
}

/// Tool response data from PostToolUse hooks.
/// 
/// Contains execution results and status information for tracking
/// command success rates and confidence adjustment.
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolResponse {
    #[allow(dead_code)]
    pub exit_code: Option<i32>,
    #[allow(dead_code)]
    pub stdout: Option<String>,
    #[allow(dead_code)]
    pub stderr: Option<String>,
}

/// Tool-specific input parameters from Claude Code.
///
/// Contains the actual command and optional description for Bash tool invocations,
/// and file editing parameters for Edit/Write/MultiEdit tools.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct ToolInput {
    // Bash tool parameters
    #[serde(default)]
    pub command: Option<String>,
    #[allow(dead_code)]
    pub description: Option<String>,

    // File editing tool parameters
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub content: Option<String>,      // Write tool
    #[serde(default)]
    pub old_string: Option<String>,   // Edit tool
    #[serde(default)]
    pub new_string: Option<String>,   // Edit tool
    #[serde(default)]
    pub edits: Option<Vec<EditOperation>>,  // MultiEdit tool
}

/// Single edit operation for MultiEdit tool
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EditOperation {
    pub old_string: String,
    pub new_string: String,
}

/// Response data sent back to Claude Code hook system.
///
/// This struct represents the JSON response that tells Claude Code whether
/// to block the command and provides suggestions or replacements.
#[derive(Debug, Serialize)]
pub struct HookOutput {
    pub decision: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_command: Option<String>,
}

/// Modern API response for PreToolUse hooks with new format
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookSpecificOutput {
    pub hook_event_name: String,
    pub permission_decision: String,
    pub permission_decision_reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<UpdatedInput>,
}

/// Updated input data for modified tool calls
#[derive(Debug, Serialize)]
pub struct UpdatedInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

/// Modern API response wrapper for PreToolUse hooks
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModernHookResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_specific_output: Option<HookSpecificOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#continue: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}



/// Helper functions for creating modern API responses
impl ModernHookResponse {
    /// Create a deny response with replacement command (maintains existing behavior)
    pub fn deny_with_replacement(decision_reason: String, replacement_command: String) -> Self {
        ModernHookResponse {
            hook_specific_output: Some(HookSpecificOutput {
                hook_event_name: "PreToolUse".to_string(),
                permission_decision: "deny".to_string(),
                permission_decision_reason: decision_reason,
                updated_input: Some(UpdatedInput {
                    command: Some(replacement_command),
                }),
            }),
            r#continue: None,
            stop_reason: None,
        }
    }

    /// Create an allow response (for compatibility)
    pub fn allow() -> Self {
        ModernHookResponse {
            hook_specific_output: Some(HookSpecificOutput {
                hook_event_name: "PreToolUse".to_string(),
                permission_decision: "allow".to_string(),
                permission_decision_reason: "Command allowed".to_string(),
                updated_input: None,
            }),
            r#continue: None,
            stop_reason: None,
        }
    }
    
    /// Create the correct JSON output manually for Claude Code API
    pub fn to_correct_json(&self) -> Result<String, serde_json::Error> {
        let mut output = serde_json::Map::new();
        
        if let Some(hook_output) = &self.hook_specific_output {
            let mut hook_specific = serde_json::Map::new();
            hook_specific.insert("hookEventName".to_string(), 
                serde_json::Value::String(hook_output.hook_event_name.clone()));
            hook_specific.insert("permissionDecision".to_string(), 
                serde_json::Value::String(hook_output.permission_decision.clone()));
            hook_specific.insert("permissionDecisionReason".to_string(), 
                serde_json::Value::String(hook_output.permission_decision_reason.clone()));
            
            if let Some(updated) = &hook_output.updated_input {
                let mut updated_input = serde_json::Map::new();
                if let Some(cmd) = &updated.command {
                    updated_input.insert("command".to_string(), 
                        serde_json::Value::String(cmd.clone()));
                }
                hook_specific.insert("updatedInput".to_string(), 
                    serde_json::Value::Object(updated_input));
            }
            
            output.insert("hookSpecificOutput".to_string(), 
                serde_json::Value::Object(hook_specific));
        }
        
        if let Some(cont) = self.r#continue {
            output.insert("continue".to_string(), 
                serde_json::Value::Bool(cont));
        }
        
        if let Some(reason) = &self.stop_reason {
            output.insert("stopReason".to_string(), 
                serde_json::Value::String(reason.clone()));
        }
        
        serde_json::to_string(&serde_json::Value::Object(output))
    }
}

/// Result of directory resolution operation.
/// 
/// Contains the canonical path and metadata about the resolution process
/// for semantic directory references.
#[derive(Debug, Clone)]
pub struct DirectoryResolution {
    pub canonical_path: String,
    pub alias_used: String,
    pub variables_substituted: Vec<(String, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_input_deserialization() {
        let json = r#"{
            "session_id": "test-session",
            "transcript_path": "/path/to/transcript",
            "cwd": "/current/directory",
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {
                "command": "npm install",
                "description": "Install packages"
            }
        }"#;
        
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.session_id, "test-session");
        assert_eq!(input.tool_name.unwrap(), "Bash");
        assert_eq!(input.tool_input.unwrap().command.unwrap(), "npm install");
    }

    #[test]
    fn test_hook_input_minimal() {
        // Test with minimal required fields
        let json = r#"{
            "session_id": "test",
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {
                "command": "ls -la"
            }
        }"#;
        
        let input: HookInput = serde_json::from_str(json).unwrap();
        let tool_input = input.tool_input.unwrap();
        assert_eq!(tool_input.command.unwrap(), "ls -la");
        assert!(tool_input.description.is_none());
    }

    #[test]
    fn test_user_prompt_submit_hook() {
        // Test UserPromptSubmit hook input
        let json = r#"{
            "session_id": "test",
            "hook_event_name": "UserPromptSubmit",
            "prompt": "use bun instead of npm"
        }"#;
        
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.hook_event_name, "UserPromptSubmit");
        assert_eq!(input.prompt.unwrap(), "use bun instead of npm");
        assert!(input.tool_name.is_none());
    }

    #[test]
    fn test_post_tool_use_hook() {
        // Test PostToolUse hook input
        let json = r#"{
            "session_id": "test",
            "hook_event_name": "PostToolUse",
            "tool_name": "Bash",
            "tool_input": {
                "command": "bun install"
            },
            "tool_response": {
                "exit_code": 0,
                "stdout": "Dependencies installed",
                "stderr": ""
            }
        }"#;
        
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.hook_event_name, "PostToolUse");
        assert_eq!(input.tool_name.unwrap(), "Bash");
        assert_eq!(input.tool_response.unwrap().exit_code.unwrap(), 0);
    }
}