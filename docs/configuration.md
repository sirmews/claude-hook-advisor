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
"central docs" = "~/Documents/Documentation"
"project docs" = "~/Documents/Documentation/my-project"
"claude docs" = "~/Documents/Documentation/claude"
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
"central docs" = "~/Documents/Documentation"
notes = "~/Documents/Notes"
projects = "~/Projects"
```

### Static Path Mapping
```toml
[semantic_directories]
# Direct alias-to-path mapping (no variable substitution)
project_docs = "~/Documents/Documentation/my-awesome-project"
project_notes = "~/Notes/my-awesome-project"
user_config = "~/.config/my-project"
```

### Natural Language Aliases (Space-Separated)
```toml
[semantic_directories]
# Use quoted keys for multi-word, natural language aliases
"project docs" = "~/Documents/Documentation/my-awesome-project"
"project notes" = "~/Notes/my-awesome-project"
"user config" = "~/.config/my-project"
"claude docs" = "~/Documents/Documentation/claude"
```

**Advantage:** Space-separated aliases feel more natural in conversation:
- *"Check the project docs directory"* → matches `"project docs"`
- *"Look in user config folder"* → matches `"user config"`

### Alias Precedence and Conflicts
```toml
[semantic_directories]
# ⚠️ Problematic: overlapping aliases
docs = "~/Documents/Documentation"           # Shorter alias
"project docs" = "~/Documents/Documentation/project"  # Contains "docs"

# ✅ Better: avoid overlapping aliases
"central docs" = "~/Documents/Documentation"
"project docs" = "~/Documents/Documentation/project"
```

**Important:** When multiple aliases could match the same text, shorter/simpler aliases take precedence. Avoid configuring both `docs` and `"project docs"` if you want `"project docs"` to match phrases like "check project docs directory".

**Note:** In v0.2.0, directory paths are static. No variable substitution is supported.

### Common Directory Patterns
```toml
[semantic_directories]
# Documentation locations
"central docs" = "~/Documents/Documentation"
"project docs" = "~/Documents/Documentation/my-project"
"claude docs" = "~/Documents/Documentation/claude"

# Development directories
src = "./src"
lib = "./lib"
tests = "./tests"
build = "./build"
dist = "./dist"

# Configuration directories
config = "~/.config"
local_config = "./.config"
project_config = "~/.config/my-project"

# Temporary and cache directories
tmp = "/tmp"
cache = "~/.cache"
project_cache = "~/.cache/my-project"
```

### Directory Configuration via TOML File Only
```bash
# Edit configuration file directly
echo '[semantic_directories]
"central docs" = "~/Documents/Documentation"
"project docs" = "~/Documents/Documentation/my-project"' > .claude-hook-advisor.toml

