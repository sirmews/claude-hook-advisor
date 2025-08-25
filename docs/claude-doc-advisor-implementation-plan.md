---
title: "Claude Doc Advisor - Implementation Plan"
created_at: "2025-08-25"
updated_at: "2025-08-25"
tags: ['#claude-doc-advisor', '#implementation', '#tdd', '#standards-enforcement']
description: "Focused implementation plan for claude-doc-advisor as a documentation standards enforcement library"
---

# Claude Doc Advisor - Implementation Plan

## Mission Statement

Create a **focused library crate** that enforces documentation standards by providing guidance before document creation and validating compliance afterward. The library should fail silently with logged errors to maintain hook resilience.

## Core Architecture

### Purpose
- **NOT a document generator** - just standards enforcement
- **NOT a CLI tool** - pure library for integration
- **NOT complex templating** - generic standards based on TEMPLATE.md

### Two-Function API Contract

```rust
// claude-doc-advisor/src/lib.rs
pub fn get_documentation_standards() -> Result<DocumentationStandards, ValidationError>;
pub fn validate_document_compliance<P: AsRef<Path>>(path: P) -> Result<ComplianceResult, ValidationError>;
```

## Project Structure

```
claude-doc-advisor/
├── Cargo.toml                 // Library crate only
├── src/
│   ├── lib.rs                 // Public API (2 functions)
│   ├── standards.rs           // DocumentationStandards type
│   ├── validator.rs           // Compliance validation logic
│   ├── error.rs               // Error types
│   └── parsing.rs             // YAML frontmatter parsing
├── tests/
│   ├── integration.rs         // API contract tests
│   └── fixtures/              // Test markdown files
│       ├── compliant/         // Documents that pass validation
│       └── non_compliant/     // Documents with various issues
└── README.md                  // Library usage documentation
```

## Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
chrono = { version = "0.4", features = ["serde"] }
regex = "1.0"
anyhow = "1.0"
thiserror = "1.0"

[dev-dependencies]
tempfile = "3.0"
test-case = "3.0"
```

## Core Data Structures

### DocumentationStandards

```rust
#[derive(Debug, Clone)]
pub struct DocumentationStandards {
    pub required_frontmatter_fields: Vec<String>,
    pub date_format: String,
    pub tag_format_rules: TagFormatRules,
    pub filename_conventions: FilenameRules,
    pub guidance_text: String,
}

impl DocumentationStandards {
    pub fn default_standards() -> Self {
        // Based on ~/Documents/Documentation/TEMPLATE.md
        Self {
            required_frontmatter_fields: vec![
                "title".to_string(),
                "created_at".to_string(),
                "updated_at".to_string(),
                "tags".to_string(),
                "description".to_string(),
            ],
            date_format: "YYYY-MM-DD".to_string(),
            tag_format_rules: TagFormatRules::default(),
            filename_conventions: FilenameRules::kebab_case(),
            guidance_text: Self::generate_guidance_text(),
        }
    }
    
    fn generate_guidance_text() -> String {
        format!(
            "Documentation Standards:\n\
            • Required YAML frontmatter: title, created_at, updated_at, tags, description\n\
            • Date format: YYYY-MM-DD (example: {})\n\
            • Tags must start with # and use kebab-case (#project-name, #guide)\n\
            • Filenames should use kebab-case.md\n\
            • Include Purpose and Content Structure sections\n\
            • Keep documents focused and concise",
            chrono::Utc::now().format("%Y-%m-%d")
        )
    }
}
```

### ComplianceResult

```rust
#[derive(Debug)]
pub struct ComplianceResult {
    pub is_compliant: bool,
    pub issues: Vec<ComplianceIssue>,
    pub suggestions: Vec<String>,
}

impl ComplianceResult {
    pub fn summary(&self) -> String {
        if self.is_compliant {
            "Document meets all standards".to_string()
        } else {
            let error_count = self.issues.iter().filter(|i| i.severity == IssueSeverity::Error).count();
            let warning_count = self.issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count();
            format!("{} errors, {} warnings", error_count, warning_count)
        }
    }
}

#[derive(Debug)]
pub struct ComplianceIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub message: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, PartialEq)]
pub enum IssueSeverity {
    Error,   // Must fix (missing required field)
    Warning, // Should fix (inconsistent formatting)
}

#[derive(Debug)]
pub enum IssueCategory {
    MissingFrontmatter,
    MissingRequiredField(String),
    InvalidDateFormat,
    InvalidTagFormat,
    FilenameConvention,
    StructureIssue,
}
```

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },
    
    #[error("IO error reading file: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid YAML frontmatter: {reason}")]
    InvalidFrontmatter { reason: String },
    
    #[error("YAML parsing error: {0}")]
    YamlParse(#[from] serde_yaml::Error),
    
    #[error("Regex compilation error: {0}")]
    Regex(#[from] regex::Error),
}
```

