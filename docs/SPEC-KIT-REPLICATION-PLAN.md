# Spec-Kit Replication Plan for Claude Hook Advisor

This document evaluates which Spec-Kit patterns we can realistically replicate and provides an implementation plan.

## Quick Summary

| Pattern | Value | Effort | Priority | Status |
|---------|-------|--------|----------|--------|
| Validation Command | High | Low | P0 | Ready to implement |
| Rich Error Messages | High | Medium | P0 | Design ready |
| Graceful Fallbacks | High | Medium | P1 | Design ready |
| JSON Output Mode | Medium | Low | P1 | Easy addition |
| Prerequisites Script | Medium | Medium | P2 | Needs design |
| Feature Flags | Low | High | P3 | Future |

## Pattern 1: Validation Command (--validate) 🎯

### What Spec-Kit Does
```bash
# Their commands run scripts that validate prerequisites
/speckit.specify
  → runs check-prerequisites.sh --json
  → validates environment before proceeding
```

### What We Can Replicate

Add `--validate` flag that checks:

```bash
claude-hook-advisor --validate

Validating claude-hook-advisor installation...

✓ Binary Installation
  ✓ Version: 1.0.0
  ✓ Location: ~/.cargo/bin/claude-hook-advisor
  ✓ Permissions: Executable

✓ Configuration
  ✓ Config file: .claude-hook-advisor.toml
  ✓ Valid TOML: Yes
  ✓ History enabled: true

✓ Database
  ✓ Path: ~/.claude-hook-advisor/bash-history.db
  ✓ Writable: Yes
  ✓ Schema: Up to date
  ✓ Records: 1,234

⚠ Hooks (Not validated - requires Claude Code)
  Note: Hook registration cannot be validated from CLI
  Check: .claude/settings.json should list plugin path

✓ All checks passed!

Ready to use claude-hook-advisor.
```

### Implementation

```rust
// src/cli.rs
if matches.get_flag("validate") {
    run_validation(config_path)?;
    return Ok(());
}

// src/validation.rs (new file)
pub fn run_validation(config_path: &str) -> Result<()> {
    println!("Validating claude-hook-advisor installation...\n");

    let mut all_passed = true;

    // Check 1: Binary
    all_passed &= validate_binary()?;

    // Check 2: Config
    all_passed &= validate_config(config_path)?;

    // Check 3: Database
    all_passed &= validate_database(config_path)?;

    // Check 4: Hooks (informational only)
    show_hook_info();

    if all_passed {
        println!("✓ All checks passed!\n");
        println!("Ready to use claude-hook-advisor.");
    } else {
        println!("✗ Some checks failed. See above for details.");
        std::process::exit(1);
    }

    Ok(())
}

fn validate_binary() -> Result<bool> {
    println!("✓ Binary Installation");
    println!("  ✓ Version: {}", env!("CARGO_PKG_VERSION"));
    println!("  ✓ Location: {}", env::current_exe()?.display());
    println!();
    Ok(true)
}

fn validate_config(config_path: &str) -> Result<bool> {
    println!("Configuration");

    if !Path::new(config_path).exists() {
        println!("  ✗ Config file not found: {}", config_path);
        println!("    Suggestion: Run 'claude-hook-advisor --install'");
        println!();
        return Ok(false);
    }

    match crate::config::load_config(config_path) {
        Ok(config) => {
            println!("  ✓ Config file: {}", config_path);
            println!("  ✓ Valid TOML: Yes");

            if let Some(hist) = &config.command_history {
                println!("  ✓ History enabled: {}", hist.enabled);
            } else {
                println!("  ⚠ History not configured");
            }
            println!();
            Ok(true)
        }
        Err(e) => {
            println!("  ✗ Invalid config: {}", e);
            println!("    Check TOML syntax in {}", config_path);
            println!();
            Ok(false)
        }
    }
}

fn validate_database(config_path: &str) -> Result<bool> {
    println!("Database");

    let config = match crate::config::load_config(config_path) {
        Ok(c) => c,
        Err(_) => return Ok(false),
    };

    let Some(hist_config) = config.command_history else {
        println!("  ⚠ History not enabled in config");
        println!();
        return Ok(true); // Not an error if history is disabled
    };

    if !hist_config.enabled {
        println!("  ⚠ History disabled in config");
        println!();
        return Ok(true);
    }

    let db_path = expand_tilde(&hist_config.log_file)?;

    if !db_path.exists() {
        println!("  ⚠ Database does not exist yet: {}", db_path.display());
        println!("    Will be created on first command");
        println!();
        return Ok(true);
    }

    // Try to open database
    match crate::history::init_database(&db_path) {
        Ok(conn) => {
            println!("  ✓ Path: {}", db_path.display());
            println!("  ✓ Writable: Yes");

            // Get record count
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM commands", [], |row| row.get(0))
                .unwrap_or(0);

            println!("  ✓ Records: {}", count);
            println!();
            Ok(true)
        }
        Err(e) => {
            println!("  ✗ Cannot open database: {}", e);
            println!("    Path: {}", db_path.display());
            println!("    Suggestion: Check file permissions");
            println!();
            Ok(false)
        }
    }
}

fn show_hook_info() {
    println!("⚠ Hooks (Cannot validate from CLI)");
    println!("  Note: Hook registration requires Claude Code");
    println!("  Check: .claude/settings.json should list plugin path");
    println!("  Test: Run a command in Claude Code and check history");
    println!();
}
```

