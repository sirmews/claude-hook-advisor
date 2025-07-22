# Claude Hook Advisor

A Rust CLI tool that advises Claude Code on better command alternatives based on project preferences. Similar to `hashtag-search`, this tool allows you to create per-project configurations that automatically suggest preferred commands when Claude Code tries to run specific commands.

## Features

- **Per-project configuration**: Each project can have its own `.claude-hook-advisor.toml` file
- **Flexible command mapping**: Map any command to any replacement with regex support
- **Claude Code integration**: Works seamlessly as a PreToolUse hook
- **Fast and lightweight**: Built in Rust for performance

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

## Configuration

Create a `.claude-hook-advisor.toml` file in your project root:

```toml
[commands]
npm = "bun"
yarn = "bun"
npx = "bunx"
curl = "wget --verbose"
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

### Option 1: Using the `/hooks` command

1. Run `/hooks` in Claude Code
2. Select `PreToolUse`
3. Add matcher: `Bash`
4. Add hook command: `/path/to/claude-hook-advisor --hook`
5. Save to project settings

### Option 2: Manual settings configuration

Add to your `.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "/path/to/claude-hook-advisor --hook"
          }
        ]
      }
    ]
  }
}
```

### Option 3: Using absolute path after installation

If you've installed via `make install`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "~/.local/bin/claude-hook-advisor --hook"
          }
        ]
      }
    ]
  }
}
```

## How It Works

1. **Command Detection**: When Claude Code tries to run a Bash command, the hook receives JSON input
2. **Configuration Loading**: The tool loads `.claude-hook-advisor.toml` from the current directory
3. **Pattern Matching**: Uses word-boundary regex to match commands (e.g., `npm` matches `npm install` but not `npm-check`)
4. **Suggestion Generation**: If a match is found, returns a blocking response with the suggested replacement
5. **Claude Integration**: Claude receives the suggestion and automatically retries with the correct command

## Example Output

When Claude tries to run `npm install`, the tool outputs:

```json
{
  "decision": "block",
  "reason": "Command 'npm' is mapped to use 'bun' instead. Try: bun install"
}
```

Claude then sees this feedback and automatically runs `bun install` instead.

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

# Manual testing
echo '{"session_id":"test","transcript_path":"","cwd":"","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"yarn start"}}' | ./target/debug/claude-hook-advisor --hook
```

## Configuration File Lookup

The tool looks for configuration files in this order:

1. Custom path specified with `-c/--config` flag
2. `.claude-hook-advisor.toml` in current directory
3. If no config found, allows all commands (no mappings)

## Use Cases

- **Package Manager Consistency**: Enforce use of `bun` instead of `npm`/`yarn`
- **Tool Preferences**: Replace `curl` with `wget`, `cat` with `bat`, etc.
- **Project Standards**: Ensure consistent tooling across team members
- **Legacy Migration**: Gradually move from old tools to new ones
- **Security Policies**: Block dangerous commands or redirect to safer alternatives

## Similar Tools

This tool is inspired by and similar to:
- `hashtag-search` (sibling Rust tool in this project)
- Shell aliases (but works at the Claude Code level)
- Git hooks (but for command execution)

## Support

If you find this tool useful, consider supporting its development:

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/A0A01HT0RG)

---
