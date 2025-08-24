---
title: "Installation Guide"
description: "Step-by-step instructions for installing Claude Hook Advisor"
tags: ["installation", "setup", "build", "rust", "cargo"]
---

# Installation Guide

This guide covers multiple ways to install Claude Hook Advisor on your system.

## 📋 Prerequisites

- **Rust toolchain** (1.70 or later)
- **Git** (for cloning the repository)
- **Make** (optional, for using Makefile commands)

### Installing Rust

If you don't have Rust installed:

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

## 🚀 Installation Methods

### Method 1: Build from Source (Recommended)

#### 1. Clone the Repository
```bash
git clone https://github.com/your-org/claude-hook-advisor.git
cd claude-hook-advisor
```

#### 2. Build and Install
```bash
# Option A: Install to ~/.local/bin (recommended)
make install-local

# Option B: Install system-wide (requires sudo)
make install

# Option C: Install via cargo (globally available)
make install-cargo
```

#### 3. Verify Installation
```bash
# Check if the binary is accessible
claude-hook-advisor --version

# Test with example configuration
claude-hook-advisor --help
```

### Method 2: Manual Build and Copy

#### 1. Build the Project
```bash
# Debug build (faster compilation)
make build

# Release build (optimized binary)
make release
```

#### 2. Copy Binary to PATH
```bash
# Copy to ~/.local/bin
mkdir -p ~/.local/bin
cp target/release/claude-hook-advisor ~/.local/bin/

# Or copy to /usr/local/bin (requires sudo)
sudo cp target/release/claude-hook-advisor /usr/local/bin/
```

#### 3. Update PATH (if needed)
```bash
# Add to ~/.bashrc or ~/.zshrc
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Method 3: Development Installation

For development and testing:

```bash
# Build in debug mode
cargo build

# Run directly from target directory
./target/debug/claude-hook-advisor --version

# Install in development mode
cargo install --path .
```

## 🔧 Available Make Targets

The project includes a comprehensive Makefile with the following targets:

```bash
# Building
make build          # Build in debug mode
make release        # Build in release mode
make clean          # Clean build artifacts

# Installation
make install        # Install using cargo (globally)
make install-local  # Install to ~/.local/bin
make install-system # Install to /usr/local/bin (requires sudo)

# Development
make test           # Run all tests
make lint           # Run clippy linting
make fmt            # Format code
make check          # Check code without building

# Configuration
make example-config # Create example configuration file
make run-example    # Test with example input

# Help
make help           # Show all available targets
```

## 🗂️ Installation Locations

### Recommended: ~/.local/bin
```bash
make install-local
```
- **Pros**: No sudo required, user-specific installation
- **Cons**: Need to ensure ~/.local/bin is in PATH
- **Best for**: Personal development, multi-user systems

### System-wide: /usr/local/bin
```bash
make install-system
```
- **Pros**: Available to all users, automatically in PATH
- **Cons**: Requires sudo privileges
- **Best for**: Single-user systems, team installations

### Cargo Global: ~/.cargo/bin
```bash
make install-cargo
```
- **Pros**: Managed by Cargo, easy updates
- **Cons**: Requires Rust toolchain on target system
- **Best for**: Rust developers, development environments

## ✅ Verification

After installation, verify everything works correctly:

### 1. Check Binary Location
```bash
which claude-hook-advisor
# Should show: /home/user/.local/bin/claude-hook-advisor
# or: /usr/local/bin/claude-hook-advisor
```

### 2. Test Version Command
```bash
claude-hook-advisor --version
# Should show: claude-hook-advisor 0.2.0
```

### 3. Test Help Command
```bash
claude-hook-advisor --help
# Should display usage information
```

### 4. Test Hook Functionality

#### Test Command Mapping (PreToolUse)
```bash
# Create a test configuration
echo '[commands]
npm = "bun"' > .claude-hook-advisor.toml

