---
description: Show command history for a specific session (usage: /history-session <session-id>)
allowed-tools: Bash(claude-hook-advisor:*)
---

!claude-hook-advisor --history --session "$1" --limit 100

The output above shows all commands from session "$1". This includes:
- All commands attempted in this session
- Success and failure status for each
- Complete timeline of command execution

Let me analyze this session's activity and help you understand what happened.
