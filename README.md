# Claude Hook Advisor

A Rust CLI tool that integrates with Claude Code using hooks to provide intelligent command suggestions, semantic directory aliasing, command history tracking, and security pattern detection.

## Features

- **🎯 Command Intelligence** - Automatically map commands to your preferred alternatives (npm → bun, curl → wget)
- **📁 Directory Aliasing** - Use natural language like "docs" or "project_docs" instead of typing full paths
- **📊 Command History** - Track all commands Claude runs in a SQLite database with session tracking
- **🔒 Security Patterns** - 27 built-in patterns detect dangerous code across 10+ languages
- **⚡ Fast & Lightweight** - Rust-based with ~1-5ms hook response time

### Quick Example

**You:** *"What files are in my docs?"*
**Claude:** *"I'll check your docs directory at /Users/you/Documents/Documentation"*

**Claude tries:** `npm install`
**Tool suggests:** `bun install`
**Claude runs:** `bun install` automatically

**Claude tries to write:** `eval(userInput)`
**Tool warns:** Security risk detected
**Claude:** Finds a safer alternative

## Installation

### Option 1: Plugin Marketplace (Recommended)

```bash
/plugin marketplace add sirmews/claude-hook-advisor
/plugin install claude-hook-advisor@sirmews
```

Includes automatic hook setup and slash commands (`/history`, `/history-failures`, `/history-search`).

### Option 2: From crates.io

```bash
cargo install claude-hook-advisor
claude-hook-advisor --install-hooks
```

### Option 3: From Source

```bash
git clone https://github.com/sirmews/claude-hook-advisor.git
cd claude-hook-advisor
make install
```

## Quick Start

1. Install via plugin marketplace (recommended) or cargo
2. Create `.claude-hook-advisor.toml` in your project root:

```toml
# Command mappings - map any command to your preferred tool
[commands]
npm = "bun"
yarn = "bun"
curl = "wget --verbose"

# Directory aliases - use natural language in conversations
[semantic_directories]
"project docs" = "~/Documents/my-project/docs"
"test data" = "~/Documents/test-data"
```

That's it! Start a Claude Code conversation and the hooks work automatically. Security patterns are enabled by default with no configuration needed.

<details>
<summary><b>📝 Optional: Enable Command History Tracking</b></summary>

Add to your `.claude-hook-advisor.toml`:

```toml
[command_history]
enabled = true
log_file = "~/.claude-hook-advisor/bash-history.db"
```

View history:
```bash
claude-hook-advisor --history               # Recent commands
claude-hook-advisor --history --failures    # Only failed commands
claude-hook-advisor --history --pattern git # Filter by pattern
```

</details>

<details>
<summary><b>🔒 Optional: Disable Noisy Security Patterns</b></summary>

Security patterns are enabled by default. To disable specific patterns:

```toml
[security_pattern_overrides]
swift_force_unwrap = false      # Common in Swift code
eval_injection = false          # If working on a REPL
rust_unsafe_block = false       # For systems programming
```

</details>

## Configuration

Run `claude-hook-advisor --install-hooks` to automatically configure hooks, or manually add to `.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": { "Bash": "claude-hook-advisor --hook" },
    "UserPromptSubmit": { ".*": "claude-hook-advisor --hook" },
    "PostToolUse": { "Bash": "claude-hook-advisor --hook" }
  }
}
```

---

## 📚 Documentation

<details>
<summary><b>How It Works</b></summary>

### Triple-Hook Architecture

Three hooks work together to provide comprehensive functionality:

#### 1. PreToolUse Hook - Command Intelligence 🚦

**The Flow:**
1. **Command Detection**: When Claude Code tries to run a Bash command, the hook receives JSON input
2. **Configuration Loading**: The tool loads `.claude-hook-advisor.toml` from the current directory
3. **Pattern Matching**: Matches only the primary command at the start of the line
4. **Suggestion Generation**: If a match is found, returns a blocking response with the suggested replacement
5. **Claude Integration**: Claude receives the suggestion and automatically retries with the correct command

**Smart Matching:**
- Start-of-line matching ensures only primary commands are replaced
- `npm install` → `bun install` ✅
- `npx npm` stays unchanged (npm is not the primary command) ✅
- `npm-check` stays unchanged (different command) ✅
- Preserves command arguments: `npm install --save` → `bun install --save` ✅

#### 2. UserPromptSubmit Hook - Directory Aliasing 📁

**The Flow:**
1. **Text Analysis**: Scans user prompts for semantic directory references (e.g., "docs", "project_docs")
2. **Pattern Recognition**: Uses regex to detect directory aliases in natural language
3. **Path Expansion**: Expands tilde (~) to user home directory
4. **Path Resolution**: Converts semantic references to canonical filesystem paths
5. **Security Validation**: Performs path canonicalization to prevent traversal attacks

