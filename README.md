# Claude Hook Advisor

A Rust CLI tool that integrates with Claude Code using a **triple-hook architecture** to provide intelligent command suggestions and semantic directory aliasing. Enhance your development workflow with automatic command mapping and natural language directory references.

## Features

### 🎯 Command Intelligence
- **Smart command mapping**: Map any command to preferred alternatives with regex support
- **Per-project configuration**: Each project can have its own `.claude-hook-advisor.toml` file
- **Triple-hook integration**: PreToolUse, UserPromptSubmit, and PostToolUse hooks

### 📁 Semantic Directory Aliasing
- **Natural language directory references**: Use "docs", "central_docs", "project_docs" in conversations
- **Variable substitution**: Dynamic paths with `{project}` and `{user_home}` variables
- **Automatic resolution**: Claude Code automatically resolves semantic references to canonical paths
- **CLI management**: Complete command-line interface for alias management

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

### 2. Set Up Directory Aliases
```bash
# Add semantic directory aliases
claude-hook-advisor --add-directory-alias "docs" "~/Documents/Documentation"
claude-hook-advisor --add-directory-alias "project_docs" "~/Documents/Documentation/{project}"

# List all configured aliases
claude-hook-advisor --list-directory-aliases

# Resolve an alias to see its canonical path
claude-hook-advisor --resolve-directory "docs"

# Remove an alias
claude-hook-advisor --remove-directory-alias "docs"
```

### 3. Configure Command Mappings
Create a `.claude-hook-advisor.toml` file in your project root:

```toml
# Command mappings
[commands]
npm = "bun"
yarn = "bun"
npx = "bunx"
curl = "wget --verbose"

# Semantic directory aliases
[semantic_directories]
docs = "~/Documents/Documentation"
central_docs = "~/Documents/Documentation"
project_docs = "~/Documents/Documentation/{project}"
claude_docs = "~/Documents/Documentation/claude"

# Directory variables for substitution
[directory_variables]
project = "my-project"          # Or auto-detected from git
user_home = "~"
```

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

### Command Intelligence (PreToolUse Hook)
1. **Command Detection**: When Claude Code tries to run a Bash command, the hook receives JSON input
2. **Configuration Loading**: The tool loads `.claude-hook-advisor.toml` from the current directory
3. **Pattern Matching**: Uses word-boundary regex to match commands (e.g., `npm` matches `npm install` but not `npm-check`)
4. **Suggestion Generation**: If a match is found, returns a blocking response with the suggested replacement
5. **Claude Integration**: Claude receives the suggestion and automatically retries with the correct command

### Directory Aliasing (UserPromptSubmit Hook)
1. **Text Analysis**: Scans user prompts for semantic directory references (e.g., "docs", "project_docs")
2. **Pattern Recognition**: Uses regex to detect directory aliases in natural language
3. **Variable Substitution**: Resolves variables like `{project}` and `{user_home}` in path templates
4. **Path Resolution**: Converts semantic references to canonical filesystem paths
5. **Security Validation**: Performs path canonicalization to prevent traversal attacks

### Analytics (PostToolUse Hook)
1. **Execution Tracking**: Receives command results with success/failure data
2. **Performance Monitoring**: Tracks command success rates and execution patterns
3. **Analytics Logging**: Provides insights for optimization and monitoring

## Example Output

### Command Mapping Example
When Claude tries to run `npm install`, the tool outputs:

```json
{
  "decision": "block",
  "reason": "Command 'npm' is mapped to use 'bun' instead. Try: bun install"
}
```

Claude then sees this feedback and automatically runs `bun install` instead.

### Directory Aliasing Example
When you say *"Please check the docs directory"*, the tool detects the reference and outputs:

```
Directory reference detected: 'docs' -> '/Users/you/Documents/Documentation'
```

Claude then automatically uses the canonical path for file operations.

### Variable Substitution Example
With `project_docs` configured as `~/Documents/Documentation/{project}`:

```bash
# In project "my-app"
claude-hook-advisor --resolve-directory "project_docs"
# Output: /Users/you/Documents/Documentation/my-app
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

# Directory alias management
./target/debug/claude-hook-advisor --add-directory-alias "test_docs" "~/Documents/test"
./target/debug/claude-hook-advisor --list-directory-aliases
./target/debug/claude-hook-advisor --resolve-directory "test_docs"
```

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
