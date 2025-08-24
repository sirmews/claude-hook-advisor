---
title: "Configuration Guide"
description: "Complete reference for configuring Claude Hook Advisor"
tags: ["configuration", "toml", "commands", "mapping", "setup"]
---

# Configuration Guide

Claude Hook Advisor uses TOML configuration files to define command mappings and semantic directory aliases. This guide covers all configuration options and patterns for both command intelligence and directory aliasing features.

## 📁 Configuration File Location

The tool searches for configuration files in this order:

1. **Custom path** specified with `-c/--config` flag
2. **`.claude-hook-advisor.toml`** in current directory
3. **No configuration** (allows all commands)

```bash
# Use custom config file
claude-hook-advisor --config /path/to/config.toml --hook

# Use default location
claude-hook-advisor --hook  # Looks for .claude-hook-advisor.toml
```

## 🔧 Complete Configuration Format

### Full Configuration Structure
```toml
# Command mappings for intelligent replacement
[commands]
npm = "bun"
yarn = "bun"
curl = "wget --verbose"

# Semantic directory aliases for natural language references
[semantic_directories]
docs = "~/Documents/Documentation"
central_docs = "~/Documents/Documentation"
project_docs = "~/Documents/Documentation/{project}"
claude_docs = "~/Documents/Documentation/claude"

# Variables for dynamic path substitution
[directory_variables]
project = "my-project"          # Or auto-detected from git
current_project = "my-project"
user_home = "~"
```

### Simple Command Mapping
```toml
[commands]
# Replace 'npm' with 'bun'
npm = "bun"

# Replace 'yarn' with 'bun'
yarn = "bun"

# Replace 'curl' with 'wget --verbose'
curl = "wget --verbose"
```

### Exact Command Matching
```toml
[commands]
# Only replace exact 'pip install' command
"pip install" = "uv add"

# Only replace exact 'npm start' command
"npm start" = "bun dev"

# Replace 'git commit' with signed commits
"git commit" = "git commit -S"
```

## 🎯 Pattern Matching Rules

### Word Boundary Matching
Claude Hook Advisor uses word-boundary regex matching:

```toml
[commands]
npm = "bun"
```

**Matches:**
- `npm install` → `bun install`
- `npm run build` → `bun run build`
- `npm --version` → `bun --version`

**Does NOT match:**
- `npm-check` (no word boundary)
- `my-npm-tool` (npm is not at word boundary)

### Exact String Matching
For precise control, use quoted strings:

```toml
[commands]
"npm install" = "bun add"
"npm uninstall" = "bun remove"
"npm run" = "bun run"
```

**Matches:**
- `npm install package` → `bun add package`
- `npm install` → `bun add`

