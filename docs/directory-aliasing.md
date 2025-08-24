---
title: "Directory Aliasing Guide"
description: "Complete guide to semantic directory aliasing with Claude Hook Advisor"
tags: ["directory-aliasing", "semantic-directories", "path-resolution", "userpromptsub mit"]
---

# Directory Aliasing Guide

Directory aliasing allows you to use natural language directory references in your conversations with Claude Code. Instead of typing full paths, use semantic names like "docs", "project_docs", or "central_docs" and let Claude Hook Advisor automatically resolve them to canonical filesystem paths.

**Current Implementation (v0.2.0):** Simple static alias-to-path mapping with tilde (~) expansion. No variable substitution or dynamic path generation.

## 🎯 Overview

### What is Directory Aliasing?
Directory aliasing creates mappings between short, memorable names and actual filesystem paths. When you say *"check the docs directory"*, Claude Code automatically understands you mean `/Users/you/Documents/Documentation`.

### Key Benefits
- **Natural language**: Use semantic names in conversations
- **Path abstraction**: Hide complex directory structures  
- **Tilde expansion**: Automatic expansion of ~ to user home directory
- **Cross-platform**: Abstract away OS-specific path differences
- **Team consistency**: Shared directory references across team members

## 🔧 Quick Setup

### 1. Install Hooks
```bash
# Install all hooks including UserPromptSubmit for directory detection
claude-hook-advisor --install
```

### 2. Configure Directory Aliases
Edit your `.claude-hook-advisor.toml` file:

```toml
[semantic_directories]
docs = "~/Documents/Documentation"
project_docs = "~/Documents/Documentation/my-project"
central_docs = "~/Documents/Documentation"
```

### 3. Use in Conversations
Now you can use natural language:
- *"Please check the docs directory for installation instructions"*
- *"Create a new file in project_docs"*
- *"List files in central_docs"*

## 📁 Configuration Format

### Basic Configuration
```toml
[semantic_directories]
docs = "~/Documents/Documentation"
notes = "~/Documents/Notes"
projects = "~/Projects"
tmp = "/tmp"
```

### Project-Specific Paths
```toml
[semantic_directories]
project_docs = "~/Documents/Documentation/my-project"
project_notes = "~/Notes/my-project"
user_config = "~/.config/my-project"
```

**Note:** Each alias maps directly to a static path. The tilde (~) character is automatically expanded to your home directory.

## 🚀 How It Works

### UserPromptSubmit Hook Integration
When you type a message in Claude Code that contains directory references:

1. **Text Analysis**: The UserPromptSubmit hook scans your prompt for configured directory aliases
2. **Pattern Recognition**: Uses word-boundary regex to detect exact alias matches  
3. **Path Resolution**: Converts aliases to absolute canonical filesystem paths
4. **Security Validation**: Performs path canonicalization to prevent traversal attacks
5. **Claude Integration**: Outputs resolved paths as hook messages for Claude to use

### Example Flow
```
User: "check the docs directory"
    ↓
Hook detects: "docs"
    ↓  
Resolves to: "~/Documents/Documentation"
    ↓
Expands to: "/Users/you/Documents/Documentation"
    ↓
Security check: Canonicalizes path
    ↓
Claude receives: "Directory reference 'docs' resolved to: /Users/you/Documents/Documentation"
```

## 💡 Example Configurations

### Documentation Workflow
```toml
[semantic_directories]
# Documentation hierarchy
docs = "~/Documents/Documentation"
central_docs = "~/Documents/Documentation"
project_docs = "~/Documents/Documentation/my-project"
claude_docs = "~/Documents/Documentation/claude"
api_docs = "~/Documents/Documentation/my-project/api"

# Development directories
src = "./src"
tests = "./tests"
build = "./target"

# Configuration locations
config = "~/.config"
project_config = "~/.config/my-project"
local_config = "./.config"
```

## 🔍 Testing Your Configuration

### Command Line Testing
```bash
# Test directory resolution with example prompt
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check the docs directory"}' | claude-hook-advisor --hook

# Expected output:
# Directory reference 'docs' resolved to: /Users/you/Documents/Documentation
```

### Debugging Tips
1. **Check configuration file**: Ensure `.claude-hook-advisor.toml` exists and has correct syntax
2. **Verify paths exist**: The directories must exist on your filesystem for resolution to work
3. **Test word boundaries**: "docs" will match but "documentation" won't (unless specifically configured)
4. **Check file permissions**: Ensure the hook can read your configuration file

## 🛡️ Security Features

### Path Canonicalization
All resolved paths go through filesystem canonicalization:
- Resolves symlinks to their actual targets
- Removes `../` and `./` components
- Prevents directory traversal attacks
- Ensures paths exist before resolution

### Trusted Directory Validation
The system only resolves to directories that:
- Are explicitly configured in your TOML file
- Exist on the filesystem
- Pass canonicalization checks

## ⚙️ Configuration Management

### File Location
The tool looks for configuration in this order:
1. Custom path specified with `-c/--config` flag
2. `.claude-hook-advisor.toml` in current directory  
3. If no config found, no directory aliases are available

### TOML Syntax
```toml
# Comments start with #
[semantic_directories]
alias_name = "path/to/directory"  # Use quotes for paths
another_alias = "~/relative/to/home"
```

### Best Practices
1. **Use absolute paths**: Start with `~` or `/` for reliability
2. **Consistent naming**: Use clear, descriptive alias names
3. **Team sharing**: Commit `.claude-hook-advisor.toml` to version control
4. **Path validation**: Ensure all configured paths actually exist

## 🔧 Troubleshooting

### Directory Not Resolved
**Problem:** "docs" doesn't resolve to the expected path

**Solutions:**
1. Check configuration file exists: `ls .claude-hook-advisor.toml`
2. Verify alias configuration in TOML file
3. Ensure the target directory exists on filesystem
4. Test with simple echo command shown above

### Hook Not Triggering
**Problem:** No directory resolution messages appear

**Solutions:**
1. Verify UserPromptSubmit hook is installed in Claude Code settings
2. Check that `claude-hook-advisor` is in your PATH
3. Test hook manually with echo command

### Path Doesn't Exist
**Problem:** Hook fails to resolve path

**Solutions:**
1. Create the target directory: `mkdir -p ~/Documents/Documentation`  
2. Fix typos in configuration file
3. Use absolute paths starting with `~` or `/`

## 📊 Current Limitations

### What's NOT Supported (v0.2.0)
- **Variable substitution**: No `{project}` or `{user_home}` variables
- **Dynamic paths**: All paths are static mappings
- **CLI management**: No `--add-directory-alias` commands  
- **Environment variables**: No `$HOME` or `$USER` expansion beyond tilde

### What IS Supported
- **Static alias mapping**: Direct alias-to-path relationships
- **Tilde expansion**: `~` automatically becomes home directory
- **Word-boundary detection**: Exact alias matching in natural language
- **Path canonicalization**: Security and symlink resolution
- **TOML configuration**: Simple file-based setup

---

*Last updated: August 2025 for claude-hook-advisor v0.2.0*