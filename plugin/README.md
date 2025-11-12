# Claude Hook Advisor Plugin

A Claude Code plugin that provides intelligent command history tracking with automatic failure detection.

## Features

### 🎯 Automatic Command Tracking
- **All commands captured**: Both successful and failed bash commands
- **Failure detection**: Automatically identifies failed commands (no manual checking needed)
- **Rich metadata**: Timestamps, exit codes, working directories, session IDs

### 📊 Slash Commands for Easy Access
- `/history` - View recent command history (last 20 commands)
- `/history-failures` - Show only failed commands with AI analysis
- `/history-search <pattern>` - Search for specific commands (e.g., `/history-search git`)
- `/history-session <id>` - View all commands from a specific session

### 🔧 Smart Hooks
- **PreToolUse**: Logs every command attempt as "pending"
- **PostToolUse**: Updates status to "success" when complete
- **UserPromptSubmit**: Enables semantic directory aliasing

### 🎨 Additional Capabilities
- **Command mapping**: Replace commands automatically (npm → bun)
- **Directory aliasing**: Use natural language for paths ("docs" → ~/Documents/Documentation)

## Prerequisites

**Install the claude-hook-advisor binary first:**

```bash
# From crates.io (recommended)
cargo install claude-hook-advisor

# Or from source
git clone https://github.com/sirmews/claude-hook-advisor.git
cd claude-hook-advisor
cargo install --path .
```

**Verify installation:**
```bash
which claude-hook-advisor
# Should output: ~/.cargo/bin/claude-hook-advisor
```

## Installation

### Option 1: Local Plugin Installation

1. **Copy the plugin directory** to your project:
   ```bash
   cp -r plugin ~/.claude/plugins/claude-hook-advisor
   ```

2. **Enable the plugin** in your `.claude/settings.json`:
   ```json
   {
     "plugins": [
       "~/.claude/plugins/claude-hook-advisor"
     ]
   }
   ```

3. **Create configuration file** in your project root:
   ```bash
   cat > .claude-hook-advisor.toml << 'EOF'
   [commands]
   # Add your command mappings here
   # npm = "bun"

   [semantic_directories]
   # Add your directory aliases here
   # docs = "~/Documents/Documentation"

   [command_history]
   enabled = true
   log_file = "~/.claude-hook-advisor/bash-history.db"
   EOF
   ```

4. **Restart Claude Code** to activate the plugin

### Option 2: Project-Specific Installation

Place the plugin directory in your project's `.claude/plugins/` folder:

```bash
mkdir -p .claude/plugins
cp -r plugin .claude/plugins/claude-hook-advisor
```

Add to `.claude/settings.local.json`:
```json
{
  "plugins": [
    "./.claude/plugins/claude-hook-advisor"
  ]
}
```

## Usage

### View Command History

Simply type:
```
/history
```

You'll see:
```
Command History (5 records)
================================================================================

2025-11-12T14:30:35Z  ✗ FAILED
  Command: npm test
  CWD:     /home/user/my-project
  Session: abc123-def456

2025-11-12T14:30:30Z  ✓
  Command: git status
  CWD:     /home/user/my-project
  Session: abc123-def456
```

### Debug Failed Commands

```
/history-failures
```

Claude will automatically:
- Show all failed commands
- Analyze why they failed
- Suggest fixes

### Search for Specific Commands

```
/history-search git
```

Finds all commands containing "git" in the history.

### Review a Specific Session

```
/history-session abc123-def456
```

Shows complete timeline of a session's commands.

## How It Works

### Automatic Failure Detection

The plugin uses a clever two-hook approach:

1. **PreToolUse Hook**: Logs every command Claude attempts with status="pending"
2. **PostToolUse Hook**: Updates status to "success" when commands complete

**Key Insight**: Claude Code only fires PostToolUse for successful commands. Any command that remains "pending" means it failed!

This workaround solves the limitation where Claude Code doesn't send PostToolUse events for failed commands.

### Database Storage

All commands are stored in SQLite at `~/.claude-hook-advisor/bash-history.db` (configurable).

Each record includes:
- Timestamp (ISO 8601)
- Command text
- Status (success/pending)
- Exit code
- Working directory
- Session ID

## Configuration

Edit `.claude-hook-advisor.toml` in your project root:

```toml
[commands]
npm = "bun"              # Replace npm with bun
yarn = "bun"             # Replace yarn with bun
curl = "wget --verbose"  # Use wget instead of curl

[semantic_directories]
docs = "~/Documents/Documentation"
project_docs = "~/Documents/my-project"

[command_history]
enabled = true
log_file = "~/.claude-hook-advisor/bash-history.db"
```

## Troubleshooting

### Plugin Not Loading

1. **Check binary installation**:
   ```bash
   which claude-hook-advisor
   ```
   Should return `~/.cargo/bin/claude-hook-advisor`

2. **Verify plugin path** in settings.json is correct

3. **Check logs** in Claude Code for errors

4. **Restart Claude Code** after installation

### History Not Recording

1. **Verify config file exists** at `.claude-hook-advisor.toml`

2. **Check history is enabled**:
   ```toml
   [command_history]
   enabled = true
   ```

3. **Test manually**:
   ```bash
   claude-hook-advisor --history
   ```

### Hooks Not Firing

1. **Restart Claude Code** (hooks load at session start)

2. **Check hooks.json** is in `plugin/hooks/hooks.json`

3. **Verify permissions**:
   ```bash
   chmod +x ~/.cargo/bin/claude-hook-advisor
   ```

## Advanced Usage

### Query History Directly

You can use the CLI directly for advanced queries:

```bash
# Last 50 commands
claude-hook-advisor --history --limit 50

# Only failures
claude-hook-advisor --history --failures

# Search pattern
claude-hook-advisor --history --pattern "npm"

# Specific session
claude-hook-advisor --history --session abc123
```

### Combine with Other Tools

The command history is stored in SQLite, so you can query it directly:

```bash
# View schema
sqlite3 ~/.claude-hook-advisor/bash-history.db ".schema"

# Custom queries
sqlite3 ~/.claude-hook-advisor/bash-history.db \
  "SELECT command, status FROM commands WHERE status='pending' LIMIT 10"
```

## Use Cases

- **Debugging**: "What commands failed in my last session?"
- **Learning**: "Show me all git commands Claude used"
- **Auditing**: Track all command attempts for compliance
- **Optimization**: Find commands that frequently fail
- **Recovery**: "What was that curl command that worked yesterday?"

## Support

- **GitHub**: https://github.com/sirmews/claude-hook-advisor
- **Issues**: https://github.com/sirmews/claude-hook-advisor/issues
- **Docs**: https://github.com/sirmews/claude-hook-advisor#readme

## License

Same as claude-hook-advisor main project.

## Credits

Created by [sirmews](https://github.com/sirmews) as part of the claude-hook-advisor project.