## Integration with claude-hook-advisor

### Enhanced Hook Functions

```rust
// claude-hook-advisor/src/hooks.rs
use claude_doc_advisor::{get_documentation_standards, validate_document_compliance};

fn handle_user_prompt_submit_with_docs(config: &Config, hook_input: &HookInput) -> Result<()> {
    // Existing directory resolution logic...
    let directory_refs = detect_directory_references(config, prompt);
    
    // NEW: Documentation standards guidance
    if contains_documentation_keywords(prompt) {
        match get_documentation_standards() {
            Ok(standards) => {
                println!("{}", standards.guidance_text);
            }
            Err(e) => {
                // Silent failure with logging
                eprintln!("Failed to load documentation standards: {}", e);
            }
        }
    }
    
    // Continue with existing logic...
    Ok(())
}

fn handle_post_tool_use_with_validation(config: &Config, hook_input: &HookInput) -> Result<()> {
    // Existing command tracking...
    
    // NEW: Validate any markdown files that were created/modified
    if let Some(markdown_files) = extract_markdown_files_from_tool_use(hook_input) {
        for file_path in markdown_files {
            match validate_document_compliance(&file_path) {
                Ok(result) => {
                    if result.is_compliant {
                        println!("✓ Document {} meets standards", file_path.display());
                    } else {
                        println!("⚠ Document {} issues: {}", file_path.display(), result.summary());
                        for issue in &result.issues {
                            println!("  • {}", issue.message);
                        }
                    }
                }
                Err(e) => {
                    // Silent failure with logging
                    eprintln!("Failed to validate {}: {}", file_path.display(), e);
                }
            }
        }
    }
    
    Ok(())
}

fn contains_documentation_keywords(prompt: &str) -> bool {
    let doc_keywords = ["document", "write doc", "create guide", "add documentation", "readme"];
    let prompt_lower = prompt.to_lowercase();
    doc_keywords.iter().any(|keyword| prompt_lower.contains(keyword))
}

fn extract_markdown_files_from_tool_use(hook_input: &HookInput) -> Option<Vec<PathBuf>> {
    // Parse tool_input.command to detect .md file creation/modification
    // Look for patterns like: touch file.md, vim file.md, echo >> file.md
    // Return list of markdown files that were likely modified
    None // Implementation needed
}
```

## Test-Driven Development Plan

### Phase 1: Core API Functions (Week 1)

#### Test Cases for `get_documentation_standards()`

```rust
#[cfg(test)]
mod standards_tests {
    use super::*;
    
    #[test]
    fn test_get_documentation_standards_success() {
        let standards = get_documentation_standards().unwrap();
        assert!(!standards.required_frontmatter_fields.is_empty());
        assert_eq!(standards.date_format, "YYYY-MM-DD");
        assert!(!standards.guidance_text.is_empty());
    }
    
    #[test]
    fn test_standards_required_fields() {
        let standards = get_documentation_standards().unwrap();
        let required = &standards.required_frontmatter_fields;
        
        assert!(required.contains(&"title".to_string()));
        assert!(required.contains(&"created_at".to_string()));
        assert!(required.contains(&"updated_at".to_string()));
        assert!(required.contains(&"tags".to_string()));
        assert!(required.contains(&"description".to_string()));
    }
    
    #[test]
    fn test_guidance_text_format() {
        let standards = get_documentation_standards().unwrap();
        assert!(standards.guidance_text.contains("YAML frontmatter"));
        assert!(standards.guidance_text.contains("YYYY-MM-DD"));
        assert!(standards.guidance_text.contains("kebab-case"));
    }
}
```

#### Test Cases for `validate_document_compliance()`

