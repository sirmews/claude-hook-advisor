# Claude Hook Advisor

An intelligent command suggestion system for Claude Code that learns from your preferences and automatically suggests better alternatives. Features advanced natural language learning, confidence tracking, and execution analytics.

## ✨ Features

### 🧠 **Intelligent Learning System**
- **Natural Language Learning**: Say "use bun instead of npm" and it learns automatically
- **Confidence Tracking**: Adjusts suggestions based on command success rates
- **Never-Suggest**: Automatically stops suggesting commands that consistently fail
- **Context Awareness**: Learns project-specific and global preferences

### 🔄 **Triple-Hook Architecture**
- **PreToolUse**: Intercepts commands and suggests alternatives
- **UserPromptSubmit**: Learns from natural language preferences
- **PostToolUse**: Tracks execution results for continuous improvement

### 📊 **Analytics & Management**
- **Execution Tracking**: Monitors command success rates and effectiveness
- **Confidence Reports**: Detailed analytics on suggestion performance
- **Export/Import**: Share learned configurations across projects and teams
- **CLI Management**: Full command-line interface for managing learned preferences

### ⚡ **Performance & Reliability**
- **Fast Regex Matching**: Word-boundary patterns prevent false positives
- **Atomic Configuration**: Race-condition-free configuration updates
- **Backwards Compatible**: Works with existing `.claude-hook-advisor.toml` files

## 🚀 Installation

### From crates.io (Recommended)
```bash
cargo install claude-hook-advisor
```

### From Source
```bash
git clone https://github.com/sirmews/claude-hook-advisor.git
cd claude-hook-advisor
make install
```

## ⚙️ Configuration

### Triple-Hook Setup (Full Learning System)

**Using Claude Code's `/hooks` command:**
1. Run `/hooks` → `PreToolUse` → `Bash` → `claude-hook-advisor --hook`
2. Run `/hooks` → `UserPromptSubmit` → `.*` → `claude-hook-advisor --hook`  
3. Run `/hooks` → `PostToolUse` → `Bash` → `claude-hook-advisor --hook`

**Manual `.claude/settings.json`:**
```json
{
  "hooks": {
    "PreToolUse": {
      "Bash": "claude-hook-advisor --hook"
    },
    "UserPromptSubmit": {
      ".*": "claude-hook-advisor --hook"
    },
    "PostToolUse": {
      "Bash": "claude-hook-advisor --hook"
    }
  }
}
```

### Project Configuration
Create `.claude-hook-advisor.toml` for static mappings:

```toml
[commands]
npm = "bun"
yarn = "bun"
npx = "bunx"
pip = "uv pip"
curl = "wget --verbose"
```

## 🎯 Usage Examples

### Natural Language Learning
```
# In Claude Code, just say:
"use bun instead of npm"
"I prefer pnpm over yarn"  
"always use deno instead of node"
```

### Automatic Command Suggestions
```bash
# You: Can you run npm install?
# Claude Hook Advisor: Suggests "bun install" 
# Claude: Runs "bun install" automatically
```

### CLI Management
```bash
# View learned mappings
claude-hook-advisor --list-learned

# Generate confidence report
claude-hook-advisor --confidence-report

# Export learned config
claude-hook-advisor --export-config > team-config.toml

# Import team config  
claude-hook-advisor --import-config team-config.toml

# Reset learning data
claude-hook-advisor --reset-learning
```

## 🧪 How It Works

### 1. Learning Phase
- **Natural Language**: Parse phrases like "use X instead of Y"
- **Pattern Recognition**: Identify 8 different learning patterns
- **Confidence Assignment**: Start with moderate confidence scores

### 2. Suggestion Phase  
- **Command Interception**: PreToolUse hook catches Bash commands
- **Mapping Resolution**: Check static config → learned mappings → never-suggest list
- **Intelligent Blocking**: Return JSON to suggest alternatives

### 3. Tracking Phase
- **Execution Monitoring**: PostToolUse hook tracks command results
- **Success Correlation**: Match executed commands with previous suggestions
- **Confidence Adjustment**: Increase confidence for successful commands, decrease for failures
- **Never-Suggest Detection**: Stop suggesting consistently failing commands

