# Claude Hook Advisor

A Rust CLI tool that integrates with Claude Code using hooks to provide intelligent command suggestions, semantic directory aliasing, command history tracking, and security pattern detection.

## Project Overview

**Purpose:** Enhances Claude Code's capabilities by intercepting commands to enforce project-specific tool preferences (e.g., using `bun` instead of `npm`), allowing natural language directory navigation, and providing security checks.

**Key Features:**
*   **Command Interception:** Maps commands to preferred alternatives via `PreToolUse` hook.
*   **Semantic Directory Aliasing:** Resolves natural language directory names (e.g., "docs") to actual paths via `UserPromptSubmit` hook.
*   **Command History:** Tracks executed commands and their status in a SQLite database via `PostToolUse` hook.
*   **Security Patterns:** Scans file edits for dangerous patterns (e.g., `eval()`, SQL injection) and warns the user.

**Architecture:**
*   **Language:** Rust
*   **Entry Point:** `src/main.rs` (CLI argument parsing and dispatch).
*   **Core Logic:** `src/lib.rs` (Hook handling, configuration loading).
*   **Hooks:**
    *   `PreToolUse`: For command mapping and security checks.
    *   `UserPromptSubmit`: For directory aliasing.
    *   `PostToolUse`: For history tracking.

## Building and Running

The project uses `Cargo` for Rust dependency management and building, and a `Makefile` for convenience.

### Prerequisites
*   Rust toolchain (`cargo`, `rustc`)
*   `make`
*   `cog` (for release management, optional for dev)

### Key Commands

*   **Build (Debug):**
    ```bash
    cargo build
    # or
    make build
    ```

*   **Build (Release):**
    ```bash
    cargo build --release
    # or
    make build-release
    ```

*   **Run Tests:**
    *Note: Tests are run single-threaded to avoid race conditions with temporary file operations.*
    ```bash
    cargo test -- --test-threads=1
    # or
    make test
    ```

*   **Install (Local):**
    ```bash
    cargo install --path .
    # or
    make install
    ```

*   **Simulate Hook Execution:**
    You can test the hook logic by piping JSON into the binary:
    ```bash
    # Test command mapping
    echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | cargo run -- --hook
    ```

## Configuration

The tool behaves based on `.claude-hook-advisor.toml` in the current working directory.

**Example `.claude-hook-advisor.toml`:**
```toml
[commands]
npm = "bun"
curl = "wget --verbose"

[semantic_directories]
"project docs" = "~/Documents/my-project/docs"

[command_history]
enabled = true
log_file = "~/.claude-hook-advisor/bash-history.db"

[security_pattern_overrides]
swift_force_unwrap = false
```

## Development Conventions

*   **Code Style:** Standard Rust formatting (`rustfmt`). Run `make fmt` to format.
*   **Linting:** Uses `clippy`. Run `make lint` to check.
*   **Commits:** Follow **Conventional Commits** (e.g., `feat: ...`, `fix: ...`). This is enforced by `cog` for release generation.
*   **Testing:** Critical path logic (hooks, config parsing) must be tested. Since some tests involve file system operations, single-threaded testing is preferred.

## Directory Structure

*   `src/`: Rust source code.
    *   `main.rs`: CLI entry point.
    *   `lib.rs`: Library entry point.
    *   `cli.rs`: Command-line argument definition.
    *   `config.rs`: Configuration file parsing.
    *   `hooks.rs`: Core hook logic implementation.
    *   `directory.rs`: Semantic directory resolution logic.
    *   `history.rs`: SQLite database interaction for command history.
    *   `security.rs`: Security pattern matching.
*   `plugin/`: Scripts and assets for the Claude Code plugin.
*   `docs/`: Detailed documentation files.
*   `.claude-plugin/`: Plugin metadata (`marketplace.json`).