**Does NOT match:**
- `npm install-something` (not exact match)
- `npm` alone (doesn't include "install")

## 📁 Directory Aliasing Configuration

### Basic Directory Aliases
```toml
[semantic_directories]
# Simple directory aliases
docs = "~/Documents/Documentation"
notes = "~/Documents/Notes"
projects = "~/Projects"
```

### Variable Substitution in Paths
```toml
[semantic_directories]
# Use variables for dynamic paths
project_docs = "~/Documents/Documentation/{project}"
project_notes = "~/Notes/{project}"
user_config = "{user_home}/.config"

[directory_variables]
# Define variables used in paths
project = "my-awesome-project"      # Or auto-detected from git repo name
current_project = "my-awesome-project"
user_home = "/Users/username"       # Or from $HOME environment variable
```

### Common Directory Patterns
```toml
[semantic_directories]
# Documentation locations
docs = "~/Documents/Documentation"
central_docs = "~/Documents/Documentation"
project_docs = "~/Documents/Documentation/{project}"
claude_docs = "~/Documents/Documentation/claude"

# Development directories
src = "./src"
lib = "./lib"
tests = "./tests"
build = "./build"
dist = "./dist"

# Configuration directories
config = "~/.config"
local_config = "./.config"
project_config = "{user_home}/.config/{project}"

# Temporary and cache directories
tmp = "/tmp"
cache = "~/.cache"
project_cache = "~/.cache/{project}"
```

### Directory Alias Management via CLI
```bash
# Add directory aliases
claude-hook-advisor --add-directory-alias "docs" "~/Documents/Documentation"
claude-hook-advisor --add-directory-alias "project_docs" "~/Documents/Documentation/{project}"

# List all configured aliases
claude-hook-advisor --list-directory-aliases

# Resolve alias to canonical path
claude-hook-advisor --resolve-directory "docs"

# Remove alias
claude-hook-advisor --remove-directory-alias "docs"
```

### Variable Detection and Substitution
The tool automatically detects these variables:

1. **Project Detection**: 
   - Automatically detects git repository name as `{project}`
   - Falls back to configured `directory_variables.project`
   
2. **Home Directory**:
   - Uses `$HOME` environment variable
   - Falls back to configured `directory_variables.user_home`

3. **Custom Variables**:
   - Define your own variables in `[directory_variables]`
   - Use them in directory templates with `{variable_name}`

## 📚 Configuration Categories

### Package Managers

#### Node.js Ecosystem
```toml
[commands]
# Bun as primary package manager
npm = "bun"
yarn = "bun"
pnpm = "bun"
npx = "bunx"

# Specific command mappings
"npm install" = "bun add"
"npm uninstall" = "bun remove"
"npm run" = "bun run"
"npm start" = "bun dev"
"npm test" = "bun test"
```

#### Python Ecosystem
```toml
[commands]
# UV for faster Python package management
pip = "uv pip"
"pip install" = "uv add"
"pip uninstall" = "uv remove"
"pip freeze" = "uv pip freeze"
python = "uv run python"
pytest = "uv run pytest"
```

#### Other Languages
```toml
[commands]
# Ruby
gem = "bundle exec gem"
"gem install" = "bundle add"

# Go
"go get" = "go mod tidy && go get"
"go run" = "go run -race"

# Rust
"cargo install" = "cargo binstall"
```

### Modern CLI Tools

#### File Operations
```toml
[commands]
# Better file listing and viewing
ls = "eza"
cat = "bat"
less = "bat --paging=always"

# Safer file operations
rm = "trash"
cp = "cp -i"
mv = "mv -i"

# Better file search
find = "fd"
locate = "fd"
```

#### Text Processing
```toml
[commands]
# Modern text tools
grep = "rg"
awk = "rg --replace"
sed = "sd"

# Better diff tools
diff = "delta"
```

#### System Monitoring
```toml
[commands]
# Process and system monitoring
top = "htop"
ps = "procs"
du = "dust"
df = "duf"

# Network tools
ping = "gping"
netstat = "ss"
```

### Development Tools

#### Version Control
```toml
[commands]
# Git best practices
"git push" = "git push --set-upstream origin HEAD"
"git commit" = "git commit -S"
"git pull" = "git pull --rebase"
"git merge" = "git merge --no-ff"

# Git shortcuts
"git status" = "git status --short --branch"
"git log" = "git log --oneline --graph"
```

#### Build Tools
```toml
[commands]
# Modern build systems
make = "just"
cmake = "meson"
autotools = "meson"

# Container tools
docker = "podman"
"docker-compose" = "podman-compose"
```

#### Editors and IDEs
```toml
[commands]
# Modern editors
vim = "nvim"
nano = "micro"
emacs = "doom emacs"
```

### Security and Safety

#### Dangerous Command Prevention
```toml
[commands]
# Prevent destructive operations
"rm -rf" = "echo 'Use trash command instead of rm -rf for safety'"
"sudo rm" = "echo 'Consider using trash or be very careful with sudo rm'"
"chmod 777" = "echo 'chmod 777 is dangerous, consider more restrictive permissions'"
"chown -R" = "echo 'Recursive chown can be dangerous, consider specific paths'"

# Encourage secure practices
ssh = "ssh -o VerifyHostKeyDNS=yes"
scp = "rsync -avz --progress"
```

#### Network Security
```toml
[commands]
# Secure HTTP tools
curl = "curl --fail --location --show-error"
wget = "wget --secure-protocol=TLSv1_2"

# VPN and tunneling
ssh = "ssh -C -o Compression=yes"
```

## 🔄 Advanced Configuration Patterns

### Environment-Specific Commands
```toml
[commands]
# Development vs Production
"npm start" = "bun dev"
"npm run build" = "bun run build:prod"

# Different tools for different contexts
kubectl = "k9s"
terraform = "tofu"
aws = "aws --cli-auto-prompt"
```

### Complex Command Transformations
```toml
[commands]
# Add safety flags
"python -m http.server" = "python -m http.server --bind 127.0.0.1"
"php -S" = "php -S 127.0.0.1:8000"

# Add useful options
"git diff" = "git diff --color-words"
"grep -r" = "rg --hidden"
```

### Project-Specific Shortcuts
```toml
[commands]
# Custom project commands
"npm test" = "bun test --watch"
"npm lint" = "bun run lint:fix"
"npm format" = "bun run format:write"

# Database shortcuts
"psql" = "psql -h localhost -U postgres"
"mysql" = "mysql -h localhost -u root -p"
```

## 🎨 Configuration Templates

### Frontend Development
```toml
[commands]
# Package management
npm = "bun"
yarn = "bun"
npx = "bunx"

# Development servers
"npm start" = "bun dev"
"npm run dev" = "bun dev"

# Build and deployment
"npm run build" = "bun run build:prod"
"npm run preview" = "bun run preview"

# Testing
"npm test" = "bun test --watch"
"npm run test:ci" = "bun test --coverage"
```

### Backend Development
```toml
[commands]
# Python tools
pip = "uv pip"
python = "uv run python"
pytest = "uv run pytest"

# Database tools
psql = "psql -h localhost -U postgres"
redis-cli = "redis-cli -h localhost"

# Container tools
docker = "podman"
"docker-compose" = "podman-compose"
```

### DevOps/Infrastructure
```toml
[commands]
# Cloud tools
aws = "aws --cli-auto-prompt"
gcloud = "gcloud --verbosity=info"
az = "az --output table"

# Kubernetes
kubectl = "k9s"
helm = "helm --debug"

# Infrastructure as Code
terraform = "tofu"
ansible = "ansible --diff"
```

## ✅ Configuration Validation

### Testing Your Configuration
```bash
# Test specific command mapping
echo '{"session_id":"test","transcript_path":"","cwd":"","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook

# Expected output for npm -> bun mapping:
# {"decision":"block","reason":"Command 'npm' is mapped to use 'bun' instead. Try: bun install"}
```

### Common Configuration Errors

#### Invalid TOML Syntax
```toml
# ❌ Wrong: Missing quotes for keys with spaces
[commands]
git commit = "git commit -S"  # Error: space in key

# ✅ Correct: Quote keys with spaces
[commands]
"git commit" = "git commit -S"
```

#### Circular Mappings
```toml
# ❌ Wrong: Creates infinite loop
[commands]
npm = "yarn"
yarn = "npm"

# ✅ Correct: Map to final tool
[commands]
npm = "bun"
yarn = "bun"
```

## 🔧 Configuration Management

### Multiple Configuration Files
```bash
# Project-specific config
.claude-hook-advisor.toml

# Team-shared config
team.claude-hook-advisor.toml

# Personal overrides
personal.claude-hook-advisor.toml
```

### Configuration Inheritance
While not built-in, you can manage multiple configs:

```bash
# Use team config as base
cp team.claude-hook-advisor.toml .claude-hook-advisor.toml

# Add personal customizations
echo '
[commands]
# Personal preferences
vim = "nvim"
' >> .claude-hook-advisor.toml
```

### Version Control
```bash
# Include in version control for team sharing
git add .claude-hook-advisor.toml

# Or keep personal configs out of version control
echo '.claude-hook-advisor.toml' >> .gitignore
```

## 🎯 Best Practices

1. **Start Simple**: Begin with basic mappings and add complexity gradually
2. **Test Thoroughly**: Verify each mapping works as expected
3. **Document Choices**: Comment your configuration for team members
4. **Use Consistent Patterns**: Establish team conventions for mappings
5. **Regular Updates**: Review and update configurations as tools evolve

---

**Next Steps:**
- [Explore example configurations](examples.md)
- [Learn best practices](best-practices.md)
- [Set up Claude Code integration](claude-integration.md)

#configuration #toml #commands #mapping #setup