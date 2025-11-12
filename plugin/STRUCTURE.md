# Plugin Structure

This document shows the complete structure of the Claude Hook Advisor plugin.

```
plugin/
├── .claude-plugin/
│   └── plugin.json                     # Plugin manifest with metadata
├── hooks/
│   └── hooks.json                      # PreToolUse, PostToolUse, UserPromptSubmit hooks
├── commands/
│   ├── history.md                      # /history command
│   ├── history-failures.md             # /history-failures command
│   ├── history-search.md               # /history-search <pattern> command
│   └── history-session.md              # /history-session <id> command
├── .claude-hook-advisor.toml.example   # Example config template
├── install.sh                          # Installation script
├── README.md                           # Plugin documentation
└── STRUCTURE.md                        # This file
```

## File Descriptions

### Core Files

**`.claude-plugin/plugin.json`**
- Plugin metadata (name, version, author)
- Required by Claude Code plugin system

**`hooks/hooks.json`**
- Configures PreToolUse, PostToolUse, and UserPromptSubmit hooks
- Automatically calls `claude-hook-advisor --hook` at appropriate times

### Slash Commands

**`commands/history.md`**
- Shows last 20 commands with success/failure status
- Includes AI analysis prompt

**`commands/history-failures.md`**
- Filters to show only failed commands
- AI suggests fixes for failures

**`commands/history-search.md`**
- Searches command history by pattern
- Uses `$ARGUMENTS` to pass search term

**`commands/history-session.md`**
- Shows all commands from a specific session
- Uses `$1` for session ID parameter

### Installation Files

**`install.sh`**
- Automated installation script
- Copies plugin to `~/.claude/plugins/`
- Updates settings.json
- Creates config template

**`.claude-hook-advisor.toml.example`**
- Example configuration file
- Users copy this to `.claude-hook-advisor.toml`

### Documentation

**`README.md`**
- Complete plugin documentation
- Installation instructions
- Usage examples
- Troubleshooting guide

## How It Works Together

1. **Plugin loads** when Claude Code starts (from settings.json)
2. **Hooks activate** automatically via hooks.json:
   - PreToolUse logs commands as "pending"
   - PostToolUse updates to "success"
   - UserPromptSubmit handles directory aliasing
3. **Slash commands** provide easy access to history
4. **Config file** (.claude-hook-advisor.toml) customizes behavior

## Installation Flow

```
User runs install.sh
    ↓
Plugin files copied to ~/.claude/plugins/claude-hook-advisor/
    ↓
settings.json updated to include plugin
    ↓
.claude-hook-advisor.toml created in project
    ↓
User restarts Claude Code
    ↓
Plugin active! Hooks running, slash commands available
```
