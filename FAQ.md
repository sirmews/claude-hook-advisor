# Claude Hook Advisor - Frequently Asked Questions

## 🔧 System Behavior

### Q: When does the confidence decay and maintenance run?

**A:** Maintenance runs **lazily** during PostToolUse hooks, not on a schedule:

- **Trigger**: Only when commands are executed (PostToolUse hook fires)
- **Frequency**: At most once per day
- **Activity Threshold**: Requires at least 10 command executions since last maintenance
- **What it does**: Applies confidence decay (2% per week), evaluates never-suggest candidates

**Important**: If you don't use the system for weeks, decay won't happen until you start running commands again. This is intentional to avoid unnecessary processing.

### Q: Why do I see `[DEBUG] Hook output does not start with {, treating as plain text`?

**A:** This is **normal behavior** for tracking hooks:

- **PreToolUse**: Returns JSON to block/suggest commands → Claude processes the suggestion
- **UserPromptSubmit/PostToolUse**: Return plain text for logging → Claude shows this debug message

The debug message just means "this hook didn't try to block anything, it was just logging info."

## 🧠 Learning System

### Q: How does natural language learning work?

**A:** The system recognizes 8 different patterns in your prompts:

```
✅ "use bun instead of npm"          → Direct replacement
✅ "I prefer pnpm over yarn"         → Preference indication  
✅ "always use deno instead of node" → Always pattern
✅ "for this project, use bun"       → Context-specific
✅ "in node projects, use bun"       → Project-type specific
✅ "never suggest npm for yarn"      → Never-suggest blacklist
✅ "replace all npm with bun"        → Global replacement
✅ "switch from npm to bun"          → Migration pattern
```

Learning happens immediately when you say these phrases to Claude Code.

### Q: How does confidence adjustment work?

**A:** The system tracks command success and adjusts confidence:

1. **Initial Confidence**: New mappings start at 0.6 (60%)
2. **Success**: Confidence increases by 0.1 per successful execution
3. **Failure**: Confidence decreases by 0.15 per failed execution  
4. **Time Decay**: Reduces by 2% per week (max 30% decay, min 10% confidence)
5. **Never-Suggest**: Commands with consistently low success move to blacklist

### Q: What's the difference between static and learned mappings?

**A:** Two types of configuration:

**Static Mappings** (`.claude-hook-advisor.toml`):
- Manual configuration you write
- Always active (no confidence system)
- Higher priority than learned mappings

**Learned Mappings** (auto-generated):
- Created from natural language ("use bun instead of npm")
- Subject to confidence tracking and decay
- Can be promoted to never-suggest if they fail consistently

## ⚙️ Configuration & Management

### Q: How do I share learned configurations with my team?

**A:** Use the export/import system:

```bash
# Export your learned preferences
claude-hook-advisor --export-config team-preferences.toml

# Team members import them
claude-hook-advisor --import-config team-preferences.toml
```

This shares learned mappings, confidence scores, and never-suggest lists.

### Q: Can I have different settings per project?

**A:** Yes, the system supports multiple configuration levels:

1. **Static Config**: Per-project `.claude-hook-advisor.toml` files
2. **Learned Global**: Applies to all projects  
3. **Learned Project**: Project-specific learned preferences
4. **Learned Context**: Context-aware mappings

Priority order: never-suggest → static → learned (by confidence level)

### Q: How do I reset or clean up learned data?

**A:** Several options:

```bash
# Reset all learned data (keeps static config)
claude-hook-advisor --reset-learning

# View what's been learned
claude-hook-advisor --list-learned

# Generate analytics report
claude-hook-advisor --confidence-report

# Export before reset (backup)
claude-hook-advisor --export-config backup.toml
```

## 🔍 Troubleshooting

### Q: Commands aren't being suggested - what's wrong?

**A:** Check these common issues:

1. **Hook Setup**: Ensure all 3 hooks are configured:
   ```json
   {
     "hooks": {
       "PreToolUse": { "Bash": "claude-hook-advisor --hook" },
       "UserPromptSubmit": { ".*": "claude-hook-advisor --hook" },
       "PostToolUse": { "Bash": "claude-hook-advisor --hook" }
     }
   }
   ```

