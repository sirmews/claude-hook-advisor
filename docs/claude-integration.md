---
title: "Claude Code Integration"
description: "Complete guide for integrating Claude Hook Advisor with Claude Code"
tags: ["claude-code", "hooks", "integration", "setup", "pretooluse"]
---

# Claude Code Integration

This guide covers how to integrate Claude Hook Advisor with Claude Code using the PreToolUse hook system.

## 🎯 Overview

Claude Code's hook system allows you to intercept and modify tool calls before they execute. Claude Hook Advisor integrates as a PreToolUse hook that:

- **Intercepts Bash commands** before execution
- **Suggests better alternatives** based on your configuration
- **Blocks problematic commands** and provides guidance
- **Works transparently** with Claude's conversation flow

## 🔧 Integration Methods

### Method 1: Using the `/hooks` Command (Recommended)

The easiest way to set up the integration:

1. **Open Claude Code** in your project directory
2. **Run the hooks command**:
   ```
   /hooks
   ```
3. **Select PreToolUse** from the hook types
4. **Add matcher**: `Bash`
5. **Add hook command**: 
   ```
   claude-hook-advisor --hook
   ```
   Or with full path:
   ```
   ~/.local/bin/claude-hook-advisor --hook
   ```
6. **Save to project settings**

### Method 2: Manual Settings Configuration

For more control, manually edit your `.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "claude-hook-advisor --hook"
          }
        ]
      }
    ]
  }
}
```

### Method 3: Global Configuration

To apply across all projects, use a global settings file:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "/usr/local/bin/claude-hook-advisor --hook"
          }
        ]
      }
    ]
  }
}
```

## 📁 Configuration File Locations

### Project-Specific Settings
```
your-project/
├── .claude/
│   └── settings.json          # Project-specific hook configuration
├── .claude-hook-advisor.toml  # Command mappings for this project
└── src/
```

### Global Settings
```
~/.claude/
└── settings.json              # Global hook configuration
```

## 🔄 Hook Protocol

### Input Format
Claude Code sends JSON input to the hook:

```json
{
  "session_id": "unique-session-id",
  "transcript_path": "/path/to/transcript",
  "cwd": "/current/working/directory",
  "hook_event_name": "PreToolUse",
  "tool_name": "Bash",
  "tool_input": {
    "command": "npm install express",
    "description": "Install express package"
  }
}
```

### Output Format
Claude Hook Advisor responds with:

```json
{
  "decision": "block",
  "reason": "Command 'npm' is mapped to use 'bun' instead. Try: bun install express"
}
```

Or for allowed commands:
```json
{
  "decision": "allow"
}
```

## 🎮 Integration Examples

### Example 1: Node.js Project Setup

1. **Create project configuration**:
   ```toml
   # .claude-hook-advisor.toml
   [commands]
   npm = "bun"
   yarn = "bun"
   npx = "bunx"
   ```

2. **Set up Claude Code hook**:
   ```json
   {
     "hooks": {
       "PreToolUse": [
         {
           "matcher": "Bash",
           "hooks": [
             {
               "type": "command",
               "command": "claude-hook-advisor --hook"
             }
           ]
         }
       ]
     }
   }
   ```

3. **Test the integration**:
   - Ask Claude: "Install express using npm"
   - Claude tries: `npm install express`
   - Hook suggests: `bun install express`
   - Claude automatically retries with `bun install express`

### Example 2: Python Project Setup

1. **Create Python-focused configuration**:
   ```toml
   # .claude-hook-advisor.toml
   [commands]
   pip = "uv pip"
   "pip install" = "uv add"
   python = "uv run python"
   pytest = "uv run pytest"
   ```

2. **Same hook configuration** as above

3. **Test with Python commands**:
   - Ask Claude: "Install requests package"
   - Claude tries: `pip install requests`
   - Hook suggests: `uv add requests`
   - Claude automatically uses the faster tool

### Example 3: Multi-Tool Development Environment

1. **Comprehensive configuration**:
   ```toml
   # .claude-hook-advisor.toml
   [commands]
   # Package managers
   npm = "bun"
   pip = "uv pip"
   
   # Modern CLI tools
   cat = "bat"
   ls = "eza"
   grep = "rg"
   
   # Safety measures
   "rm -rf" = "echo 'Use trash command for safety'"
   
   # Git best practices
   "git commit" = "git commit -S"
   ```

2. **Hook handles all Bash commands** automatically

## 🔍 Testing Integration

### Manual Testing

1. **Test hook directly**:
   ```bash
   echo '{"session_id":"test","transcript_path":"","cwd":"","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook
   ```

2. **Expected output**:
   ```json
   {
     "decision": "block",
     "reason": "Command 'npm' is mapped to use 'bun' instead. Try: bun install"
   }
   ```

### Integration Testing

1. **Ask Claude to run a mapped command**:
   ```
   Please run: npm install express
   ```

2. **Observe the flow**:
   - Claude attempts: `npm install express`
   - Hook intercepts and suggests: `bun install express`
   - Claude automatically retries with: `bun install express`

3. **Check for success indicators**:
   - Command executes with suggested tool
   - No error messages about hook failures
   - Smooth conversation flow

## 🛠️ Advanced Configuration

### Custom Hook Paths

For non-standard installations:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "/custom/path/to/claude-hook-advisor --hook"
          }
        ]
      }
    ]
  }
}
```

