//! Claude Hook Advisor
//! 
//! A Rust CLI tool that integrates with Claude Code hooks to provide intelligent
//! command suggestions and semantic directory aliasing.

// Public API - main functions and essential types for external users
pub use cli::run_cli;
pub use directory::resolve_directory;
pub use types::{DirectoryResolution, Config};

// Modules needed by internal binary and tests
pub mod cli;
pub mod types;

// Private implementation modules
mod config;
mod hooks;
mod installer;
mod directory;
pub mod history;