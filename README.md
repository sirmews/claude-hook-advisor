# Claude Hook Advisor

A Rust CLI tool that integrates with Claude Code using a **triple-hook architecture** to provide intelligent command suggestions, semantic directory aliasing, and automatic documentation standards enforcement. Enhance your development workflow with automatic command mapping, natural language directory references, and real-time documentation validation.

## 🎬 What You'll Experience

Once installed, claude-hook-advisor works invisibly in your Claude Code conversations:

### Directory Aliasing Magic ✨
**You type:** *"What files are in my docs?"*
**Claude responds:** *"I'll check what files are in your docs directory at /Users/you/Documents/Documentation."*

Behind the scenes, you'll see:
```
<user-prompt-submit-hook>Directory reference 'docs' resolved to: /Users/you/Documents/Documentation</user-prompt-submit-hook>
```

**You type:** *"Check the project_docs for API documentation"*
**Claude automatically knows:** *Uses `/Users/you/Documents/Documentation/my-project/` without you typing the full path*

### Command Intelligence in Action 🚀
**Claude tries to run:** `npm install`
**Tool intervenes:** *Suggests `bun install` based on your configuration*
**Claude automatically runs:** `bun install` *with no manual intervention*

**You see:** Claude seamlessly uses your preferred tools without you having to correct it every time.

### Documentation Standards Enforcement ✅
**You type:** *"Help me create documentation for the API"*
**Hook responds:** *Automatically displays documentation standards including required YAML frontmatter, date formats, and tag conventions*

**Claude creates:** `api-guide.md` with minimal content
**Hook validates:** *Automatically checks compliance and shows detailed issues*
```
⚠ Markdown file 'api-guide.md' has compliance issues:
  - error (missing frontmatter): Document is missing YAML frontmatter. Add frontmatter with required fields.
```

### The Magic is Invisible
- No extra commands to remember
- No interruptions to your workflow  
- Natural language directory references just work
- Your preferred tools are used automatically
- Documentation standards are enforced automatically
- All happens transparently in Claude Code conversations

## Features

### 🎯 Command Intelligence
- **Smart command mapping**: Map any command to preferred alternatives with regex support
- **Per-project configuration**: Each project can have its own `.claude-hook-advisor.toml` file
- **Triple-hook integration**: PreToolUse, UserPromptSubmit, and PostToolUse hooks

### 📁 Semantic Directory Aliasing
- **Natural language directory references**: Use "docs", "central_docs", "project_docs" in conversations
- **Simple path mapping**: Direct alias-to-path mapping with tilde expansion
- **Automatic resolution**: Claude Code automatically resolves semantic references to canonical paths
- **TOML configuration**: Simple configuration file-based setup

### 📝 Documentation Standards Enforcement
- **Automatic guidance**: Shows documentation standards when you mention documentation keywords
- **Real-time validation**: Validates markdown files after creation/modification via any tool (Write, Edit, Bash)
- **Comprehensive checks**: YAML frontmatter, required fields, date formats, tag conventions, filename rules
- **Detailed feedback**: Shows specific compliance issues with suggestions for fixes

### 🚀 Performance & Security
- **Fast and lightweight**: Built in Rust for optimal performance (<21ms hook execution)
- **Path canonicalization**: Security against directory traversal attacks
- **Graceful error handling**: Robust fallback mechanisms that never break hooks

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

### 2. Configure Directory Aliases
Edit your `.claude-hook-advisor.toml` file to set up directory aliases:

```toml
# Semantic directory aliases - use natural language!
[semantic_directories]
"project docs" = "~/Documents/Documentation/my-project"
"central docs" = "~/Documents/Documentation" 
"claude docs" = "~/Documents/Documentation/claude"
"test data" = "~/Documents/test-data"
```

**Pro tip:** Use quoted, space-separated aliases for natural conversation:
- *"Check the project docs folder"* → matches `"project docs"`
- *"Look in test data directory"* → matches `"test data"`

### 3. Configure Command Mappings
Create a `.claude-hook-advisor.toml` file in your project root:

