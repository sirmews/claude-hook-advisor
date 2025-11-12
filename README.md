# Claude Hook Advisor

A Rust CLI tool that integrates with Claude Code using a **triple-hook architecture** to provide intelligent command suggestions and semantic directory aliasing. Enhance your development workflow with automatic command mapping and natural language directory references.

## 🎬 What You'll Experience

Once installed, claude-hook-advisor works invisibly in your Claude Code conversations:

### Directory Aliasing Magic ✨
**You type:** *"What files are in my docs?"*
**Claude responds:** *"I'll check what files are in your docs directory at /Users/you/Documents/Documentation."*

Behind the scenes, you'll see:
```
<user-prompt-submit-hook>Directory reference 'docs' resolved to: /Users/you/Documents/Documentation</user-prompt-submit-hook>
```

**You type:** *"Check the project_docs for API documentation"*
**Claude automatically knows:** *Uses `/Users/you/Documents/Documentation/my-project/` without you typing the full path*

### Command Intelligence in Action 🚀
**Claude tries to run:** `npm install`
**Tool intervenes:** *Suggests `bun install` based on your configuration*
**Claude automatically runs:** `bun install` *with no manual intervention*

**You see:** Claude seamlessly uses your preferred tools without you having to correct it every time.

### The Magic is Invisible
- No extra commands to remember
- No interruptions to your workflow  
- Natural language directory references just work
- Your preferred tools are used automatically
- All happens transparently in Claude Code conversations

## Features

### 🎯 Command Intelligence
- **Smart command mapping**: Map any command to preferred alternatives with regex support
- **Per-project configuration**: Each project can have its own `.claude-hook-advisor.toml` file
- **Triple-hook integration**: PreToolUse, UserPromptSubmit, and PostToolUse hooks

### 📁 Semantic Directory Aliasing
- **Natural language directory references**: Use "docs", "central_docs", "project_docs" in conversations
- **Simple path mapping**: Direct alias-to-path mapping with tilde expansion
- **Automatic resolution**: Claude Code automatically resolves semantic references to canonical paths
- **TOML configuration**: Simple configuration file-based setup

### 📊 Command History Tracking
- **Persistent Bash history**: All commands Claude runs are logged to SQLite database
- **Never lose a command**: Commands run by Claude don't appear in your shell history, but now you can retrieve them
- **Powerful querying**: Filter by session, command pattern, exit code, or time range
- **Audit trail**: Track what Claude actually executed for debugging and compliance

### 🚀 Performance & Security
- **Fast and lightweight**: Built in Rust for optimal performance
- **Path canonicalization**: Security against directory traversal attacks
- **Graceful error handling**: Robust fallback mechanisms

## Installation

### From crates.io (Recommended)

Install directly from crates.io using cargo:

```bash
cargo install claude-hook-advisor
```

This installs the binary to `~/.cargo/bin/claude-hook-advisor` (make sure `~/.cargo/bin` is in your PATH).

### From Source

```bash
git clone https://github.com/sirmews/claude-hook-advisor.git
cd claude-hook-advisor
make install
```

## Quick Start

### 1. Install and Configure Hooks
```bash
# Install the binary
cargo install claude-hook-advisor

# Automatically install hooks into Claude Code (creates backups)
claude-hook-advisor --install-hooks

# Remove hooks if needed (with backup)
claude-hook-advisor --uninstall-hooks
```

### 2. Configure Directory Aliases
Edit your `.claude-hook-advisor.toml` file to set up directory aliases:

```toml
# Semantic directory aliases - use natural language!
[semantic_directories]
"project docs" = "~/Documents/Documentation/my-project"
"central docs" = "~/Documents/Documentation" 
"claude docs" = "~/Documents/Documentation/claude"
"test data" = "~/Documents/test-data"
```

**Pro tip:** Use quoted, space-separated aliases for natural conversation:
- *"Check the project docs folder"* → matches `"project docs"`
- *"Look in test data directory"* → matches `"test data"`

### 3. Configure Command Mappings
Create a `.claude-hook-advisor.toml` file in your project root:

```toml
# Command mappings
[commands]
npm = "bun"
yarn = "bun"
npx = "bunx"
curl = "wget --verbose"

# Semantic directory aliases - natural language
[semantic_directories]
"project docs" = "~/Documents/Documentation/my-project"
"central docs" = "~/Documents/Documentation"
"claude docs" = "~/Documents/Documentation/claude"
```

### 4. (Optional) Enable Command History Tracking
Track all commands Claude runs to a SQLite database:

```toml
[command_history]
enabled = true
log_file = "~/.claude-hook-advisor/bash-history.db"
```

