---
title: "Hashtag Search Integration Plan"
description: "Simple advisory pattern for integrating hashtag search functionality with claude-hook-advisor"
date: "2025-08-26"
tags: ["#hashtag-search", "#integration", "#ripgrep", "#advisory", "#hooks"]
---

# Hashtag Search Integration Plan

## Overview

This document outlines a simple, elegant approach to integrate hashtag search functionality into claude-hook-advisor through an **advisory pattern**. Instead of implementing complex search logic, we provide guidance to Claude about ripgrep patterns and let existing hooks and Claude's intelligence handle the rest.

## Core Philosophy

**Advise, Don't Execute** - The system detects hashtag search intent and provides pattern guidance, letting Claude make the final decisions about command construction and execution.

## Architecture

### Simple Advisory Pattern

```rust
// User intent: "hashtag search authentication async"
// System response: "Use ripgrep with patterns like: rg '#authentication|#async' [directories] --glob='*.md' -n -C 2"
// Claude decides: Fills in directories, modifies flags, executes command
```

### Integration Points

1. **PreToolUse Hook**: Detects `hashtag search` commands
2. **Directory Resolution Hook**: Provides available search directories  
3. **Claude Intelligence**: Combines guidance into final command
4. **Existing Tools**: Uses ripgrep directly (no wrapper tools needed)

## Implementation Strategy

### 1. Command Pattern Detection

**Trigger**: Commands starting with `hashtag search`

```rust
fn handle_pre_tool_use(input: &HookInput, config: &Config) -> Result<()> {
    if let Some(command) = &input.tool_input.command {
        if let Some(search_terms) = command.strip_prefix("hashtag search ") {
            // Extract terms and build guidance
            let terms: Vec<&str> = search_terms.split_whitespace().collect();
            let hashtag_pattern = terms
                .iter()
                .map(|term| format!("#{}", term))
                .collect::<Vec<_>>()
                .join("|");
            
            return Ok(HookOutput {
                decision: "block".to_string(),
                reason: format!(
                    "For hashtag search, use ripgrep with patterns like: rg '{}' [directories] --glob='*.md' --glob='*.txt' -n -C 2",
                    hashtag_pattern
                ),
                replacement_command: None, // Advisory only
            });
        }
    }
    Ok(())
}
```

### 2. Pattern Building Logic

**Input Processing**:
- `hashtag search auth async patterns` → `["auth", "async", "patterns"]`
- **Pattern Generation**: `#auth|#async|#patterns`  
- **Guidance Format**: `rg '#auth|#async|#patterns' [directories] --glob='*.md' --glob='*.txt' -n -C 2`

### 3. Hook Ecosystem Integration

**Composable Behavior**:
1. **Hashtag Search Detection** → Provides ripgrep pattern guidance
2. **Directory Resolution Hook** → Suggests available directories when Claude asks
3. **Claude Assembly** → Combines guidance into executable command
4. **Command Execution** → Standard ripgrep execution with full functionality

## User Experience Flow

### Example 1: Basic Hashtag Search

**User Action**: Claude attempts `hashtag search authentication security`

**Hook Response**: 
```
For hashtag search, use ripgrep with patterns like: 
rg '#authentication|#security' [directories] --glob='*.md' --glob='*.txt' -n -C 2
```

**Claude Reasoning**: "I need directories for this search. Let me check what's available..."

**Claude Action**: `rg '#authentication|#security' ~/Documents/Documentation ~/Documents/Documentation/my-project --glob='*.md' --glob='*.txt' -n -C 2`

### Example 2: Context-Aware Search

**User Prompt**: "Look in the project docs for hashtag search async patterns"

**Hook Pipeline**:
1. Directory resolution detects "project docs" → `~/Documents/Documentation/my-project`
2. Hashtag search guidance for "async patterns" → `#async|#patterns`

**Claude Final Command**: `rg '#async|#patterns' ~/Documents/Documentation/my-project --glob='*.md' -n -C 3`

## Benefits