**Effort:** ~2 hours
**Value:** High - Makes debugging much easier
**Priority:** P0 - Should do first

---

## Pattern 2: Rich Error Messages 🎯

### What Spec-Kit Does

Their scripts always provide:
- What failed
- Why it failed
- How to fix it
- Where to learn more

### What We Can Replicate

```rust
// Current error handling
if let Err(e) = log_command(&conn, &record) {
    eprintln!("Failed to log command: {}", e);
}

// With rich error handling
if let Err(e) = log_command(&conn, &record) {
    print_error(ErrorContext {
        what: "Failed to log command to history database",
        why: &e.to_string(),
        path: Some(db_path.to_string()),
        suggestion: "Check that the database file is writable",
        fix_command: Some(format!("chmod 644 {}", db_path.display())),
        docs_url: Some("https://github.com/sirmews/claude-hook-advisor#troubleshooting"),
    });
}

// Helper function
pub fn print_error(ctx: ErrorContext) {
    eprintln!("❌ {}", ctx.what);
    eprintln!("   Reason: {}", ctx.why);

    if let Some(path) = ctx.path {
        eprintln!("   Path: {}", path);
    }

    if let Some(suggestion) = ctx.suggestion {
        eprintln!("   Suggestion: {}", suggestion);
    }

    if let Some(fix) = ctx.fix_command {
        eprintln!("   Fix: {}", fix);
    }

    if let Some(docs) = ctx.docs_url {
        eprintln!("   Docs: {}", docs);
    }
}

struct ErrorContext<'a> {
    what: &'a str,
    why: &'a str,
    path: Option<String>,
    suggestion: &'a str,
    fix_command: Option<String>,
    docs_url: Option<&'a str>,
}
```

**Effort:** ~4 hours (add to all error sites)
**Value:** High - Users can self-diagnose
**Priority:** P0 - Do alongside validation

---

## Pattern 3: Graceful Fallbacks 🎯

### What Spec-Kit Does

```bash
# Multiple strategies for finding things
get_repo_root() {
    if git rev-parse --show-toplevel >/dev/null 2>&1; then
        git rev-parse --show-toplevel
    else
        # Fallback to script location
        local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        (cd "$script_dir/../../.." && pwd)
    fi
}
```

### What We Can Replicate

```rust
// Database initialization with fallback strategy
pub fn init_database_with_fallbacks(config: &Config) -> Result<Option<Connection>> {
    let hist_config = match &config.command_history {
        Some(cfg) if cfg.enabled => cfg,
        _ => return Ok(None), // History disabled
    };

    // Strategy 1: Try configured path
    let primary_path = expand_tilde(&hist_config.log_file)?;
    if let Ok(conn) = try_init_database(&primary_path) {
        return Ok(Some(conn));
    }

    // Strategy 2: Try creating parent directories
    if let Ok(()) = create_parent_dirs(&primary_path) {
        if let Ok(conn) = try_init_database(&primary_path) {
            warn!("Created parent directory for database");
            return Ok(Some(conn));
        }
    }

    // Strategy 3: Try fallback location
    let fallback_path = PathBuf::from(env::var("HOME")?)
        .join(".cache/claude-hook-advisor/history.db");

    if let Ok(conn) = try_init_database(&fallback_path) {
        warn!("Using fallback database location: {}", fallback_path.display());
        warn!("Primary location failed: {}", primary_path.display());
        return Ok(Some(conn));
    }

    // Strategy 4: Continue without persistence
    warn!("Could not initialize history database");
    warn!("Commands will not be logged");
    warn!("Check configuration and file permissions");

    Ok(None) // Return None, don't crash
}

// In hooks - use gracefully
fn handle_post_tool_use(config: &Config, hook_input: &HookInput) -> Result<()> {
    // ... other code ...

    // If database fails, log warning but continue
    match init_database_with_fallbacks(config)? {
        Some(conn) => {
            // Use database
            let _ = update_command_status(&conn, ...);
        }
        None => {
            // No database, continue silently
            // Hook should never crash
        }
    }

    Ok(())
}
```

**Effort:** ~3 hours
**Value:** High - Improves reliability significantly
**Priority:** P1 - Do after validation

---

## Pattern 4: JSON Output Mode

### What Spec-Kit Does

All scripts have `--json` flag for structured output

### What We Can Replicate