# Test with example input
echo '{"session_id":"test","transcript_path":"","cwd":"","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm install"}}' | claude-hook-advisor --hook
```

Expected output:
```json
{
  "decision": "block",
  "reason": "Command 'npm' is mapped to use 'bun' instead. Try: bun install"
}
```

#### Test Directory Aliasing
```bash
# Create a test configuration with directory aliases
echo '[semantic_directories]
"central docs" = "~/Documents/Documentation"' > .claude-hook-advisor.toml

# Test UserPromptSubmit hook
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check the docs directory"}' | claude-hook-advisor --hook
```

#### Install All Claude Code Hooks
```bash
# Automatically install all three hooks with backup
claude-hook-advisor --install
```

## 🔄 Updating

### From Source
```bash
cd claude-hook-advisor
git pull origin main
make release
make install-local  # or your preferred installation method
```

### Via Cargo
```bash
cargo install --path . --force
```

## 🗑️ Uninstallation

### Remove Binary
```bash
# If installed to ~/.local/bin
rm ~/.local/bin/claude-hook-advisor

# If installed system-wide
sudo rm /usr/local/bin/claude-hook-advisor

# If installed via cargo
cargo uninstall claude-hook-advisor
```

### Remove Configuration Files
```bash
# Remove project-specific configs (optional)
find . -name ".claude-hook-advisor.toml" -delete

# Remove any global configs you may have created
rm ~/.claude-hook-advisor.toml  # if you created one
```

## 🐛 Troubleshooting Installation

### Binary Not Found
```bash
# Check if binary exists
ls -la ~/.local/bin/claude-hook-advisor

# Check PATH
echo $PATH | grep -o ~/.local/bin

# Add to PATH if missing
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Permission Denied
```bash
# Make binary executable
chmod +x ~/.local/bin/claude-hook-advisor

# Or for system-wide installation
sudo chmod +x /usr/local/bin/claude-hook-advisor
```

### Build Failures
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
make clean
make release

# Check for missing dependencies
cargo check
```

### Missing Dependencies
```bash
# Install build essentials (Ubuntu/Debian)
sudo apt update
sudo apt install build-essential

# Install build essentials (CentOS/RHEL)
sudo yum groupinstall "Development Tools"

# Install build essentials (macOS)
xcode-select --install
```

## 📚 Available CLI Commands

After installation, Claude Hook Advisor provides these commands:

### Core Operations
```bash
# Run as hook (reads JSON from stdin)
claude-hook-advisor --hook

# Use custom configuration file
claude-hook-advisor --config custom.toml --hook

# Show version information
claude-hook-advisor --version

# Show help
claude-hook-advisor --help
```

### Hook Management
```bash
# Install all hooks into Claude Code settings (with backup)
claude-hook-advisor --install

# Remove hooks from Claude Code settings (with backup)  
claude-hook-advisor --uninstall
```

### Directory Aliasing
**Note:** Directory aliases are configured via TOML file only. No CLI management commands are available in v0.2.0.

```bash
# Create configuration file
echo '[semantic_directories]
"project docs" = "~/Documents/Documentation/my-project"
"central docs" = "~/Documents/Documentation"' > .claude-hook-advisor.toml

# Test directory resolution via hook
echo '{"session_id":"test","hook_event_name":"UserPromptSubmit","prompt":"check docs"}' | claude-hook-advisor --hook
```

## 🎯 Next Steps

After successful installation:

1. **[Set up directory aliases](directory-aliasing.md)** - Configure semantic directory references
2. **[Create your first configuration](configuration.md)** - Set up project-specific command mappings
3. **[Integrate with Claude Code](claude-integration.md)** - Configure the hook in Claude Code
4. **[Explore examples](examples.md)** - See real-world configuration examples

---

**Need help?** Check our [Troubleshooting Guide](troubleshooting.md) or [FAQ](faq.md).

#installation #setup #build #rust #cargo #make