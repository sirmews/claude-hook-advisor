# Learnings from GitHub Spec-Kit

This document captures patterns and practices from [github/spec-kit](https://github.com/github/spec-kit) that could improve claude-hook-advisor's reliability and user experience.

## What is Spec-Kit?

Spec-Kit is GitHub's toolkit for spec-driven development with AI coding assistants. It uses slash commands, validation checklists, and structured workflows to ensure high-quality AI-assisted development.

## Key Patterns We Can Adopt

### 1. Validation Checklists ⭐⭐⭐

**What they do:**
- "Unit tests for requirements" - validate quality before proceeding
- Explicit success criteria for each phase
- Self-service debugging for users

**How we applied it:**
- Created `INSTALL-CHECKLIST.md` for installation validation
- Added success criteria for each component (binary, plugin, hooks, config)
- Provided troubleshooting steps inline

**Future enhancements:**
- Add runtime validation checklist for ongoing usage
- Create automated checklist validation command
- Add to slash commands for easy access

### 2. Slash Commands with Scripts

**What they do:**
```markdown
---
description: Validate installation
scripts:
  sh: scripts/validate.sh --json
---
```

**How we could apply it:**
- `/hook-advisor-validate` - Run installation validation
- `/hook-advisor-diagnose` - Debug common issues
- `/hook-advisor-stats` - Show usage statistics
- Scripts output JSON for easy parsing

**Status:** Partially implemented
- ✅ Created `/validate` command
- ⏳ Need to implement `--validate` flag in binary
- ⏳ Need validation script

### 3. Rich Error Messages with Context

**What they do:**
```json
{
  "error": "File not found",
  "path": "/path/to/file",
  "suggestion": "Run: mkdir -p /path/to",
  "docs": "https://..."
}
```

**How we could apply it:**
```rust
// Current: "Failed to log command"
// Better:
eprintln!("❌ Failed to log command to history database");
eprintln!("   Path: {}", db_path);
eprintln!("   Reason: {}", error);
eprintln!("   Fix: chmod 644 {}", db_path);
eprintln!("   Docs: https://github.com/sirmews/claude-hook-advisor#troubleshooting");
```

**Status:** Not implemented
**Priority:** HIGH - Improves user experience significantly

### 4. Graceful Fallbacks

**What they do:**
- Multiple strategies for finding repo root
- Fallback to sensible defaults
- Never crash, always continue

**Example from spec-kit:**
```bash
get_repo_root() {
    if git rev-parse --show-toplevel >/dev/null 2>&1; then
        git rev-parse --show-toplevel
    else
        # Fall back to script location for non-git repos
        local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        (cd "$script_dir/../../.." && pwd)
    fi
}
```

**How we could apply it:**
```rust
// Database initialization with fallback strategy:
fn init_database_with_fallbacks(config_path: &str) -> Result<Connection> {
    // 1. Try configured path
    if let Ok(conn) = try_init_database(config_path) {
        return Ok(conn);
    }

    // 2. Try creating parent directories
    if let Ok(conn) = try_create_parent_and_init(config_path) {
        warn!("Created parent directory for database");
        return Ok(conn);
    }

    // 3. Try alternative location
    let fallback_path = "~/.cache/claude-hook-advisor/history.db";
    if let Ok(conn) = try_init_database(fallback_path) {
        warn!("Using fallback database location: {}", fallback_path);
        return Ok(conn);
    }

    // 4. Last resort: in-memory database (no persistence)
    warn!("Using in-memory database - history will not persist");
    Connection::open_in_memory()
}
```

**Status:** Not implemented
**Priority:** MEDIUM - Improves reliability

### 5. Prerequisites Checking

**What they do:**
- Dedicated `check-prerequisites.sh` script
- Validates environment before proceeding
- Clear pass/fail output with paths

**How we could apply it:**
Add `--preflight` or `--validate` command:

```bash
claude-hook-advisor --validate

Validating claude-hook-advisor installation...

✓ Binary installation
  ✓ Binary found at: ~/.cargo/bin/claude-hook-advisor
  ✓ Version: 1.0.0
  ✓ Executable permissions: OK

✓ Configuration
  ✓ Config file: .claude-hook-advisor.toml
  ✓ Valid TOML syntax
  ✓ History enabled: true
  ✓ Database path: ~/.claude-hook-advisor/bash-history.db

✓ Database
  ✓ Database file exists
  ✓ Writable: yes
  ✓ Schema version: 1
  ✓ Record count: 1,234

✗ Hooks
  ✓ PreToolUse registered
  ✗ PostToolUse not registered
  ✓ UserPromptSubmit registered

❌ Validation failed: PostToolUse hook not registered
Fix: Add PostToolUse hook to .claude/settings.json
See: https://github.com/sirmews/claude-hook-advisor#installation
```

**Status:** Not implemented
**Priority:** HIGH - Makes installation much easier

### 6. JSON Output for Tooling

**What they do:**
- All scripts have `--json` flag
- Structured output for AI parsing
- Easier integration with other tools

**How we could apply it:**
```bash
# Current output
claude-hook-advisor --history
# Shows formatted table

# With JSON flag
claude-hook-advisor --history --json
{
  "records": [
    {
      "timestamp": "2025-11-12T...",
      "command": "ls /",
      "status": "success",
      "exit_code": 0,
      "cwd": "/tmp",
      "session_id": "abc123"
    }
  ],
  "total": 1,
  "filters": {"failures_only": false, "limit": 20}
}
```

**Status:** Not implemented
**Priority:** MEDIUM - Nice to have for tooling

### 7. Self-Documenting Commands

**What they do:**
- Rich descriptions in command files
- Inline examples and expected outcomes
- Troubleshooting tips directly in commands

**How we applied it:**
- ✅ Added detailed descriptions to slash commands
- ✅ Added AI analysis prompts
- ✅ Included expected output examples

**Future enhancements:**
- Add troubleshooting hints inline
- Add "what success looks like" sections
- Add common issues and fixes

### 8. Feature Flags / Conditional Execution

**What they do:**
- Check for available resources before using them
- Graceful degradation when features unavailable
- User-configurable behavior

**How we could apply it:**
```toml
[features]
fail_fast = false           # Stop on first error or continue
verbose_logging = true      # Show detailed output
auto_retry = true           # Retry failed operations
max_retries = 3             # Number of retry attempts
fallback_locations = [      # Alternative DB locations to try
  "~/.cache/claude-hook-advisor/history.db",
  "/tmp/claude-hook-advisor-history.db"
]

[performance]
batch_writes = true         # Batch DB writes for performance
flush_interval = 5          # Seconds between flushes
```

**Status:** Not implemented
**Priority:** LOW - Advanced feature

## Implementation Roadmap

### Phase 1: Immediate Wins (Week 1)
- [x] Add installation checklist
- [x] Add validation slash command skeleton
- [ ] Implement `--validate` flag in binary
- [ ] Enhance error messages with context
- [ ] Add graceful database fallbacks

### Phase 2: Better DX (Week 2-3)
- [ ] Add JSON output mode (`--json` flag)
- [ ] Create validation script for slash command
- [ ] Add troubleshooting to command output
- [ ] Implement retry logic for transient failures

### Phase 3: Advanced Features (Week 4+)
- [ ] Add feature flags to config
- [ ] Implement performance optimizations
- [ ] Add diagnostic logging mode
- [ ] Create integration tests using validation

## Files Created

Based on spec-kit learnings:

1. `plugin/INSTALL-CHECKLIST.md` - Installation validation checklist
2. `plugin/commands/validate.md` - Validation slash command
3. `docs/SPEC-KIT-LEARNINGS.md` - This document

## References

- [Spec-Kit Repository](https://github.com/github/spec-kit)
- [Spec-Kit Documentation](https://github.github.io/spec-kit/)
- [Common utilities](https://github.com/github/spec-kit/blob/main/scripts/bash/common.sh)

## Key Takeaways

1. **Validation is critical** - Users need to know if setup worked
2. **Fail gracefully** - Never crash, always provide helpful context
3. **Self-service debugging** - Give users tools to fix issues themselves
4. **Structured output** - JSON makes AI integration easier
5. **Check prerequisites** - Validate early, fail fast with clear messages
6. **Fallback strategies** - Have Plan B, C, and D ready
7. **Feature flags** - Let users configure behavior

## Next Steps

1. Review this document with team
2. Prioritize features for implementation
3. Create GitHub issues for each enhancement
4. Start with Phase 1 (immediate wins)