### Multiple Hook Configurations

You can chain multiple hooks:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "claude-hook-advisor --hook"
          },
          {
            "type": "command",
            "command": "other-security-hook --check"
          }
        ]
      }
    ]
  }
}
```

### Conditional Hook Activation

Use environment variables for conditional activation:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "bash -c 'if [ \"$ENABLE_HOOK_ADVISOR\" = \"true\" ]; then claude-hook-advisor --hook; else cat; fi'"
          }
        ]
      }
    ]
  }
}
```

## 🐛 Troubleshooting Integration

### Hook Not Triggering

1. **Check hook configuration**:
   ```bash
   # Verify settings.json syntax
   cat .claude/settings.json | jq .
   ```

2. **Verify binary path**:
   ```bash
   which claude-hook-advisor
   # Should return the path to the binary
   ```

3. **Test hook manually**:
   ```bash
   echo '{"session_id":"test","transcript_path":"","cwd":"","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook
   ```

### Permission Issues

1. **Make binary executable**:
   ```bash
   chmod +x ~/.local/bin/claude-hook-advisor
   ```

2. **Check file permissions**:
   ```bash
   ls -la ~/.local/bin/claude-hook-advisor
   ```

### Configuration Not Loading

1. **Verify config file location**:
   ```bash
   ls -la .claude-hook-advisor.toml
   ```

2. **Test config syntax**:
   ```bash
   claude-hook-advisor --config .claude-hook-advisor.toml --hook < /dev/null
   ```

3. **Check working directory**:
   - Hook runs in the directory where Claude Code is opened
   - Ensure `.claude-hook-advisor.toml` is in that directory

### JSON Parsing Errors

1. **Check input format**:
   ```bash
   # Test with valid JSON
   echo '{"session_id":"test","transcript_path":"","cwd":"","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"test"}}' | claude-hook-advisor --hook
   ```

2. **Verify output format**:
   ```bash
   # Output should be valid JSON
   claude-hook-advisor --hook < test_input.json | jq .
   ```

## 📊 Monitoring Hook Activity

### Logging Hook Calls

Add logging to monitor hook activity:

```bash
# Create a wrapper script for logging
cat > ~/.local/bin/claude-hook-advisor-logged << 'EOF'
#!/bin/bash
echo "Hook called at $(date): $*" >> ~/.claude-hook-advisor.log
claude-hook-advisor "$@"
EOF

chmod +x ~/.local/bin/claude-hook-advisor-logged
```

Update your hook configuration:
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "claude-hook-advisor-logged --hook"
          }
        ]
      }
    ]
  }
}
```

### Performance Monitoring

Monitor hook performance:

```bash
# Time hook execution
time echo '{"session_id":"test","transcript_path":"","cwd":"","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook
```

## 🎯 Best Practices

1. **Use absolute paths** for hook commands to avoid PATH issues
2. **Test thoroughly** before deploying to team
3. **Keep configurations simple** to avoid maintenance overhead
4. **Monitor hook performance** to ensure fast response times
5. **Document team conventions** for hook usage
6. **Version control settings** for team consistency

---

**Next Steps:**
- [Explore configuration examples](examples.md)
- [Learn troubleshooting techniques](troubleshooting.md)
- [Review best practices](best-practices.md)

#claude-code #hooks #integration #setup #pretooluse