```rust
// Add --json flag
if matches.get_flag("json") {
    output_history_json(&records)?;
} else {
    output_history_formatted(&records)?;
}

fn output_history_json(records: &[CommandRecord]) -> Result<()> {
    let output = serde_json::json!({
        "records": records.iter().map(|r| serde_json::json!({
            "timestamp": r.timestamp,
            "command": r.command,
            "status": r.status,
            "exit_code": r.exit_code,
            "cwd": r.cwd,
            "session_id": r.session_id,
            "was_replaced": r.was_replaced,
            "original_command": r.original_command,
        })).collect::<Vec<_>>(),
        "total": records.len(),
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
```

**Effort:** ~1 hour
**Value:** Medium - Nice for tooling/scripting
**Priority:** P1 - Easy win

---

## Pattern 5: Prerequisites Checking Script

### What Spec-Kit Does

Dedicated bash scripts that validate environment before proceeding

### What We Can Replicate

Create `scripts/validate.sh` that slash commands can call:

```bash
#!/usr/bin/env bash
# scripts/validate.sh

set -e

# Check binary
if ! command -v claude-hook-advisor &> /dev/null; then
    echo '{"status": "error", "message": "Binary not found in PATH"}'
    exit 1
fi

# Check version
VERSION=$(claude-hook-advisor --version 2>&1 | grep -o '[0-9.]*')

# Check config
if [ ! -f ".claude-hook-advisor.toml" ]; then
    echo '{"status": "warning", "message": "No config file found"}'
fi

# Output JSON
cat <<EOF
{
  "status": "ok",
  "binary": {
    "found": true,
    "version": "$VERSION",
    "path": "$(which claude-hook-advisor)"
  },
  "config": {
    "found": $([ -f ".claude-hook-advisor.toml" ] && echo "true" || echo "false")
  }
}
EOF
```

Update `/validate` command to use it:

```markdown
---
description: Validate claude-hook-advisor installation
allowed-tools: Bash(scripts/*)
scripts:
  sh: scripts/validate.sh --json
---

!{SCRIPT}

The validation results above show your installation status.
Let me analyze and help resolve any issues.
```

**Effort:** ~2 hours
**Value:** Medium - Complements --validate flag
**Priority:** P2 - After core validation works

---

## Pattern 6: Feature Flags (Future)

### What Spec-Kit Does

Conditional execution based on available resources

### What We Could Add

```toml
[behavior]
fail_fast = false           # Continue on errors
verbose_errors = true       # Show detailed error context
auto_retry = true           # Retry transient failures
max_retries = 3

[fallback]
locations = [
    "~/.cache/claude-hook-advisor/history.db",
    "/tmp/claude-hook-advisor-history.db"
]
```

**Effort:** ~6 hours
**Value:** Low - Advanced feature
**Priority:** P3 - Future enhancement

---

## Implementation Roadmap

### Phase 1: Foundation (Week 1) 🎯
**Effort:** ~8 hours
**Value:** Immediate impact on reliability

1. ✅ Implement `--validate` flag (2h)
2. ✅ Add rich error messages module (2h)
3. ✅ Apply rich errors to key failure points (2h)
4. ✅ Add graceful database fallbacks (2h)

**Deliverables:**
- Users can run `claude-hook-advisor --validate`
- All errors show context + suggestions
- Database failures don't crash hooks
- Clear path to self-diagnosis

### Phase 2: Polish (Week 2)
**Effort:** ~3 hours
**Value:** Nice-to-have improvements

1. ⏳ Add `--json` flag to history command (1h)
2. ⏳ Create validation bash script (1h)
3. ⏳ Update `/validate` to use script (1h)

**Deliverables:**
- JSON output for tooling
- Validation script for slash commands
- Complete validation workflow

### Phase 3: Future (Later)
**Effort:** ~6 hours
**Value:** Advanced features

1. ⏳ Feature flags system
2. ⏳ Retry logic
3. ⏳ Performance optimizations

---

## Files to Create/Modify

### New Files
- `src/validation.rs` - Validation command logic
- `src/error_context.rs` - Rich error handling
- `scripts/validate.sh` - Prerequisites checking script

### Modified Files
- `src/cli.rs` - Add --validate and --json flags
- `src/hooks.rs` - Add graceful fallbacks
- `src/history.rs` - Use rich error messages
- `Cargo.toml` - Add serde_json dependency

---

## Success Criteria

### Phase 1 Complete When:
- ✅ `claude-hook-advisor --validate` runs successfully
- ✅ All database errors show helpful context
- ✅ Hooks never crash, even on errors
- ✅ Users can self-diagnose issues

### Phase 2 Complete When:
- ✅ JSON output works for all commands
- ✅ Validation script integrated
- ✅ `/validate` command fully functional

---

## Next Steps

**Recommendation:** Start with Phase 1, Pattern 1 (--validate flag)

1. Create `src/validation.rs` module
2. Implement validation checks
3. Add --validate flag to CLI
4. Test with various scenarios
5. Update documentation

**Estimated time to first working version:** 2-3 hours

Ready to start implementation?
