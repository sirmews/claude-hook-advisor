# Claude Hook Advisor Installation Checklist

Use this checklist to validate your installation is working correctly.

## Installation Verification

### Binary Installation
- [ ] Binary is installed: `which claude-hook-advisor`
- [ ] Binary is executable: `claude-hook-advisor --version`
- [ ] Binary is in PATH (should show `~/.cargo/bin/claude-hook-advisor`)

### Plugin Installation
- [ ] Plugin directory exists: `~/.claude/plugins/claude-hook-advisor/`
- [ ] Plugin registered in settings.json
- [ ] Slash commands available: `/history` works in Claude Code

### Configuration
- [ ] Config file exists: `.claude-hook-advisor.toml`
- [ ] Config is valid TOML (no syntax errors)
- [ ] History is enabled: `[command_history] enabled = true`
- [ ] Log file path is writable

### Hook Registration
- [ ] PreToolUse hook registered for Bash
- [ ] PostToolUse hook registered for Bash
- [ ] UserPromptSubmit hook registered (optional)
- [ ] Hooks point to correct binary path

### Functional Testing
- [ ] Run a test command: `echo "test"` (should log to history)
- [ ] Check history: `claude-hook-advisor --history` shows the command
- [ ] Status shows success: ✓ for the test command
- [ ] Run a failing command: `ls /nonexistent`
- [ ] Check failures: `claude-hook-advisor --history --failures` shows it
- [ ] Status shows failure: ✗ FAILED for the failed command

## Troubleshooting

### If binary not found:
```bash
# Reinstall the binary
cargo install claude-hook-advisor --force

# Ensure ~/.cargo/bin is in PATH
echo $PATH | grep -q ".cargo/bin" || echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### If hooks not firing:
1. Restart Claude Code (hooks load at session start)
2. Check settings.json has correct plugin path
3. Verify hooks.json exists in plugin directory
4. Test manually:
   ```bash
   echo '{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"echo test"}}' | claude-hook-advisor --hook
   ```

### If history not recording:
1. Check config file: `cat .claude-hook-advisor.toml`
2. Verify history is enabled
3. Check database is writable:
   ```bash
   touch ~/.claude-hook-advisor/bash-history.db
   chmod 644 ~/.claude-hook-advisor/bash-history.db
   ```
4. Check for errors: Add `RUST_LOG=debug` to hook command

### If database errors:
```bash
# Check database file
ls -la ~/.claude-hook-advisor/bash-history.db

# Fix permissions
chmod 644 ~/.claude-hook-advisor/bash-history.db

# Recreate database (WARNING: loses history)
rm ~/.claude-hook-advisor/bash-history.db
# Run a test command to recreate it
```

## Quick Validation Command

Run this one-liner to check everything:

```bash
which claude-hook-advisor && \
claude-hook-advisor --version && \
[ -f .claude-hook-advisor.toml ] && \
claude-hook-advisor --history --limit 1 && \
echo "✓ All checks passed!" || echo "✗ Some checks failed"
```

## Getting Help

If you're still having issues:

1. Check the troubleshooting guide: [README.md](../README.md#troubleshooting)
2. Review plugin documentation: [plugin/README.md](README.md)
3. Open an issue: https://github.com/sirmews/claude-hook-advisor/issues

## Success Criteria

You're ready to use claude-hook-advisor when:

✅ All checklist items are checked
✅ Test commands appear in history
✅ Failed commands are marked as FAILED
✅ Slash commands work in Claude Code
✅ No error messages in Claude Code output
