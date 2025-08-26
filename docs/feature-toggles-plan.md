---
title: "Feature Toggles Configuration Plan"
description: "Design document for implementing granular feature toggles in claude-hook-advisor configuration"
date: "2025-08-25"
tags: ["#configuration", "#features", "#design", "#toml"]
---

# Feature Toggles Configuration Plan

## Overview

This document outlines the plan for implementing granular feature toggles in claude-hook-advisor configuration. The goal is to provide users fine-grained control over each feature while maintaining backward compatibility and performance.

## Current State

Currently, all features are hardcoded and always enabled:
- Command suggestions (PreToolUse hook)
- Directory resolution (UserPromptSubmit hook)
- Documentation guidance (UserPromptSubmit hook)
- Documentation validation (PostToolUse hook)
- Execution tracking (PostToolUse hook)

## Proposed Configuration Schema

### TOML Structure

```toml
[commands]
npm = "bun"
yarn = "bun"

[semantic_directories] 
docs = "~/Documents/Documentation"
project_docs = "~/Documents/Documentation/my-project"

[features]
# Core features
command_suggestions = true          # PreToolUse command mapping
directory_resolution = true        # UserPromptSubmit directory aliases

# Documentation features  
documentation_guidance = true      # UserPromptSubmit doc keyword detection
documentation_validation = true   # PostToolUse markdown validation

# Search features
hashtag_search_advisory = true    # PreToolUse hashtag search pattern guidance

# Analytics features
execution_tracking = false        # PostToolUse command analytics
verbose_logging = false           # Detailed stderr output

[documentation_standards]
# Future: custom template paths and validation rules
# template_path = "~/Documents/Documentation/TEMPLATE.md"
# enforce_frontmatter = true
# require_timestamps = true
```

## Implementation Strategy

### 1. Backward Compatibility
- Default all existing features to `true` for existing users
- New features default to `false` or sensible defaults
- Missing `[features]` section means all features enabled

### 2. Config Structure Changes

```rust
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub commands: HashMap<String, String>,
    
    #[serde(default)]
    pub semantic_directories: HashMap<String, String>,
    
    #[serde(default)]
    pub features: FeatureFlags,
    
    #[serde(default)]
    pub documentation_standards: DocumentationConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FeatureFlags {
    #[serde(default = "default_true")]
    pub command_suggestions: bool,
    
    #[serde(default = "default_true")]
    pub directory_resolution: bool,
    
    #[serde(default = "default_true")]
    pub documentation_guidance: bool,
    
    #[serde(default = "default_true")]
    pub documentation_validation: bool,
    
    #[serde(default = "default_true")]
    pub hashtag_search_advisory: bool,
    
    #[serde(default = "default_false")]
    pub execution_tracking: bool,
    
    #[serde(default = "default_false")]
    pub verbose_logging: bool,
}

fn default_true() -> bool { true }
fn default_false() -> bool { false }
```

### 3. Hook Function Modifications

Each hook function will check feature flags before executing:

```rust
pub fn handle_user_prompt_submit(input: &HookInput, config: &Config) -> Result<()> {
    // Directory resolution
    if config.features.directory_resolution {
        if let Some(resolved) = resolve_directories(&input.prompt, config)? {
            println!("{}", resolved);
        }
    }
    
    // Documentation guidance
    if config.features.documentation_guidance {
        if contains_documentation_keywords(&input.prompt) {
            display_documentation_standards()?;
        }
    }
    
    Ok(())
}

pub fn handle_pre_tool_use(input: &HookInput, config: &Config) -> Result<()> {
    // Command suggestions
    if config.features.command_suggestions {
        // Existing command suggestion logic...
    }
    
    // Hashtag search advisory
    if config.features.hashtag_search_advisory {
        if let Some(command) = &input.tool_input.command {
            if let Some(search_terms) = command.strip_prefix("hashtag search ") {
                // Provide ripgrep pattern guidance
                return provide_hashtag_search_guidance(search_terms);
            }
        }
    }
    
    Ok(())
}

pub fn handle_post_tool_use(input: &HookInput, config: &Config) -> Result<()> {
    // Analytics tracking
    if config.features.execution_tracking {
        track_command_execution(input)?;
    }
    
    // Documentation validation
    if config.features.documentation_validation {
        validate_markdown_files(input)?;
    }
    
    Ok(())
}
```

## Benefits

### 1. User Control
- **Selective Disable**: Turn off just documentation validation while keeping guidance
- **Performance**: Skip expensive operations (regex matching, file validation) when disabled
- **Privacy**: Disable execution tracking for sensitive environments

### 2. Development Benefits
- **Testing**: Easy to test individual features in isolation
- **Debugging**: Disable noisy features during development
- **Future-Ready**: Simple pattern for adding new features

### 3. Use Cases

#### Minimal Setup (Performance-focused)
```toml
[features]
command_suggestions = true
directory_resolution = true
documentation_guidance = false
documentation_validation = false
execution_tracking = false
```

#### Documentation-focused Setup
```toml
[features]
command_suggestions = false
directory_resolution = true
hashtag_search_advisory = true
documentation_guidance = true
documentation_validation = true
execution_tracking = false
```

#### Full Analytics Setup
```toml
[features]
command_suggestions = true
directory_resolution = true
hashtag_search_advisory = true
documentation_guidance = true
documentation_validation = true
execution_tracking = true
verbose_logging = true
```

## Implementation Tasks

1. **Update Config struct** to include `FeatureFlags` and `DocumentationConfig`
2. **Modify hook functions** to check feature flags before executing
3. **Update config generation** to include `[features]` section with defaults
4. **Add comprehensive tests** for feature flag behavior
5. **Update documentation** with feature toggle examples
6. **Ensure backward compatibility** with existing config files

## Future Extensions

### Custom Documentation Standards
```toml
[documentation_standards]
template_path = "~/Documents/Documentation/TEMPLATE.md"
enforce_frontmatter = true
require_timestamps = true
require_tags = true
custom_tag_prefix = "#"
```

### Per-Tool Feature Control
```toml
[features.tools]
bash_command_suggestions = true
write_documentation_validation = true
edit_documentation_validation = false
```

### Environment-based Toggles
```toml
[features.environments]
development = { documentation_validation = false }
production = { execution_tracking = true }
```

## Migration Strategy

1. **Phase 1**: Add feature flags with all defaults to `true`
2. **Phase 2**: Update hook functions to respect flags
3. **Phase 3**: Add new features with sensible defaults
4. **Phase 4**: Document usage patterns and best practices

This approach ensures smooth adoption while providing the flexibility users need for different use cases and environments.

---

*Last updated: 2025-08-25*
*Tags: #configuration #features #design #toml*