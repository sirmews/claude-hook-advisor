# AGENTS.md - Development Guidelines

## Build, Test, and Lint Commands

- **Build**: `cargo build` (debug) or `cargo build --release` (production)
- **Test**: `cargo test -- --test-threads=1` (single-threaded to avoid race conditions)
- **Run single test**: `cargo test test_name -- --test-threads=1`
- **Lint**: `cargo clippy -- -D warnings`
- **Format**: `cargo fmt`
- **Check**: `cargo check` (verify without building)
- **Install**: `cargo install --path .` (globally) or `make install-local` (~/.local/bin)

## Architecture & Structure

**Purpose**: Rust CLI tool for Claude Code integration providing command suggestions, directory aliasing, command history tracking, and security pattern detection.

**Key Subprojects**:
- **src/main.rs**: Binary entrypoint (thin wrapper calling `cli::run_cli`)
- **src/lib.rs**: Public API exports (run_cli, resolve_directory, DirectoryResolution, Config)
- **src/cli.rs**: CLI argument parsing and main dispatch logic
- **src/hooks.rs**: Hook event processing (PreToolUse, UserPromptSubmit, PostToolUse)
- **src/config.rs**: TOML configuration loading and parsing
- **src/types.rs**: Core types (Config, DirectoryResolution, HookInput, etc.)
- **src/directory.rs**: Semantic directory alias resolution with path canonicalization
- **src/security.rs**: 27 built-in security pattern definitions for detecting dangerous code
- **src/history.rs**: SQLite command history logging and querying
- **src/installer.rs**: Hook installation/uninstallation to .claude/settings.json

**Config Files**: `.claude-hook-advisor.toml` (user config) with sections: [commands], [semantic_directories], [command_history], [security_pattern_overrides]

**Database**: SQLite at `~/.claude-hook-advisor/bash-history.db` (command history logs)

## Code Style Guidelines

- **Language**: Rust edition 2021
- **Error Handling**: Use `anyhow::Result<T>` and `.context()` for error propagation; avoid panics
- **Imports**: Group by standard library, external crates, then internal modules; use explicit imports
- **Documentation**: Add `//!` module-level doc comments and `///` for public items
- **Naming**: snake_case for functions/variables, PascalCase for types; descriptive names (e.g., `resolve_directory`, `security_pattern_overrides`)
- **Functions**: Keep functions focused; return `Result` types for fallible operations
- **Tests**: Place integration tests in src/ with `#[cfg(test)]` modules; use `tempfile` crate for test files
- **Formatting**: Run `cargo fmt` (automatic); clippy rules enforced with `-D warnings`
- **Configuration**: Use serde for JSON/TOML serialization; preserve TOML field order with `preserve_order` feature
- **Security**: Always canonicalize paths to prevent traversal attacks; validate all user inputs