**Security Features:**
- Path canonicalization prevents `../../../etc/passwd` attacks
- Only resolves to configured directories
- Validates paths exist before resolution

#### 3. PostToolUse Hook - Analytics & History 📊

**The Flow:**
1. **Execution Tracking**: Receives command results with success/failure data
2. **Database Logging**: Stores commands in SQLite with full metadata
3. **Session Tracking**: Links commands to Claude Code sessions

**Failure Detection:**
Uses a clever two-hook approach:
- **PreToolUse**: Logs every command Claude *attempts* with status="pending"
- **PostToolUse**: Updates status to "success" when commands complete
- Commands remaining "pending" = failed (PostToolUse never fired)

This workaround handles the limitation where Claude Code doesn't send PostToolUse events for failed commands.

</details>

<details>
<summary><b>Command History Tracking - Detailed Guide</b></summary>

### Setup

Enable in your `.claude-hook-advisor.toml`:

```toml
[command_history]
enabled = true
log_file = "~/.claude-hook-advisor/bash-history.db"
```

The PostToolUse hook (installed via `--install-hooks`) will automatically start logging. No restart needed!

### Viewing History

```bash
# Show recent commands
claude-hook-advisor --history

# Show last 50 commands
claude-hook-advisor --history --limit 50

# Show only failed commands
claude-hook-advisor --history --failures

# Show git commands only
claude-hook-advisor --history --pattern git

# Show commands from a specific session
claude-hook-advisor --history --session abc123
```

### What Gets Logged

Each command record includes:
- **Timestamp**: When the command was executed
- **Command**: The exact command that ran
- **Status**: Success (✓) or Failed (✗) - automatically tracked
- **Exit code**: Success (0) or failure code
- **Working directory**: Where the command was executed
- **Session ID**: Link commands to Claude Code sessions

### Example Output

```
Command History (5 records)
================================================================================

2025-11-10T14:30:22Z  ✓
  Command: git status
  CWD:     /home/user/my-project
  Session: abc123-def456

2025-11-10T14:30:25Z  ✓
  Command: cargo test
  CWD:     /home/user/my-project
  Session: abc123-def456

2025-11-10T14:30:30Z  ✗ FAILED
  Command: npm test
  CWD:     /home/user/my-project
  Session: abc123-def456
```

### Use Cases

- **Track failures**: Automatically identify which commands failed
- **Debugging**: "Which commands failed in this session?" - `--history --failures`
- **Retrieve commands**: "What was that complex curl command Claude ran yesterday?"
- **Audit trail**: Track all command attempts for compliance
- **Learning**: See what commands Claude tries and which ones work

</details>

<details>
<summary><b>Security Pattern Detection - Complete Reference</b></summary>

### Overview

**27 built-in security patterns** automatically detect dangerous code patterns when Claude edits files using `Edit`, `Write`, or `MultiEdit` tools. **Enabled by default** with no configuration needed.

### How It Works

When Claude tries to edit a file, the PreToolUse hook:
1. **Checks the file path** against glob patterns (e.g., `.github/workflows/*.yml`)
2. **Scans the content** for dangerous substrings (e.g., `eval(`, `dangerouslySetInnerHTML`)
3. **Blocks the operation** if a pattern matches and shows a security warning
4. **Tracks warnings per-session** so each warning is only shown once

Claude sees the warning and either finds a safer alternative, asks for your guidance, or explains why the risky operation is needed.

### Built-in Security Patterns

#### JavaScript / TypeScript (7 patterns)
- **`eval_injection`**: Detects `eval()` usage that can execute arbitrary code
- **`new_function_injection`**: Detects `new Function()` code injection risks
- **`innerHTML_xss`**: Detects `innerHTML` XSS vulnerabilities
- **`react_dangerously_set_html`**: Detects React `dangerouslySetInnerHTML` XSS risks
- **`document_write_xss`**: Detects `document.write()` XSS attacks
- **`child_process_exec`**: Detects command injection via `exec()`/`execSync()`

#### Python (4 patterns)
- **`python_eval`**: Detects `eval()` arbitrary code execution
- **`python_exec`**: Detects `exec()` arbitrary code execution
- **`pickle_deserialization`**: Detects unsafe `pickle.load()` usage
- **`os_system_injection`**: Detects command injection via `os.system()`

#### SQL (2 patterns)
- **`sql_injection`**: Detects SQL injection via string interpolation
- **`sql_string_format`**: Detects SQL injection via format strings

#### Rust (2 patterns)
- **`rust_unsafe_block`**: Detects `unsafe {}` blocks that bypass safety
- **`rust_command_injection`**: Detects shell command usage that could allow injection