## 📈 Advanced Features

### Confidence System
- **Dynamic Adjustment**: Success increases confidence, failures decrease it
- **Time Decay**: Confidence naturally decreases over time without reinforcement
- **Threshold Filtering**: Only suggest mappings above confidence threshold
- **Analytics**: Detailed reports on suggestion effectiveness

### Learning Patterns
```
Direct: "use bun instead of npm"
Preference: "I prefer pnpm over yarn"  
Always: "always use deno instead of node"
Context: "for this project, use bun instead of npm"
Project: "in node projects, use bun instead of npm"
Never: "never suggest npm for yarn"
Replace: "replace all npm with bun"  
Switch: "switch from npm to bun"
```

### Configuration Management
- **Atomic Updates**: Race-condition-free configuration saves
- **Version Migration**: Automatic upgrade from v0.1.0 to v0.3.0 format
- **Export/Import**: Share configurations across projects and teams
- **Backup/Restore**: Full configuration lifecycle management

## 🔧 Development

### Available Commands
```bash
make build          # Build debug version
make release        # Build release version  
make test           # Run 18 unit tests
make lint           # Run clippy (zero warnings)
make fmt            # Format code
make install        # Install globally
make install-local  # Install to ~/.local/bin
make example-config # Create example config
make run-example    # Test with example input
```

### Testing
```bash
# Run comprehensive test suite
make test

# Manual testing with different learning patterns
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"use bun instead of npm"}' | ./target/debug/claude-hook-advisor --hook

# Test command interception
echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | ./target/debug/claude-hook-advisor --hook
```

## 📚 Documentation

- **[FAQ.md](FAQ.md)**: Frequently asked questions and troubleshooting guide
- **Hook Architecture**: Understanding the triple-hook system
- **Learning System**: How natural language processing works
- **Configuration Format**: TOML schema and migration guide
- **CLI Reference**: Complete command-line interface documentation
- **Troubleshooting**: Common issues and debugging tips

## 🎯 Use Cases

### Individual Developers
- **Tool Preferences**: Enforce personal tool choices (`bun` vs `npm`)
- **Workflow Optimization**: Learn from repeated command patterns
- **Muscle Memory**: Automatically adapt to changing tool preferences

### Teams & Organizations
- **Standard Enforcement**: Ensure consistent tooling across team members
- **Migration Support**: Gradually move from legacy tools to modern alternatives
- **Policy Compliance**: Implement security or performance-based tool restrictions
- **Knowledge Sharing**: Export/import learned configurations across team

### Enterprise
- **Audit Logging**: Track command usage and suggestion effectiveness
- **Policy Enforcement**: Block dangerous commands or redirect to approved alternatives
- **Team Analytics**: Generate reports on tooling adoption and effectiveness
- **Configuration Management**: Centralized configuration distribution

## 🤝 Contributing

Contributions welcome! The codebase is well-tested with 18 passing tests and zero clippy warnings.

### Architecture
- **Single File**: All functionality in `src/main.rs` for simplicity
- **Serde Integration**: Full JSON/TOML serialization support
- **Error Handling**: Comprehensive error contexts with `anyhow`
- **Atomic Operations**: Race-condition-free configuration updates

## 📊 Project Status

- **Version**: 0.2.0 (Advanced Learning System)
- **Tests**: 18 passing, 0 failing  
- **Code Quality**: 0 clippy warnings
- **Features**: Complete natural language learning system
- **Stability**: Production-ready with comprehensive error handling

## 🔗 Links

- **Repository**: [github.com/sirmews/claude-hook-advisor](https://github.com/sirmews/claude-hook-advisor)
- **Crates.io**: [crates.io/crates/claude-hook-advisor](https://crates.io/crates/claude-hook-advisor)
- **Issues**: [GitHub Issues](https://github.com/sirmews/claude-hook-advisor/issues)

## 📄 License

MIT OR Apache-2.0

---

*Built with ❤️ for the Claude Code community*

---

**Last updated**: 2025-01-24  
**Version**: 0.2.0  
*#claude-code #rust #cli-tools #intelligent-automation #learning-system*