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
/// Contains the actual command and optional description for Bash tool invocations.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct ToolInput {
    #[serde(default)]
    pub command: Option<String>,
    #[allow(dead_code)]
    pub description: Option<String>,
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