#### Go (2 patterns)
- **`go_command_injection`**: Detects shell command injection risks
- **`go_sql_injection`**: Detects SQL injection via `fmt.Sprintf`

#### Swift (3 patterns)
- **`swift_force_unwrap`**: Detects force unwrap `!` that can cause crashes
- **`swift_unsafe_operations`**: Detects unsafe pointer operations
- **`swift_nspredicate_format`**: Detects NSPredicate injection vulnerabilities

#### Java (2 patterns)
- **`java_runtime_exec`**: Detects command injection via `Runtime.exec()`
- **`java_deserialization`**: Detects unsafe deserialization

#### PHP (2 patterns)
- **`php_eval`**: Detects `eval()` arbitrary code execution
- **`php_unserialize`**: Detects object injection via `unserialize()`

#### Ruby (2 patterns)
- **`ruby_eval`**: Detects `eval()` variants (eval, instance_eval, class_eval)
- **`ruby_yaml_load`**: Detects arbitrary code execution via `YAML.load`

#### GitHub Actions (2 patterns)
- **`github_actions_workflow`**: Detects workflow injection in `.yml` files
- **`github_actions_workflow_yaml`**: Detects workflow injection in `.yaml` files

### Example Security Warning

When Claude tries to write code with `eval()`:

```
⚠️ Security Warning: eval() executes arbitrary code and is a major security risk.

Consider using JSON.parse() for data parsing or alternative design patterns that
don't require code evaluation. Only use eval() if you truly need to evaluate
arbitrary code.
```

### Disabling Patterns

All patterns are enabled by default. To disable specific patterns:

```toml
[security_pattern_overrides]
swift_force_unwrap = false      # Common in Swift code
eval_injection = false          # If working on a REPL
python_eval = false
rust_unsafe_block = false       # For systems programming
```

**All pattern names:**
```
github_actions_workflow          github_actions_workflow_yaml
eval_injection                   new_function_injection
react_dangerously_set_html       document_write_xss
innerHTML_xss                    child_process_exec
pickle_deserialization           os_system_injection
python_eval                      python_exec
sql_injection                    sql_string_format
rust_unsafe_block                rust_command_injection
go_command_injection             go_sql_injection
swift_force_unwrap               swift_unsafe_operations
swift_nspredicate_format         java_runtime_exec
java_deserialization             php_eval
php_unserialize                  ruby_eval
ruby_yaml_load
```

</details>

<details>
<summary><b>Configuration Examples & Use Cases</b></summary>

### Example Configurations

**Node.js project (prefer bun):**
```toml
[commands]
npm = "bun"
yarn = "bun"
npx = "bunx"
```

**Python project (prefer uv):**
```toml
[commands]
pip = "uv pip"
"pip install" = "uv add"
```

**General tool preferences:**
```toml
[commands]
curl = "wget --verbose"
cat = "bat"
ls = "eza"
```

**Comprehensive project setup:**
```toml
# Command mappings
[commands]
npm = "bun"
yarn = "bun"
npx = "bunx"

# Semantic directory aliases
[semantic_directories]
"project docs" = "~/Documents/Documentation/my-project"
"central docs" = "~/Documents/Documentation"
"test data" = "~/Documents/test-data"

# Command history
[command_history]
enabled = true
log_file = "~/.claude-hook-advisor/bash-history.db"

# Disable noisy security patterns for this project
[security_pattern_overrides]
swift_force_unwrap = false
```

### Use Cases

#### Command Intelligence
- **Package Manager Consistency**: Enforce use of `bun` instead of `npm`/`yarn`
- **Tool Preferences**: Replace `curl` with `wget`, `cat` with `bat`, etc.
- **Project Standards**: Ensure consistent tooling across team members
- **Legacy Migration**: Gradually move from old tools to new ones

#### Directory Aliasing
- **Documentation Management**: Use "docs" instead of typing full paths
- **Project Organization**: Reference "project_docs", "central_docs" naturally
- **Team Collaboration**: Shared semantic directory references across team members
- **Workflow Automation**: Natural language directory references in Claude conversations

### Configuration File Lookup

The tool looks for configuration files in this order:
1. Custom path specified with `-c/--config` flag
2. `.claude-hook-advisor.toml` in current directory
3. If no config found, allows all commands (no mappings)

</details>

<details>
<summary><b>Testing & Development</b></summary>

### Testing Hooks Directly

Test hooks without Claude Code:

```bash
# Test directory resolution
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs"}' | claude-hook-advisor --hook

# Test command mapping
echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook

# Test PostToolUse hook
echo '{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{"command":"bun install"},"tool_response":{"exit_code":0}}' | claude-hook-advisor --hook
```

### Development Commands

```bash
make test          # Run unit tests
make build         # Build in debug mode
make release       # Build in release mode
make lint          # Run clippy linting
make fmt           # Format code with rustfmt
make clean         # Clean build artifacts
make help          # Show all available targets
```