```toml
# Command mappings
[commands]
npm = "bun"
yarn = "bun"
npx = "bunx"
curl = "wget --verbose"

# Semantic directory aliases - natural language
[semantic_directories]
"project docs" = "~/Documents/Documentation/my-project"
"central docs" = "~/Documents/Documentation"
"claude docs" = "~/Documents/Documentation/claude"
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
- **UserPromptSubmit**: Directory reference detection and documentation standards guidance
- **PostToolUse**: Analytics, execution tracking, and markdown file validation

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

### Command Intelligence (PreToolUse Hook) 🚦

**The Flow:**
1. **Command Detection**: When Claude Code tries to run a Bash command, the hook receives JSON input
2. **Configuration Loading**: The tool loads `.claude-hook-advisor.toml` from the current directory
3. **Pattern Matching**: Uses word-boundary regex to match commands (e.g., `npm` matches `npm install` but not `npm-check`)
4. **Suggestion Generation**: If a match is found, returns a blocking response with the suggested replacement
5. **Claude Integration**: Claude receives the suggestion and automatically retries with the correct command

**Behind the Scenes:**
```rust
// Simplified code flow
let config = load_config(".claude-hook-advisor.toml")?;
let command = parse_bash_command(&hook_input.tool_input.command);

if let Some(replacement) = config.commands.get(&command.base_command) {
    return Ok(HookResponse::Block {
        reason: format!("Command '{}' is mapped to '{}'", command.base_command, replacement),
        suggested_command: command.replace_base_with(replacement),
    });
}
```

**What makes it smart:**
- Word-boundary matching prevents false positives (`npm` won't match `npm-check`)
- Preserves command arguments (`npm install --save` → `bun install --save`)
- Fast regex-based pattern matching (~1ms response time)

---

### Directory Aliasing (UserPromptSubmit Hook) 📁

**The Flow:**
1. **Text Analysis**: Scans user prompts for semantic directory references (e.g., "docs", "project_docs")
2. **Pattern Recognition**: Uses regex to detect directory aliases in natural language
3. **Path Expansion**: Expands tilde (~) to user home directory
4. **Path Resolution**: Converts semantic references to canonical filesystem paths
5. **Security Validation**: Performs path canonicalization to prevent traversal attacks

**Behind the Scenes:**
```rust
// Pattern detection
let patterns = [
    r"\b(docs|documentation)\b",
    r"\bproject[_\s]docs?\b", 
    r"\bcentral[_\s]docs?\b"
];

// Tilde expansion
let resolved = expand_tilde(path_template)?;

// Security canonicalization
let canonical = fs::canonicalize(&resolved)?;
```

**What makes it secure:**
- Path canonicalization prevents `../../../etc/passwd` attacks
- Only resolves to configured directories
- Validates paths exist before resolution

---

### Documentation Standards Enforcement (UserPromptSubmit & PostToolUse Hooks) 📝

**UserPromptSubmit Flow - Proactive Guidance:**
1. **Keyword Detection**: Scans user prompts for documentation-related keywords ("write documentation", "create guide", "readme", etc.)
2. **Standards Retrieval**: Loads documentation standards based on established conventions
3. **Guidance Display**: Shows required YAML frontmatter, date formats, tag rules, and filename conventions

**PostToolUse Flow - Automatic Validation:**
1. **File Detection**: Monitors tool outputs for markdown file creation/modification (Write, Edit, Bash tools)
2. **Standards Validation**: Validates files against comprehensive documentation standards
3. **Compliance Reporting**: Provides detailed feedback on issues and suggestions

**Behind the Scenes:**
```rust
// Documentation keyword detection
let doc_keywords = ["write documentation", "create guide", "document", "readme", "manual"];
if doc_keywords.iter().any(|keyword| prompt_lower.contains(keyword)) {
    let standards = get_documentation_standards()?;
    println!("{}", standards.guidance_text);
}

