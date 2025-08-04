//! Claude Hook Advisor
//! 
//! A Rust CLI tool that integrates with Claude Code as a PreToolUse hook

pub mod types;
pub mod config;
pub mod hooks;
pub mod installer;
pub mod patterns;
pub mod cli;

// Re-exports for clean public API
pub use types::{Config, HookInput, HookOutput, ToolInput};
pub use cli::run_cli;