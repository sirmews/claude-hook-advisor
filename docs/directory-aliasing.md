---
title: "Directory Aliasing Guide"
description: "Complete guide to semantic directory aliasing with Claude Hook Advisor"
tags: ["directory-aliasing", "semantic-directories", "path-resolution", "variables", "userpromptsub mit"]
---

# Directory Aliasing Guide

Directory aliasing allows you to use natural language directory references in your conversations with Claude Code. Instead of typing full paths, use semantic names like "docs", "project_docs", or "central_docs" and let Claude Hook Advisor automatically resolve them to canonical filesystem paths.

## 🎯 Overview

### What is Directory Aliasing?
Directory aliasing creates mappings between short, memorable names and actual filesystem paths. When you say *"check the docs directory"*, Claude Code automatically understands you mean `/Users/you/Documents/Documentation`.

### Key Benefits
- **Natural language**: Use semantic names in conversations
- **Path abstraction**: Hide complex directory structures  
- **Variable substitution**: Dynamic paths with `{project}`, `{user_home}`
- **Cross-platform**: Abstract away OS-specific path differences
- **Team consistency**: Shared directory references across team members

## 🔧 Quick Setup

### 1. Install Hooks
```bash
# Install all hooks including UserPromptSubmit for directory detection
claude-hook-advisor --install-hooks
```

### 2. Add Directory Aliases
```bash
# Add semantic directory aliases
claude-hook-advisor --add-directory-alias "docs" "~/Documents/Documentation"
claude-hook-advisor --add-directory-alias "project_docs" "~/Documents/Documentation/{project}"
claude-hook-advisor --add-directory-alias "central_docs" "~/Documents/Documentation"

# List configured aliases
claude-hook-advisor --list-directory-aliases
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

### With Variable Substitution
```toml
[semantic_directories]
project_docs = "~/Documents/Documentation/{project}"
project_notes = "~/Notes/{project}"
user_config = "{user_home}/.config/{project}"

[directory_variables]
project = "my-awesome-project"      # Auto-detected from git or configured
user_home = "~"                     # From $HOME or configured
```

## 🎮 Usage Examples

### Basic Usage
```bash
# Configure an alias
claude-hook-advisor --add-directory-alias "docs" "~/Documents/Documentation"

# Use in Claude Code conversation
# You: "Please list files in the docs directory"
# Claude automatically uses: /Users/you/Documents/Documentation
```

### Variable Substitution Examples
```bash
# Set up project-specific docs
claude-hook-advisor --add-directory-alias "project_docs" "~/Documents/Documentation/{project}"

# In project "my-app", this resolves to:
claude-hook-advisor --resolve-directory "project_docs"
# Output: /Users/you/Documents/Documentation/my-app
```

### Common Patterns
```toml
[semantic_directories]
# Documentation hierarchy
docs = "~/Documents/Documentation"
central_docs = "~/Documents/Documentation"
project_docs = "~/Documents/Documentation/{project}"
claude_docs = "~/Documents/Documentation/claude"
api_docs = "~/Documents/Documentation/{project}/api"

# Development directories
src = "./src"
lib = "./lib"
tests = "./tests"
build = "./build"
dist = "./dist"

# Configuration locations
config = "~/.config"
project_config = "~/.config/{project}"
local_config = "./.config"
```

## 🔄 Variable Substitution

### Automatic Variables
Claude Hook Advisor automatically detects:

1. **{project}** / **{current_project}**:
   - Auto-detected from git repository name
   - Fallback to manually configured value
   - Example: In repo "my-app" → `{project}` = "my-app"

2. **{user_home}**:
   - From `$HOME` environment variable  
   - Fallback to manually configured value
   - Example: `{user_home}` = "/Users/username"

### Manual Variable Configuration
```toml
[directory_variables]
# Override auto-detection
project = "custom-project-name"
current_project = "custom-project-name"
user_home = "/custom/home/path"

# Custom variables
team_shared = "/shared/team/directory"
backup_location = "/backups/{project}"
```

### Variable Usage in Paths
```toml
[semantic_directories]
# Simple variable usage
project_docs = "~/Documents/{project}"
user_config = "{user_home}/.config"

# Complex paths with multiple variables
project_backup = "{backup_location}/{project}/backup"
team_project = "{team_shared}/{project}/workspace"
```

## 🛠️ CLI Management

### Adding Aliases
```bash
# Basic alias
claude-hook-advisor --add-directory-alias "docs" "~/Documents/Documentation"

# With variables
claude-hook-advisor --add-directory-alias "project_docs" "~/Documents/Documentation/{project}"

# Complex paths
claude-hook-advisor --add-directory-alias "api_docs" "~/Documents/Documentation/{project}/api"
```

### Listing and Inspecting
```bash
# List all configured aliases
claude-hook-advisor --list-directory-aliases

# Resolve specific alias to see canonical path
claude-hook-advisor --resolve-directory "docs"
claude-hook-advisor --resolve-directory "project_docs"
```

### Removing Aliases
```bash
# Remove specific alias
claude-hook-advisor --remove-directory-alias "docs"