### 1. Simplicity
- **Minimal Code**: Just pattern building, no complex search logic
- **No Dependencies**: Uses ripgrep directly (already available)
- **Clear Intent**: "hashtag search" explicitly indicates desired functionality

### 2. Flexibility  
- **Claude Control**: LLM makes final command decisions
- **Customizable**: Claude can modify patterns, directories, flags as needed
- **Composable**: Works seamlessly with existing hook ecosystem

### 3. Performance
- **No Overhead**: No wrapper tools or additional processes
- **Direct Execution**: Full ripgrep performance and capabilities
- **Efficient Parsing**: Simple string operations for pattern building

### 4. Maintainability
- **Leverages Existing**: Uses current directory resolution and hook infrastructure
- **Future-Proof**: Easy to extend with additional patterns or guidance
- **Testable**: Simple input/output behavior for testing

## Configuration Integration

### Feature Flag Support
```toml
[features]
hashtag_search_advisory = true    # Enable hashtag search guidance
```

### Existing Configuration Reuse
```toml
[semantic_directories]
# These directories become available for Claude to use in searches
docs = "~/Documents/Documentation"
project_docs = "~/Documents/Documentation/my-project"
claude_docs = "~/Documents/Documentation/claude"
```

## Implementation Tasks

### Phase 1: Core Advisory Pattern (30 minutes)
1. **Add hashtag search detection** (10 min)
   - Detect commands starting with "hashtag search"
   - Extract search terms from command

2. **Add pattern building logic** (10 min)
   - Convert terms to hashtag patterns (#term|#term)
   - Format ripgrep guidance message

3. **Integrate with PreToolUse hook** (10 min)
   - Add to existing hook logic
   - Ensure proper blocking and guidance response

### Phase 2: Testing and Documentation (20 minutes)
1. **Add test cases** (10 min)
   - Test pattern building with various inputs
   - Test integration with hook system

2. **Update documentation** (10 min)
   - Add to CLAUDE.md usage examples
   - Document new functionality in README

## Advanced Scenarios

### Multiple Term Patterns
- **Input**: `hashtag search rust async tokio performance`
- **Pattern**: `#rust|#async|#tokio|#performance`
- **Benefits**: Finds documents tagged with any of the terms

### Integration with Directory Context  
- **User in project directory**: Automatic preference for project-specific docs
- **Semantic directory resolution**: "project docs" → actual paths
- **Multi-directory search**: Searches across all relevant documentation

### Error Handling
- **Empty search terms**: Provide general hashtag search guidance
- **Invalid characters**: Clean terms before pattern building  
- **Missing directories**: Graceful handling when directories don't exist

## Future Enhancements

### 1. Pattern Sophistication
- **AND patterns**: `#rust AND #async` for documents with both tags
- **Exclusion patterns**: `#rust NOT #beginner` for advanced content
- **Fuzzy matching**: Handle tag variations and typos

### 2. Context Intelligence
- **Recent document weighting**: Prefer recently modified docs
- **Project context awareness**: Different search behavior based on current project
- **Learning from usage**: Improve pattern suggestions based on successful searches

### 3. Result Enhancement
- **Preview snippets**: Show relevant content excerpts
- **Tag frequency**: Display how many times tags appear
- **Cross-references**: Find related tags and documents

## Conclusion

The hashtag search integration provides powerful documentation discovery capabilities through a simple advisory pattern. By leveraging existing infrastructure and keeping Claude in control, we achieve maximum functionality with minimal complexity.

The approach is:
- **Lightweight**: Simple pattern building and guidance
- **Powerful**: Full ripgrep capabilities with intelligent integration
- **Extensible**: Easy to enhance with additional features
- **User-Friendly**: Natural "hashtag search" command interface

This integration transforms claude-hook-advisor into an intelligent documentation discovery system while maintaining its core simplicity and reliability.

---

*Last updated: 2025-08-26*
*Tags: #hashtag-search #integration #ripgrep #advisory #hooks*