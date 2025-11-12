# Claude Hook Advisor Marketplace

This repository is a Claude Code plugin marketplace for the claude-hook-advisor plugin.

## For Users: Installing the Plugin

### Quick Installation (Recommended)

1. **Add the marketplace** to Claude Code:
   ```
   /plugin marketplace add sirmews/claude-hook-advisor
   ```

2. **Install the plugin**:
   ```
   /plugin install claude-hook-advisor@sirmews
   ```

3. **Restart Claude Code** to activate the plugin

That's it! The plugin is now installed and ready to use.

### Alternative: Interactive Installation

1. **Add the marketplace**:
   ```
   /plugin marketplace add sirmews/claude-hook-advisor
   ```

2. **Browse plugins interactively**:
   ```
   /plugin
   ```

3. Select "Browse Plugins" → Choose "claude-hook-advisor" → "Install now"

4. **Restart Claude Code**

### Verify Installation

After restarting Claude Code, verify the plugin is active:

```
/plugin list
```

You should see `claude-hook-advisor` in the list.

## Using the Plugin

### Slash Commands

Once installed, these commands are available:

- **`/history`** - View recent command history (last 20 commands)
- **`/history-failures`** - Show only failed commands with AI-suggested fixes
- **`/history-search <pattern>`** - Search for specific commands
  - Example: `/history-search git`
- **`/history-session <id>`** - View complete session timeline
  - Example: `/history-session abc123-def456`

### Configuration

The plugin automatically creates a `.claude-hook-advisor.toml` config file in your project. Edit it to customize:

```toml
[commands]
# Command mappings - replace commands with preferred alternatives
npm = "bun"

[semantic_directories]
# Natural language directory references
docs = "~/Documents/Documentation"

[command_history]
# Command history tracking
enabled = true
log_file = "~/.claude-hook-advisor/bash-history.db"
```

## Prerequisites

**Important**: You must install the `claude-hook-advisor` binary first:

```bash
cargo install claude-hook-advisor
```

This installs the command-line tool that the plugin hooks call.

## Features

### Automatic Failure Detection
- Tracks all bash commands (successful and failed)
- Uses PreToolUse + PostToolUse hooks to detect failures
- Commands that remain "pending" are automatically marked as failed

### AI-Powered Analysis
- Slash commands include AI analysis prompts
- `/history-failures` suggests fixes for failed commands
- Contextual insights about command execution

### Zero Configuration
- Hooks configured automatically
- Works out of the box
- Optional customization via config file

## Team Installation

For teams, add the marketplace to your `.claude/settings.json`:

```json
{
  "marketplaces": [
    {
      "name": "sirmews",
      "url": "https://github.com/sirmews/claude-hook-advisor"
    }
  ],
  "plugins": [
    "claude-hook-advisor@sirmews"
  ]
}
```

This automatically installs the plugin for all team members.

## Troubleshooting

### Plugin Not Found

If you get "plugin not found":

1. Verify the marketplace is added:
   ```
   /plugin marketplace list
   ```

2. If missing, add it:
   ```
   /plugin marketplace add sirmews/claude-hook-advisor
   ```

### Binary Not Found

If you get errors about `claude-hook-advisor` not being found:

1. Install the binary:
   ```bash
   cargo install claude-hook-advisor
   ```

2. Verify it's in your PATH:
   ```bash
   which claude-hook-advisor
   ```

3. Ensure `~/.cargo/bin` is in your PATH

### Hooks Not Firing

1. Restart Claude Code (hooks load at session start)
2. Check the config file exists: `.claude-hook-advisor.toml`
3. Verify history is enabled in config

## Manual Installation (Alternative)

If you prefer not to use the marketplace:

1. Clone the repository:
   ```bash
   git clone https://github.com/sirmews/claude-hook-advisor.git
   cd claude-hook-advisor
   ```

2. Run the installation script:
   ```bash
   ./plugin/install.sh
   ```

3. Restart Claude Code

## Support

- **GitHub**: https://github.com/sirmews/claude-hook-advisor
- **Issues**: https://github.com/sirmews/claude-hook-advisor/issues
- **Documentation**: https://github.com/sirmews/claude-hook-advisor#readme

## Version

Current version: 1.0.0

## License

MIT License - See LICENSE file for details