# Aliases are also stored in .claude-hook-advisor.toml and can be edited directly
```

## 🔍 How Directory Detection Works

### UserPromptSubmit Hook
1. **Text Analysis**: Scans user prompts for potential directory references
2. **Pattern Matching**: Uses regex to detect configured aliases
3. **Path Resolution**: Resolves variables and expands paths
4. **Security Validation**: Canonicalizes paths to prevent traversal attacks
5. **Claude Integration**: Provides resolved path information to Claude Code

### Detection Examples
```
User: "Please check the docs directory for examples"
Hook detects: "docs" 
Resolves to: "/Users/you/Documents/Documentation"
Claude receives: "Directory reference detected: 'docs' -> '/Users/you/Documents/Documentation'"

User: "Create a file in project_docs called README.md"  
Hook detects: "project_docs"
Resolves to: "/Users/you/Documents/Documentation/my-project"
Claude uses the canonical path automatically
```

## 🔐 Security Features

### Path Canonicalization
- All paths are canonicalized to prevent directory traversal attacks
- Symbolic links are resolved to their targets
- Relative paths are converted to absolute paths

### Safety Checks
```rust
// Example of security validation (internal)
let canonical_path = fs::canonicalize(&expanded_path)?;
// This prevents attacks like:
// docs = "~/../../etc/passwd"
```

### Trusted Directories Only
- Only configured aliases are resolved
- No arbitrary path resolution from user input
- Variables are substituted from trusted configuration

## 📊 Advanced Usage

### Team Collaboration
```toml
# Team-shared configuration
[semantic_directories]
# Shared documentation location
team_docs = "/shared/team/documentation"
project_specs = "/shared/team/specifications/{project}"

# Individual workspace
my_workspace = "~/workspace/{project}"
my_notes = "~/Notes/{project}"

[directory_variables]
# Consistent across team members
project = "team-project"
```

### Cross-Platform Support
```toml
[semantic_directories]
# Use ~ for home directory abstraction
docs = "~/Documents/Documentation"     # Works on macOS/Linux
config = "~/.config"                   # Unix-style config dir

# Variable substitution for platform differences
user_data = "{user_home}/AppData/Roaming/{project}"  # Windows
user_data = "{user_home}/.local/share/{project}"     # Linux
```

### Development Workflow Integration
```toml
[semantic_directories]
# Project structure
src = "./src"
tests = "./tests"  
build = "./build"
docs = "./docs"

# External dependencies
node_modules = "./node_modules"
vendor = "./vendor"

# Development tools
logs = "./logs"
tmp = "./tmp"
cache = "./.cache"
```

## 🐛 Troubleshooting

### Alias Not Detected
1. **Check configuration**:
   ```bash
   claude-hook-advisor --list-directory-aliases
   ```

2. **Test resolution**:
   ```bash
   claude-hook-advisor --resolve-directory "your-alias"
   ```

3. **Verify hook installation**:
   ```bash
   # Should show UserPromptSubmit hook
   cat .claude/settings.json | grep -A5 UserPromptSubmit
   ```

### Path Resolution Errors
1. **Check path exists**:
   ```bash
   # Ensure the resolved path actually exists
   ls -la "$(claude-hook-advisor --resolve-directory "docs")"
   ```

2. **Verify permissions**:
   ```bash
   # Check read permissions on the directory
   ls -ld "$(claude-hook-advisor --resolve-directory "docs")"
   ```

3. **Variable substitution issues**:
   ```bash
   # Check if variables are being substituted correctly
   claude-hook-advisor --resolve-directory "project_docs"
   ```

### Hook Not Triggering
1. **Manual test**:
   ```bash
   echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check the docs directory"}' | claude-hook-advisor --hook
   ```

2. **Check output format**:
   ```bash
   # Should return plain text, not JSON
   # Expected: "Directory reference detected: 'docs' -> '/path/to/docs'"
   ```

## ✅ Best Practices

### Naming Conventions
1. **Use clear, semantic names**: `docs`, `project_docs`, `api_docs`
2. **Avoid abbreviations**: `documentation` vs `docs` (choose one consistently)
3. **Use underscores**: `project_docs` vs `project-docs` (underscores recommended)
4. **Hierarchical naming**: `docs`, `project_docs`, `api_docs` (show relationships)

### Path Organization
1. **Consistent base paths**: Use common root directories
2. **Variable usage**: Use `{project}` for project-specific paths
3. **Absolute paths**: Prefer absolute over relative paths for reliability
4. **Cross-platform**: Use `~` and variables for portability

### Team Management
1. **Shared configuration**: Keep common aliases in version control
2. **Personal overrides**: Use local config for personal preferences  
3. **Documentation**: Document team conventions for directory aliases
4. **Regular review**: Update aliases as project structure evolves

### Security Considerations
1. **Trusted paths only**: Only configure aliases for trusted directories
2. **No sensitive locations**: Avoid aliases to system or security-sensitive directories
3. **Review regularly**: Audit configured aliases periodically
4. **Path validation**: Rely on built-in canonicalization for security

---

**Next Steps:**
- [Learn about command mappings](configuration.md)
- [Set up Claude Code integration](claude-integration.md) 
- [Explore configuration examples](examples.md)

#directory-aliasing #semantic-directories #path-resolution #variables #userpromptsub mit #claude-code #productivity

---

*Last updated: 2025-08-06*
*Revision: 1.0*  
*Tags: #claude-code #directory-aliasing #semantic-directories #path-resolution #variables #productivity*