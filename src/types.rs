//! Type definitions for Claude Hook Advisor

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration structure for command mappings.
/// 
/// Loaded from .claude-hook-advisor.toml files, this struct contains
/// the mapping from original commands to their preferred replacements.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub commands: HashMap<String, String>,
}

/// Input data received from Claude Code hook system.
/// 
/// This struct represents the JSON data sent to PreToolUse hooks,
/// containing information about the tool being invoked and its parameters.
#[derive(Debug, Deserialize)]
pub struct HookInput {
    #[allow(dead_code)]
    pub session_id: String,
    #[allow(dead_code)]
    pub transcript_path: String,
    #[allow(dead_code)]
    pub cwd: String,
    #[allow(dead_code)]
    pub hook_event_name: String,
    pub tool_name: String,
    pub tool_input: ToolInput,
}

/// Tool-specific input parameters from Claude Code.
/// 
/// Contains the actual command and optional description for Bash tool invocations.
#[derive(Debug, Deserialize)]
pub struct ToolInput {
    pub command: String,
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
        assert_eq!(input.tool_name, "Bash");
        assert_eq!(input.tool_input.command, "npm install");
        assert_eq!(input.tool_input.description.unwrap(), "Install packages");
    }

    #[test]
    fn test_hook_input_minimal() {
        // Test with minimal required fields
        let json = r#"{
            "session_id": "test",
            "transcript_path": "",
            "cwd": "",
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {
                "command": "ls -la"
            }
        }"#;
        
        let input: HookInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.tool_input.command, "ls -la");
        assert!(input.tool_input.description.is_none());
    }
}