// Markdown file validation
if file_path.ends_with(".md") && Path::new(file_path).exists() {
    let result = validate_document_compliance(file_path)?;
    if !result.is_compliant {
        for issue in result.issues {
            println!("  - {issue}");
        }
    }
}
```

**What gets validated:**
- **YAML Frontmatter**: Required fields (title, created_at, updated_at, tags, description)
- **Date Formats**: YYYY-MM-DD validation with chrono
- **Tag Conventions**: Must start with # and use kebab-case (#project-name, #guide)
- **Filename Rules**: Suggests kebab-case.md naming conventions

---

### Analytics (PostToolUse Hook) 📊

**The Flow:**
1. **Execution Tracking**: Receives command results with success/failure data
2. **Performance Monitoring**: Tracks command success rates and execution patterns
3. **Analytics Logging**: Provides insights for optimization and monitoring

**Behind the Scenes:**
```rust
// Success/failure tracking
match hook_data.tool_response.exit_code {
    0 => log::info!("Command '{}' succeeded", command),
    code => log::warn!("Command '{}' failed (exit: {})", command, code),
}
```

**Enhanced capabilities:**
- Command success rate analytics
- Documentation compliance tracking
- Performance optimization suggestions
- Usage pattern insights

## Example Output

### Real Claude Code Conversation

Here's what an actual conversation looks like with claude-hook-advisor working:

**🗣️ You:** "What files are in my docs?"

**🤖 Claude:** "⏺ I'll check what files are in your docs directory at /Users/you/Documents/Documentation."

**Behind the scenes:**
```
[DEBUG] UserPromptSubmit hook triggered
[DEBUG] Pattern matched: 'docs' -> '~/Documents/Documentation'  
[DEBUG] Path resolved: /Users/you/Documents/Documentation
```

**Hook message in Claude:**
```
<user-prompt-submit-hook>Directory reference 'docs' resolved to: /Users/you/Documents/Documentation</user-prompt-submit-hook>
```

---

**🗣️ You:** "Install the dependencies for this project"

**🤖 Claude:** "I'll install the dependencies using npm install."
*(Claude attempts: `npm install`)*

**Hook intercepts:**
```json
{
  "decision": "block",
  "reason": "Command 'npm' is mapped to 'bun' instead",
  "suggested_command": "bun install"
}
```

**🤖 Claude:** "I'll use bun install instead based on your project preferences."
*(Claude runs: `bun install`)*

**Result:** Your preferred package manager is used automatically, no manual correction needed!

---

**🗣️ You:** "I need to write documentation for the new API"

**🤖 Claude:** "I'll help you create documentation for the new API."

**Hook provides guidance:**
```
Documentation standards detected:
  Required frontmatter fields: ["title", "created_at", "updated_at", "tags", "description"]
  Date format: YYYY-MM-DD
  Tag rules: require_hash_prefix=true, prefer_kebab_case=true
  Filename conventions: kebab-case with .md

Documentation Standards:
• Required YAML frontmatter: title, created_at, updated_at, tags, description
• Date format: YYYY-MM-DD (example: 2025-08-25)
• Tags must start with # and use kebab-case (#project-name, #guide)
• Filenames should use kebab-case.md
• Include Purpose and Content Structure sections
• Keep documents focused and concise
```

**Later, Claude creates:** `api-guide.md` with basic content

**Hook automatically validates:**
```
⚠ Markdown file 'api-guide.md' has compliance issues:
  - error (missing frontmatter): Document is missing YAML frontmatter. Add frontmatter with required fields.
```

**Result:** You get proactive guidance before creating docs, and automatic validation after!

---

### Command Line Testing

**Directory Resolution:**
```bash
# Test directory resolution via hook
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs directory"}' | claude-hook-advisor --hook

# Expected output:
# Directory reference 'docs' resolved to: /Users/you/Documents/Documentation

*Note: Directory resolution requires the path to exist on your filesystem.*
```

**Hook Simulation:**
```bash
$ echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check the docs directory"}' | claude-hook-advisor --hook
<user-prompt-submit-hook>Directory reference 'docs' resolved to: /Users/you/Documents/Documentation</user-prompt-submit-hook>

$ echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook
{
  "decision": "block", 
  "reason": "Command 'npm' is mapped to 'bun' instead",
  "suggested_command": "bun install"
}
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

# Manual testing - Documentation guidance (UserPromptSubmit)
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"create documentation for the API"}' | ./target/debug/claude-hook-advisor --hook

# Manual testing - Markdown validation (PostToolUse)
echo '{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Write","tool_input":{"file_path":"test.md"},"tool_response":{"exit_code":0}}' | ./target/debug/claude-hook-advisor --hook

# Test directory resolution with existing config
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check the docs directory"}' | ./target/debug/claude-hook-advisor --hook
```

## 🔧 Troubleshooting & Debug

### Understanding Hook Messages

When claude-hook-advisor is working correctly, you'll see these messages in Claude Code:

**Directory Resolution:**
```
<user-prompt-submit-hook>Directory reference 'docs' resolved to: /Users/you/Documents/Documentation</user-prompt-submit-hook>
```

**Command Suggestions:**
```
<pre-tool-use-hook>Command 'npm' mapped to 'bun'. Suggested: bun install</pre-tool-use-hook>
```

**Documentation Guidance:**
```
Documentation standards detected:
  Required frontmatter fields: ["title", "created_at", "updated_at", "tags", "description"]
  Date format: YYYY-MM-DD
  [additional guidance text...]