2. **Confidence Threshold**: Low-confidence mappings are filtered out
   ```bash
   claude-hook-advisor --confidence-report  # Check confidence levels
   ```

3. **Never-Suggest**: Commands might be blacklisted due to failures
   ```bash
   claude-hook-advisor --list-learned  # Check never-suggest section
   ```

4. **Word Boundaries**: System uses exact word matching
   - ✅ `npm install` matches `npm` mapping
   - ❌ `npm-check` does NOT match `npm` mapping

### Q: Why do I see `❌ Command correlation` in the logs?

**A:** This indicates PostToolUse tracking issues:

- **❌**: Usually means `exit_code: None` or execution correlation failed
- **✅**: Successful tracking and correlation

**Fix**: This is usually harmless - the system assumes success if PostToolUse fires (it only fires on successful commands).

### Q: Learning isn't working from my prompts

**A:** Ensure UserPromptSubmit hook is configured:

```json
"UserPromptSubmit": { ".*": "claude-hook-advisor --hook" }
```

Check that your phrases match the learning patterns:
- ✅ "use bun instead of npm" 
- ❌ "bun is better than npm" (too indirect)

## 📊 Analytics & Reporting

### Q: How can I see what the system has learned?

**A:** Use the CLI management commands:

```bash
# List all learned mappings with confidence scores
claude-hook-advisor --list-learned

# Detailed analytics report
claude-hook-advisor --confidence-report

# Export everything to a file for inspection
claude-hook-advisor --export-config inspection.toml
```

### Q: How do I know if suggestions are working well?

**A:** The confidence report shows effectiveness:

- **High Confidence (>0.8)**: Consistently successful suggestions
- **Medium Confidence (0.4-0.8)**: Mixed results, still learning
- **Low Confidence (<0.4)**: Poor performance, may be moved to never-suggest
- **Never-Suggest**: Commands that consistently failed

## 🚀 Performance & Technical

### Q: Does this slow down Claude Code?

**A:** Minimal performance impact:

- **Fast Regex**: Word-boundary matching is very efficient
- **Lazy Maintenance**: Only runs when needed (once/day max)
- **Single Binary**: No external dependencies at runtime
- **Atomic Operations**: Configuration updates are race-condition-free

### Q: What happens if the hook fails?

**A:** Graceful degradation:

- **Configuration Missing**: Commands proceed normally with warning to stderr
- **Parse Errors**: Commands proceed with error logged to stderr  
- **Hook Timeout**: Claude Code continues with original command
- **Malformed JSON**: Plain text response, doesn't block execution

The system is designed to never break your workflow - failures are non-blocking.

## 🔄 Migration & Updates

### Q: I'm upgrading from v0.1.0 - what changes?

**A:** Automatic migration:

- **Static Config**: No changes needed - `.claude-hook-advisor.toml` works as before
- **New Features**: Learning system is opt-in via additional hooks
- **Backwards Compatibility**: v0.1.0 behavior preserved exactly
- **Configuration Format**: Automatically upgraded when you use new features

### Q: Can I use this without the learning system?

**A:** Yes! Configure only the PreToolUse hook:

```json
{
  "hooks": {
    "PreToolUse": { "Bash": "claude-hook-advisor --hook" }
  }
}
```

This gives you v0.1.0 behavior - static mappings only, no learning or tracking.

---

## 💡 Pro Tips

- **Start Simple**: Begin with static config, add learning hooks when ready
- **Team Adoption**: Use export/import to share successful configurations  
- **Monitor Confidence**: Run `--confidence-report` periodically to see what's working
- **Debug Issues**: Check Claude Code debug output for hook execution details
- **Performance**: The system learns and improves over time - give it a few days

---

**Need more help?** Check the [GitHub Issues](https://github.com/sirmews/claude-hook-advisor/issues) or create a new issue with your specific question.

---

*Last updated: 2025-01-24 | Version: 0.2.0*  
*#claude-hook-advisor #faq #troubleshooting #learning-system*