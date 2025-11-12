#!/bin/bash
# Claude Hook Advisor Plugin Installation Script

set -e

PLUGIN_NAME="claude-hook-advisor"
PLUGIN_DIR="$HOME/.claude/plugins/$PLUGIN_NAME"

echo "🚀 Installing Claude Hook Advisor Plugin"
echo "========================================"
echo ""

# Check if claude-hook-advisor binary is installed
if ! command -v claude-hook-advisor &> /dev/null; then
    echo "❌ Error: claude-hook-advisor binary not found"
    echo ""
    echo "Please install it first:"
    echo "  cargo install claude-hook-advisor"
    echo ""
    echo "Or from source:"
    echo "  git clone https://github.com/sirmews/claude-hook-advisor.git"
    echo "  cd claude-hook-advisor"
    echo "  cargo install --path ."
    exit 1
fi

echo "✅ Found claude-hook-advisor at: $(which claude-hook-advisor)"
echo ""

# Create plugin directory
echo "📁 Creating plugin directory at: $PLUGIN_DIR"
mkdir -p "$PLUGIN_DIR"

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Copy plugin files
echo "📋 Copying plugin files..."
cp -r "$SCRIPT_DIR/.claude-plugin" "$PLUGIN_DIR/"
cp -r "$SCRIPT_DIR/hooks" "$PLUGIN_DIR/"
cp -r "$SCRIPT_DIR/commands" "$PLUGIN_DIR/"
cp "$SCRIPT_DIR/README.md" "$PLUGIN_DIR/"
cp "$SCRIPT_DIR/.claude-hook-advisor.toml.example" "$PLUGIN_DIR/"

echo "✅ Plugin files copied"
echo ""

# Check for existing settings file
SETTINGS_FILE="$HOME/.claude/settings.json"
if [ -f "$SETTINGS_FILE" ]; then
    echo "📝 Settings file found at: $SETTINGS_FILE"
    echo ""
    echo "⚠️  Please manually add the plugin to your settings:"
    echo ""
    echo '  "plugins": ['
    echo '    "~/.claude/plugins/claude-hook-advisor"'
    echo '  ]'
    echo ""
else
    echo "📝 Creating new settings file..."
    mkdir -p "$HOME/.claude"
    cat > "$SETTINGS_FILE" << 'EOF'
{
  "plugins": [
    "~/.claude/plugins/claude-hook-advisor"
  ]
}
EOF
    echo "✅ Settings file created"
    echo ""
fi

# Create config template in current directory
if [ ! -f ".claude-hook-advisor.toml" ]; then
    echo "📄 Creating config template in current directory..."
    cp "$PLUGIN_DIR/.claude-hook-advisor.toml.example" ".claude-hook-advisor.toml"
    echo "✅ Config template created: .claude-hook-advisor.toml"
    echo ""
else
    echo "ℹ️  Config file already exists: .claude-hook-advisor.toml"
    echo ""
fi

echo "🎉 Installation complete!"
echo ""
echo "Next steps:"
echo "  1. Edit .claude-hook-advisor.toml to configure command mappings and directory aliases"
echo "  2. Restart Claude Code to activate the plugin"
echo "  3. Try the slash commands:"
echo "     - /history"
echo "     - /history-failures"
echo "     - /history-search <pattern>"
echo ""
echo "For more information, see:"
echo "  $PLUGIN_DIR/README.md"
echo ""