Then view history anytime with: `claude-hook-advisor --history`

See the [Command History Tracking](#command-history-tracking) section for full details.

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

**General preferences:**
```toml
[commands]
curl = "wget --verbose"
cat = "bat"
ls = "eza"
```

## Claude Code Integration

### Automatic Installation (Recommended)
```bash
claude-hook-advisor --install-hooks
```

This automatically configures all three hooks:
- **PreToolUse**: Command suggestion and blocking
- **UserPromptSubmit**: Directory reference detection  
- **PostToolUse**: Analytics and execution tracking

### Manual Configuration

If you prefer manual setup, add to your `.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": { "Bash": "claude-hook-advisor --hook" },
    "UserPromptSubmit": { ".*": "claude-hook-advisor --hook" },
    "PostToolUse": { "Bash": "claude-hook-advisor --hook" }
  }
}
```

**Note**: This assumes `claude-hook-advisor` is in your PATH. After `cargo install`, the binary is typically located at `~/.cargo/bin/claude-hook-advisor`.

## How It Works

### Command Intelligence (PreToolUse Hook) 🚦

**The Flow:**
1. **Command Detection**: When Claude Code tries to run a Bash command, the hook receives JSON input
2. **Configuration Loading**: The tool loads `.claude-hook-advisor.toml` from the current directory
3. **Pattern Matching**: Matches only the primary command at the start of the line (e.g., `npm` matches `npm install` but not `npx npm` or `my-npm-tool`)
4. **Suggestion Generation**: If a match is found, returns a blocking response with the suggested replacement
5. **Claude Integration**: Claude receives the suggestion and automatically retries with the correct command

**Behind the Scenes:**
```rust
// Simplified code flow
let config = load_config(".claude-hook-advisor.toml")?;
let command = parse_bash_command(&hook_input.tool_input.command);

if let Some(replacement) = config.commands.get(&command.base_command) {
    return Ok(HookResponse::Block {
        reason: format!("Command '{}' is mapped to '{}'", command.base_command, replacement),
        suggested_command: command.replace_base_with(replacement),
    });
}
```

**What makes it smart:**
- Start-of-line matching ensures only primary commands are replaced (e.g., `npm install` → `bun install`, but `npx npm` is unchanged)
- Prevents false positives with substrings (e.g., `npm` won't match `npm-check` or `my-npm-tool`)
- Doesn't interfere with subcommands (e.g., `git rm` won't trigger an `rm` mapping)
- Preserves command arguments (`npm install --save` → `bun install --save`)
- Fast regex-based pattern matching (~1ms response time)

---

### Directory Aliasing (UserPromptSubmit Hook) 📁

**The Flow:**
1. **Text Analysis**: Scans user prompts for semantic directory references (e.g., "docs", "project_docs")
2. **Pattern Recognition**: Uses regex to detect directory aliases in natural language
3. **Path Expansion**: Expands tilde (~) to user home directory
4. **Path Resolution**: Converts semantic references to canonical filesystem paths
5. **Security Validation**: Performs path canonicalization to prevent traversal attacks

**Behind the Scenes:**
```rust
// Pattern detection
let patterns = [
    r"\b(docs|documentation)\b",
    r"\bproject[_\s]docs?\b", 
    r"\bcentral[_\s]docs?\b"
];

// Tilde expansion
let resolved = expand_tilde(path_template)?;

// Security canonicalization
let canonical = fs::canonicalize(&resolved)?;
```

**What makes it secure:**
- Path canonicalization prevents `../../../etc/passwd` attacks
- Only resolves to configured directories
- Validates paths exist before resolution

---

### Analytics (PostToolUse Hook) 📊

**The Flow:**
1. **Execution Tracking**: Receives command results with success/failure data
2. **Performance Monitoring**: Tracks command success rates and execution patterns
3. **Analytics Logging**: Provides insights for optimization and monitoring

**Behind the Scenes:**
```rust
// Success/failure tracking
match hook_data.tool_response.exit_code {
    0 => log::info!("Command '{}' succeeded", command),
    code => log::warn!("Command '{}' failed (exit: {})", command, code),
}
```

**Future possibilities:**
- Command success rate analytics
- Performance optimization suggestions
- Usage pattern insights

## Command History Tracking

Track every Bash command Claude runs in a SQLite database. Commands executed by Claude don't show up in your shell's history, but now you can retrieve them anytime.

### Setup

**1. Install/Update the binary:**

If you just cloned or pulled the latest code:
```bash
cargo install --path .
```

If you installed from crates.io, the feature is already available in v0.2.0+.

**2. Enable command history:**

Edit your `.claude-hook-advisor.toml` and add:

```toml
[command_history]
enabled = true
log_file = "~/.claude-hook-advisor/bash-history.db"
```

**Or** if you don't have a config file yet, run:
```bash
claude-hook-advisor --install
```
This creates a config file with command history as a commented example you can uncomment.

**3. That's it!**

The PostToolUse hook (already installed if you ran `--install` before) will automatically start logging commands. No restart needed!

### Viewing History

**Show recent commands:**
```bash
claude-hook-advisor --history
```

**Show last 50 commands:**
```bash
claude-hook-advisor --history --limit 50
```

**Show only failed commands:**
```bash
claude-hook-advisor --history --failures
```

**Show git commands only:**
```bash
claude-hook-advisor --history --pattern git
```

**Show commands from a specific session:**
```bash
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

### How Failure Detection Works

The tool uses a clever two-hook approach to track both successful and failed commands:

1. **PreToolUse Hook**: Logs every command Claude *attempts* to run with status="pending"
2. **PostToolUse Hook**: Updates the status to "success" when commands complete

**Key insight**: PostToolUse hooks only fire for successful commands in Claude Code. Any command that remains with status="pending" means it failed (PostToolUse never fired).

This workaround solves the limitation where Claude Code doesn't send PostToolUse events for failed commands, allowing you to track all command attempts and their outcomes.

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

2025-11-10T14:30:35Z  ✗ FAILED
  Command: ls /nonexistent
  CWD:     /home/user/my-project
  Session: abc123-def456
```

### Use Cases

- **Track failures**: Automatically identify which commands failed without manual checking
- **Debugging**: "Which commands failed in this session?" - `--history --failures`
- **Retrieve that perfect command**: "What was that complex curl command Claude ran yesterday?"
- **Audit trail**: Track all command attempts (successful and failed) for compliance
- **Learning**: See what commands Claude tries and which ones work
- **Identify patterns**: Find commands that consistently fail and need attention

## Example Output

### Real Claude Code Conversation

Here's what an actual conversation looks like with claude-hook-advisor working:

**🗣️ You:** "What files are in my docs?"

**🤖 Claude:** "⏺ I'll check what files are in your docs directory at /Users/you/Documents/Documentation."

**Behind the scenes:**
```
[DEBUG] UserPromptSubmit hook triggered
[DEBUG] Pattern matched: 'docs' -> '~/Documents/Documentation'  
[DEBUG] Path resolved: /Users/you/Documents/Documentation
```

**Hook message in Claude:**
```
<user-prompt-submit-hook>Directory reference 'docs' resolved to: /Users/you/Documents/Documentation</user-prompt-submit-hook>
```

---

**🗣️ You:** "Install the dependencies for this project"

**🤖 Claude:** "I'll install the dependencies using npm install."
*(Claude attempts: `npm install`)*

**Hook intercepts:**
```json
{
  "decision": "block",
  "reason": "Command 'npm' is mapped to 'bun' instead",
  "suggested_command": "bun install"
}
```

**🤖 Claude:** "I'll use bun install instead based on your project preferences."
*(Claude runs: `bun install`)*

**Result:** Your preferred package manager is used automatically, no manual correction needed!

---

### Command Line Testing

**Directory Resolution:**
```bash
# Test directory resolution via hook
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs directory"}' | claude-hook-advisor --hook

# Expected output:
# Directory reference 'docs' resolved to: /Users/you/Documents/Documentation

*Note: Directory resolution requires the path to exist on your filesystem.*
```

**Hook Simulation:**
```bash
$ echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check the docs directory"}' | claude-hook-advisor --hook
<user-prompt-submit-hook>Directory reference 'docs' resolved to: /Users/you/Documents/Documentation</user-prompt-submit-hook>

$ echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook
{
  "decision": "block", 
  "reason": "Command 'npm' is mapped to 'bun' instead",
  "suggested_command": "bun install"
}
```

## Development

### Available Make Targets

```bash
make build         # Build in debug mode
make release       # Build in release mode
make test          # Run tests
make lint          # Run clippy linting
make fmt           # Format code
make clean         # Clean build artifacts
make example-config# Create example config
make run-example   # Test with example input
make help          # Show all targets
```

### Testing

```bash
# Run unit tests
make test

# Test with example npm command
make run-example

# Manual testing - Command mapping (PreToolUse)
echo '{"session_id":"test","transcript_path":"","cwd":"","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"yarn start"}}' | ./target/debug/claude-hook-advisor --hook

# Manual testing - Directory detection (UserPromptSubmit)  
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check the docs directory"}' | ./target/debug/claude-hook-advisor --hook

# Manual testing - Analytics (PostToolUse)
echo '{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{"command":"bun install"},"tool_response":{"exit_code":0}}' | ./target/debug/claude-hook-advisor --hook

# Test directory resolution with existing config
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check the docs directory"}' | ./target/debug/claude-hook-advisor --hook
```

## 🔧 Troubleshooting & Debug

### Understanding Hook Messages

When claude-hook-advisor is working correctly, you'll see these messages in Claude Code:

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

### Debug Mode

Enable detailed logging to see what's happening behind the scenes:

```bash
# Add RUST_LOG=debug to your Claude Code settings
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

### Common Issues & Solutions

#### 🚫 Hooks Not Triggering
**Problem:** No hook messages appear in Claude Code conversations

**Solutions:**
1. Verify hook installation by checking your Claude Code settings file
2. Check `.claude/settings.json` or `.claude/settings.local.json`:
   ```json
   {
     "hooks": {
       "UserPromptSubmit": { ".*": "claude-hook-advisor --hook" }
     }
   }
   ```
3. Ensure `claude-hook-advisor` is in your PATH: `which claude-hook-advisor`
4. Test manually: `echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs"}' | claude-hook-advisor --hook`

#### 📁 Directory Not Resolved
**Problem:** "docs" doesn't resolve to the expected path

**Solutions:**
1. Check configuration file exists: `ls .claude-hook-advisor.toml`
2. Verify alias configuration:
   ```toml
   [semantic_directories]
   docs = "~/Documents/Documentation"
   ```
3. Test resolution via hook: `echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs"}' | claude-hook-advisor --hook`
4. Check file permissions: `ls -la .claude-hook-advisor.toml`

#### ⚙️ Commands Not Being Mapped
**Problem:** `npm` still runs instead of `bun`

**Solutions:**
1. Verify command mapping in config:
   ```toml
   [commands]
   npm = "bun"
   ```
2. Test mapping: `echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook`
3. Remember: Commands only match at the start of the line (by design):
   - ✅ `npm install` matches and becomes `bun install`
   - ❌ `npx npm` won't match (npm is not the primary command)
   - ❌ `npm-check` won't match (different command)
4. Add debug logging to see pattern matching

#### 🔒 Permission Issues
**Problem:** Hook fails with permission errors

**Solutions:**
1. Make binary executable: `chmod +x ~/.cargo/bin/claude-hook-advisor`
2. Check file ownership: `ls -la ~/.cargo/bin/claude-hook-advisor`
3. Verify PATH includes `~/.cargo/bin`: `echo $PATH`

#### 🐛 Debugging Your Configuration

**Test each component individually:**

```bash
# Test directory resolution via hook
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs"}' | claude-hook-advisor --hook

# Test command mapping
echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook

# Test user prompt analysis
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check the docs directory"}' | claude-hook-advisor --hook

# Check configuration by testing resolution
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs"}' | claude-hook-advisor --hook
```

### Performance Notes

- **Startup time:** ~1-5ms per hook call
- **Memory usage:** ~2-3MB per process  
- **File watching:** Configuration is loaded on each hook call (no caching)
- **Path resolution:** Uses filesystem canonicalization for security

## Configuration File Lookup

The tool looks for configuration files in this order:

1. Custom path specified with `-c/--config` flag
2. `.claude-hook-advisor.toml` in current directory
3. If no config found, allows all commands (no mappings)

## Use Cases

### Command Intelligence
- **Package Manager Consistency**: Enforce use of `bun` instead of `npm`/`yarn`
- **Tool Preferences**: Replace `curl` with `wget`, `cat` with `bat`, etc.
- **Project Standards**: Ensure consistent tooling across team members
- **Legacy Migration**: Gradually move from old tools to new ones
- **Security Policies**: Block dangerous commands or redirect to safer alternatives

### Directory Aliasing
- **Documentation Management**: Use "docs" instead of typing full paths
- **Project Organization**: Reference "project_docs", "central_docs" naturally
- **Cross-Platform Paths**: Abstract away platform-specific directory structures
- **Team Collaboration**: Shared semantic directory references across team members
- **Workflow Automation**: Natural language directory references in Claude conversations

## Similar Tools

This tool is inspired by and similar to:
- Shell aliases (but works at the Claude Code level)
- Git hooks (but for command execution)
- Package manager configuration files

## Support

If you find this tool useful, consider supporting its development:

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/A0A01HT0RG)

---