```rust
#[cfg(test)]
mod validation_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_validate_compliant_document() {
        let temp_file = create_compliant_test_document();
        let result = validate_document_compliance(temp_file.path()).unwrap();
        
        assert!(result.is_compliant);
        assert_eq!(result.issues.len(), 0);
    }
    
    #[test]
    fn test_validate_missing_frontmatter() {
        let temp_file = create_document_without_frontmatter();
        let result = validate_document_compliance(temp_file.path()).unwrap();
        
        assert!(!result.is_compliant);
        assert!(result.issues.iter().any(|i| matches!(i.category, IssueCategory::MissingFrontmatter)));
    }
    
    #[test]
    fn test_validate_missing_required_field() {
        let temp_file = create_document_missing_title();
        let result = validate_document_compliance(temp_file.path()).unwrap();
        
        assert!(!result.is_compliant);
        assert!(result.issues.iter().any(|i| matches!(i.category, IssueCategory::MissingRequiredField(_))));
    }
    
    #[test_case("2025-08-25", true; "valid ISO date")]
    #[test_case("08-25-2025", false; "US date format")]
    #[test_case("2025/08/25", false; "slash separators")]
    #[test_case("invalid-date", false; "completely invalid")]
    fn test_date_format_validation(date_str: &str, should_be_valid: bool) {
        let temp_file = create_document_with_date(date_str);
        let result = validate_document_compliance(temp_file.path()).unwrap();
        
        let has_date_error = result.issues.iter()
            .any(|i| matches!(i.category, IssueCategory::InvalidDateFormat));
        
        assert_eq!(has_date_error, !should_be_valid);
    }
    
    #[test]
    fn test_validate_nonexistent_file() {
        let result = validate_document_compliance("nonexistent.md");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::FileNotFound { .. }));
    }
    
    // Helper functions for creating test documents
    fn create_compliant_test_document() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "---").unwrap();
        writeln!(temp_file, "title: \"Test Document\"").unwrap();
        writeln!(temp_file, "created_at: \"2025-08-25\"").unwrap();
        writeln!(temp_file, "updated_at: \"2025-08-25\"").unwrap();
        writeln!(temp_file, "tags: ['#test', '#guide']").unwrap();
        writeln!(temp_file, "description: \"A test document\"").unwrap();
        writeln!(temp_file, "---").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "# Test Document").unwrap();
        writeln!(temp_file, "Content here...").unwrap();
        temp_file
    }
}
```

### Phase 2: Integration Tests (Week 2)

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_api_contract_stability() {
        // Ensure the public API doesn't change accidentally
        let _: Result<DocumentationStandards, ValidationError> = get_documentation_standards();
        let _: Result<ComplianceResult, ValidationError> = validate_document_compliance("test.md");
    }
    
    #[test]
    fn test_error_handling_resilience() {
        // All error conditions should be gracefully handled
        let invalid_file_result = validate_document_compliance("does-not-exist.md");
        assert!(invalid_file_result.is_err());
        // Should not panic, should return proper error
    }
    
    #[test]
    fn test_performance_within_hook_constraints() {
        // Validate that operations complete within reasonable time
        use std::time::Instant;
        
        let start = Instant::now();
        let _standards = get_documentation_standards().unwrap();
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 100, "Standards loading too slow: {:?}", duration);
    }
}
```

## Implementation Phases

### Phase 1: Foundation (Days 1-2) ✅ COMPLETED
```bash
# ✅ Convert claude-hook-advisor to Cargo workspace
# ✅ Create claude-doc-advisor library crate as workspace member  
# ✅ Verify existing functionality works after workspace conversion
# ✅ Add placeholder lib.rs with minimal test

# Status: Workspace conversion successful, all 25 existing tests passing
# Branch: feature/claude-doc-advisor-workspace
```

### Phase 2: Validation Logic (Days 3-4)
```bash
# Implement YAML frontmatter parsing
# Build validation rules for each compliance check
# Implement validate_document_compliance()
# All tests should pass
```

### Phase 3: Integration Preparation (Day 5)
```bash
# Create comprehensive test fixtures
# Document integration points with claude-hook-advisor
# Performance testing and optimization
# Error handling refinement
```

### Phase 4: claude-hook-advisor Integration (Days 6-7)
```bash
# Add claude-doc-advisor dependency to claude-hook-advisor
# Enhance UserPromptSubmit hook with standards guidance
# Enhance PostToolUse hook with compliance validation
# End-to-end testing
```

## Next Session Checklist

~~Phase 1 Complete!~~ Ready for Phase 2:

1. **Implement core data structures** (DocumentationStandards, ComplianceResult, ValidationError)
2. **Write failing tests** for both API functions
3. **Implement `get_documentation_standards()`** with hardcoded standards from TEMPLATE.md
4. **Begin validation logic** for YAML frontmatter parsing
5. **Implement `validate_document_compliance()`** with comprehensive validation rules

## Success Criteria

- ✅ Two-function API that claude-hook-advisor can reliably call
- ✅ All failures are graceful and logged (no hook breaking)
- ✅ 95%+ test coverage with comprehensive edge case handling
- ✅ Performance suitable for hook context (<100ms operations)
- ✅ Integration with existing claude-hook-advisor.toml config
- ✅ Standards enforcement without file manipulation

---

*Created: 2025-08-25 | Updated: 2025-08-25*

#claude-doc-advisor #implementation #tdd #standards-enforcement #library-design