### Manual Testing Examples

```bash
# Run unit tests
make test

# Test with example npm command
make run-example

# Manual testing - Command mapping (PreToolUse)
echo '{"session_id":"test","transcript_path":"","cwd":"","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"yarn start"}}' | ./target/debug/claude-hook-advisor --hook

# Manual testing - Directory detection (UserPromptSubmit)
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check the docs directory"}' | ./target/debug/claude-hook-advisor --hook
```

### Performance Notes

- **Startup time**: ~1-5ms per hook call
- **Memory usage**: ~2-3MB per process
- **File watching**: Configuration is loaded on each hook call (no caching)
- **Path resolution**: Uses filesystem canonicalization for security

</details>

<details>
<summary><b>Troubleshooting Guide</b></summary>

### Enable Debug Mode

Add `RUST_LOG=debug` to your Claude Code settings for detailed logging:

```json
{
  "hooks": {
    "UserPromptSubmit": { ".*": "RUST_LOG=debug claude-hook-advisor --hook" },
    "PreToolUse": { "Bash": "RUST_LOG=debug claude-hook-advisor --hook" },
    "PostToolUse": { "Bash": "RUST_LOG=debug claude-hook-advisor --hook" }
  }
}
```

**Debug output shows:**
- Configuration file loading
- Pattern matching details
- Path resolution steps
- Variable substitution
- Security validation

### Common Issues

#### 🚫 Hooks Not Triggering
**Problem:** No hook messages appear in Claude Code conversations

**Solutions:**
1. Verify hook installation: Check `.claude/settings.json` or `.claude/settings.local.json`
2. Ensure binary is in PATH: `which claude-hook-advisor`
3. Test manually:
   ```bash
   echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs"}' | claude-hook-advisor --hook
   ```
4. Verify hooks are configured:
   ```json
   {
     "hooks": {
       "UserPromptSubmit": { ".*": "claude-hook-advisor --hook" }
     }
   }
   ```

#### 📁 Directory Not Resolved
**Problem:** Directory aliases don't resolve to expected paths

**Solutions:**
1. Check configuration file exists: `ls .claude-hook-advisor.toml`
2. Verify alias configuration:
   ```toml
   [semantic_directories]
   docs = "~/Documents/Documentation"
   ```
3. Test resolution:
   ```bash
   echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs"}' | claude-hook-advisor --hook
   ```
4. Ensure path exists on filesystem
5. Check file permissions: `ls -la .claude-hook-advisor.toml`

#### ⚙️ Commands Not Being Mapped
**Problem:** Commands still run with original tool instead of mapped replacement

**Solutions:**
1. Verify command mapping in config:
   ```toml
   [commands]
   npm = "bun"
   ```
2. Test mapping:
   ```bash
   echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook
   ```
3. Remember: Commands only match at the start of the line (by design):
   - ✅ `npm install` → `bun install`
   - ❌ `npx npm` stays unchanged (npm is not the primary command)
   - ❌ `npm-check` stays unchanged (different command)

#### 🔒 Permission Issues
**Problem:** Hook fails with permission errors

**Solutions:**
1. Make binary executable: `chmod +x ~/.cargo/bin/claude-hook-advisor`
2. Check file ownership: `ls -la ~/.cargo/bin/claude-hook-advisor`
3. Verify PATH includes `~/.cargo/bin`: `echo $PATH`

#### 📊 Command History Not Logging
**Problem:** Commands aren't being saved to the database

**Solutions:**
1. Verify command history is enabled in config:
   ```toml
   [command_history]
   enabled = true
   log_file = "~/.claude-hook-advisor/bash-history.db"
   ```
2. Check PostToolUse hook is installed
3. Verify log file location is writable
4. Check database file permissions

### Understanding Hook Messages

When working correctly, you'll see these messages in Claude Code:

**Directory Resolution:**
```
<user-prompt-submit-hook>Directory reference 'docs' resolved to: /Users/you/Documents/Documentation</user-prompt-submit-hook>
```

**Command Suggestions:**
```
<pre-tool-use-hook>Command 'npm' mapped to 'bun'. Suggested: bun install</pre-tool-use-hook>
```

**Execution Tracking:**
```
<post-tool-use-hook>Command 'bun install' completed successfully (exit code: 0)</post-tool-use-hook>
```

</details>

## Support

If you find this tool useful, consider supporting its development:

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/A0A01HT0RG)

## Inspiration

The security pattern detection feature was inspired by Claude Code itself. While building this tool, I noticed how Claude Code implements its own security checks and validation patterns to protect users from dangerous operations. This inspired me to bring similar protective capabilities to custom hooks, allowing the community to extend Claude Code's safety mechanisms in their own workflows.