# Test directory resolution via hook
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs"}' | claude-hook-advisor --hook
```

**Note:** No CLI commands for directory alias management in v0.2.0. Use TOML configuration only.

### Path Expansion (v0.2.0)
The tool supports basic path expansion:

1. **Tilde Expansion**: 
   - `~` is automatically expanded to user home directory
   - Example: `~/Documents` becomes `/Users/username/Documents`
   
2. **Static Paths Only**:
   - No variable substitution or dynamic path generation
   - Each alias maps directly to a fixed path

3. **Path Canonicalization**:
   - All paths are resolved to canonical absolute paths
   - Provides security against directory traversal attacks

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

## 🔒 Security Pattern Configuration

### Overview

Claude Hook Advisor includes **27 built-in security patterns** that are **enabled by default**. These patterns detect dangerous code patterns when Claude edits files, covering vulnerabilities across 10+ programming languages.

### How Security Patterns Work

Security patterns check two things:
1. **File paths** using glob patterns (e.g., `.github/workflows/*.yml`)
2. **File content** using substring matching (e.g., `eval(`, `pickle.loads`)

When a pattern matches, the operation is **blocked** and Claude receives a security warning explaining the risk and suggesting safer alternatives.

### Default Behavior (No Configuration Needed)

All 27 security patterns are enabled by default. You don't need any configuration - they work out of the box:

```toml
# No configuration needed! Security patterns are automatically enabled.
```

### Disabling Patterns

If a pattern is too noisy for your workflow, disable it by setting it to `false`:

```toml
[security_pattern_overrides]
# Disable Swift force unwrap warnings (common in Swift projects)
swift_force_unwrap = false

# Disable eval warnings (if building a REPL/interpreter)
eval_injection = false
python_eval = false
ruby_eval = false

# Disable unsafe warnings (if doing low-level systems programming)
rust_unsafe_block = false
```

### Built-in Security Patterns Reference

#### JavaScript / TypeScript (7 patterns)

```toml
[security_pattern_overrides]
eval_injection = false                  # Detects eval() usage
new_function_injection = false          # Detects new Function() usage
innerHTML_xss = false                   # Detects innerHTML assignment
react_dangerously_set_html = false      # Detects dangerouslySetInnerHTML
document_write_xss = false              # Detects document.write()
child_process_exec = false              # Detects exec()/execSync()
```

**What they detect:**
- `eval()` - Arbitrary code execution
- `new Function()` - Dynamic code generation
- `.innerHTML =` - XSS vulnerabilities
- `dangerouslySetInnerHTML` - React XSS risks
- `document.write()` - XSS attacks and performance issues
- `child_process.exec` - Command injection via shell

#### Python (4 patterns)

```toml
[security_pattern_overrides]
python_eval = false                     # Detects eval() usage
python_exec = false                     # Detects exec() usage
pickle_deserialization = false          # Detects pickle.load/loads
os_system_injection = false             # Detects os.system() usage
```

**What they detect:**
- `eval()` - Arbitrary code execution
- `exec()` - Arbitrary code execution
- `pickle.load()` / `pickle.loads()` - Unsafe deserialization
- `os.system()` - Command injection

#### SQL (2 patterns)

```toml
[security_pattern_overrides]
sql_injection = false                   # Detects SQL string interpolation
sql_string_format = false               # Detects format() in SQL queries
```

**What they detect:**
- `execute(f"SELECT...")` - String interpolation in SQL
- `query(format!(...))` - Rust format! in SQL queries

#### Rust (2 patterns)

```toml
[security_pattern_overrides]
rust_unsafe_block = false               # Detects unsafe {} blocks
rust_command_injection = false          # Detects shell command usage
```

**What they detect:**
- `unsafe {}` - Bypasses Rust's safety guarantees
- `Command::new("sh")` - Shell command injection risks

#### Go (2 patterns)

```toml
[security_pattern_overrides]
go_command_injection = false            # Detects shell command usage
go_sql_injection = false                # Detects fmt.Sprintf in SQL
```

**What they detect:**
- `exec.Command("sh", ...)` - Command injection
- `db.Query(fmt.Sprintf(...))` - SQL injection

#### Swift (3 patterns)

```toml
[security_pattern_overrides]
swift_force_unwrap = false              # Detects ! force unwrap
swift_unsafe_operations = false         # Detects unsafe pointers
swift_nspredicate_format = false        # Detects NSPredicate injection
```

**What they detect:**
- `optional!` - Force unwrap that can crash
- `UnsafeMutablePointer` - Memory safety bypasses
- `NSPredicate(format:...)` - Injection vulnerabilities

#### Java (2 patterns)

```toml
[security_pattern_overrides]
java_runtime_exec = false               # Detects Runtime.exec()
java_deserialization = false            # Detects ObjectInputStream
```

**What they detect:**
- `Runtime.getRuntime().exec()` - Command injection
- `ObjectInputStream` / `readObject()` - Unsafe deserialization

#### PHP (2 patterns)

```toml
[security_pattern_overrides]
php_eval = false                        # Detects eval() usage
php_unserialize = false                 # Detects unserialize() usage
```

**What they detect:**
- `eval()` - Arbitrary code execution
- `unserialize()` - Object injection attacks

#### Ruby (2 patterns)

```toml
[security_pattern_overrides]
ruby_eval = false                       # Detects eval/instance_eval/class_eval
ruby_yaml_load = false                  # Detects YAML.load usage
```

**What they detect:**
- `eval()` / `instance_eval()` / `class_eval()` - Code execution
- `YAML.load()` - Arbitrary code execution (use `YAML.safe_load`)

#### GitHub Actions (2 patterns)

```toml
[security_pattern_overrides]
github_actions_workflow = false         # Detects .yml workflow files
github_actions_workflow_yaml = false    # Detects .yaml workflow files
```

**What they detect:**
- `.github/workflows/*.yml` files - Workflow injection risks
- `.github/workflows/*.yaml` files - Workflow injection risks

### Configuration Examples by Use Case

#### Web Development (React/Node.js)
```toml
[security_pattern_overrides]
# Keep most security warnings enabled
# Only disable if you have specific needs
```

Most web developers should keep all patterns enabled for maximum security.

#### Systems Programming (Rust/C++)
```toml
[security_pattern_overrides]
# Disable unsafe warnings if working on low-level code
rust_unsafe_block = false
```

#### Swift/iOS Development
```toml
[security_pattern_overrides]
# Force unwrap is common in Swift, might be too noisy
swift_force_unwrap = false
```

#### Building Developer Tools (REPL, Interpreters)
```toml
[security_pattern_overrides]
# Disable eval warnings if building tools that need dynamic code
eval_injection = false
python_eval = false
ruby_eval = false
new_function_injection = false
```

#### Data Science / Jupyter Notebooks
```toml
[security_pattern_overrides]
# pickle is common in ML/data science workflows
pickle_deserialization = false
```

### Pattern State Management

Security warnings are **session-scoped**:
- Each warning is shown **once per session** per file+pattern combination
- State is tracked in `~/.claude/security_warnings_state_{session_id}.json`
- When a session ends, you'll see warnings again in new sessions
- State files are automatically cleaned up after 30 days

### How Warnings Appear

When Claude tries to write dangerous code, you'll see:

```
⚠️ Security Warning: eval() executes arbitrary code and is a major security risk.

Consider using JSON.parse() for data parsing or alternative design patterns that
don't require code evaluation. Only use eval() if you truly need to evaluate
arbitrary code.
```

Claude will then:
- Look for a safer alternative
- Ask if you want to proceed anyway
- Explain why the dangerous pattern might be necessary

### Testing Security Patterns

You can test security pattern detection:

```bash
# Create a test file with dangerous code
echo 'const data = eval(userInput)' > test.js

# Simulate Claude editing the file
echo '{
  "session_id": "test",
  "hook_event_name": "PreToolUse",
  "tool_name": "Write",
  "tool_input": {
    "file_path": "test.js",
    "content": "const data = eval(userInput)"
  }
}' | claude-hook-advisor --hook

# You should see the security warning
```

### Viewing All Pattern Names

All 27 pattern names for the `[security_pattern_overrides]` section:

```
github_actions_workflow          github_actions_workflow_yaml
eval_injection                   new_function_injection
react_dangerously_set_html       document_write_xss
innerHTML_xss                    child_process_exec
pickle_deserialization           os_system_injection
python_eval                      python_exec
sql_injection                    sql_string_format
rust_unsafe_block                rust_command_injection
go_command_injection             go_sql_injection
swift_force_unwrap               swift_unsafe_operations
swift_nspredicate_format         java_runtime_exec
java_deserialization             php_eval
php_unserialize                  ruby_eval
ruby_yaml_load
```

### Benefits of Built-in Patterns

1. **Zero Configuration**: Security warnings work immediately, no setup needed
2. **Comprehensive Coverage**: 10+ languages and 27+ vulnerability types
3. **Low Noise**: Warnings shown once per session
4. **Educational**: Learn about security as you code
5. **Easy Customization**: Disable specific patterns with one line
6. **Always Updated**: Pattern updates don't require config changes

## 🎯 Best Practices

1. **Start Simple**: Begin with basic mappings and add complexity gradually
2. **Test Thoroughly**: Verify each mapping works as expected
3. **Document Choices**: Comment your configuration for team members
4. **Use Consistent Patterns**: Establish team conventions for mappings
5. **Regular Updates**: Review and update configurations as tools evolve
6. **Security First**: Keep security patterns enabled unless you have a specific reason to disable them
7. **Review Warnings**: Don't automatically disable patterns - consider why they're triggering

---

**Next Steps:**
- [Explore example configurations](examples.md)
- [Learn best practices](best-practices.md)
- [Set up Claude Code integration](claude-integration.md)

#configuration #toml #commands #mapping #setup #security