```

**Markdown Validation:**
```
✓ Markdown file 'api-guide.md' is compliant with documentation standards
```
or
```
⚠ Markdown file 'readme.md' has compliance issues:
  - error (missing frontmatter): Document is missing YAML frontmatter. Add frontmatter with required fields.
```

**Execution Tracking:**
```
<post-tool-use-hook>Command 'bun install' completed successfully (exit code: 0)</post-tool-use-hook>
```

### Debug Mode

Enable detailed logging to see what's happening behind the scenes:

```bash
# Add RUST_LOG=debug to your Claude Code settings
{
  "hooks": {
    "UserPromptSubmit": { ".*": "RUST_LOG=debug claude-hook-advisor --hook" },
    "PreToolUse": { "Bash": "RUST_LOG=debug claude-hook-advisor --hook" },
    "PostToolUse": { "Bash": "RUST_LOG=debug claude-hook-advisor --hook" }
  }
}
```

**Debug output shows:**
- Configuration file loading
- Pattern matching details
- Path resolution steps
- Variable substitution
- Security validation

### Common Issues & Solutions

#### 🚫 Hooks Not Triggering
**Problem:** No hook messages appear in Claude Code conversations

**Solutions:**
1. Verify hook installation by checking your Claude Code settings file
2. Check `.claude/settings.json` or `.claude/settings.local.json`:
   ```json
   {
     "hooks": {
       "UserPromptSubmit": { ".*": "claude-hook-advisor --hook" }
     }
   }
   ```
3. Ensure `claude-hook-advisor` is in your PATH: `which claude-hook-advisor`
4. Test manually: `echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs"}' | claude-hook-advisor --hook`

#### 📁 Directory Not Resolved
**Problem:** "docs" doesn't resolve to the expected path

**Solutions:**
1. Check configuration file exists: `ls .claude-hook-advisor.toml`
2. Verify alias configuration:
   ```toml
   [semantic_directories]
   docs = "~/Documents/Documentation"
   ```
3. Test resolution via hook: `echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs"}' | claude-hook-advisor --hook`
4. Check file permissions: `ls -la .claude-hook-advisor.toml`

#### ⚙️ Commands Not Being Mapped
**Problem:** `npm` still runs instead of `bun`

**Solutions:**
1. Verify command mapping in config:
   ```toml
   [commands]
   npm = "bun"
   ```
2. Test mapping: `echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook`
3. Check word boundaries: `npm-check` won't match `npm = "bun"` (by design)
4. Add debug logging to see pattern matching

#### 🔒 Permission Issues
**Problem:** Hook fails with permission errors

**Solutions:**
1. Make binary executable: `chmod +x ~/.cargo/bin/claude-hook-advisor`
2. Check file ownership: `ls -la ~/.cargo/bin/claude-hook-advisor`
3. Verify PATH includes `~/.cargo/bin`: `echo $PATH`

#### 🐛 Debugging Your Configuration

**Test each component individually:**

```bash
# Test directory resolution via hook
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs"}' | claude-hook-advisor --hook

# Test command mapping
echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook

# Test user prompt analysis
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check the docs directory"}' | claude-hook-advisor --hook

# Check configuration by testing resolution
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs"}' | claude-hook-advisor --hook
```

### Performance Notes

- **Startup time:** ~1-5ms per hook call
- **Memory usage:** ~2-3MB per process  
- **File watching:** Configuration is loaded on each hook call (no caching)
- **Path resolution:** Uses filesystem canonicalization for security

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

### Documentation Standards Enforcement
- **Consistent Documentation**: Automatic enforcement of YAML frontmatter, date formats, and tag conventions
- **Proactive Guidance**: Get documentation standards before you start writing
- **Quality Assurance**: Real-time validation catches formatting issues immediately
- **Team Standards**: Ensure all team members follow the same documentation conventions
- **Workflow Integration**: Works seamlessly with any tool that creates/modifies markdown files

## Similar Tools

This tool is inspired by and similar to:
- Shell aliases (but works at the Claude Code level)
- Git hooks (but for command execution)
- Package manager configuration files

## Support

If you find this tool useful, consider supporting its development:

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/A0A01HT0RG)

---
