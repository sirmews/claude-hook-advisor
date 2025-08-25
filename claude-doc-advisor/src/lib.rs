//! Claude Doc Advisor - Documentation standards enforcement library
//!
//! This library provides documentation standards enforcement through two core functions:
//! - `get_documentation_standards()`: Retrieves current documentation standards
//! - `validate_document_compliance()`: Validates documents against those standards
//!
//! # Example Usage
//!
//! ```rust
//! use claude_doc_advisor::{get_documentation_standards, validate_document_compliance};
//!
//! // Get current documentation standards
//! let standards = get_documentation_standards().unwrap();
//! println!("Guidance: {}", standards.guidance_text);
//!
//! // Validate a document (will return error for non-existent file)
//! match validate_document_compliance("example.md") {
//!     Ok(result) => {
//!         if result.is_compliant {
//!             println!("Document meets all standards!");
//!         } else {
//!             println!("Issues found: {}", result.summary());
//!         }
//!     }
//!     Err(e) => println!("Could not validate document: {}", e),
//! }
//! ```

pub mod error;
pub mod parsing;
pub mod standards;
pub mod validator;

use error::ValidationResult;
use parsing::FrontmatterParser;
use std::path::Path;

// Re-export key types for convenience
pub use error::ValidationError;
pub use standards::{DocumentationStandards, TagFormatRules, FilenameRules};
pub use validator::{ComplianceResult, ComplianceIssue, IssueCategory, IssueSeverity};

/// Retrieves the current documentation standards
/// 
/// This function returns the documentation standards based on the TEMPLATE.md
/// requirements. It always succeeds and provides standards that can be used
/// for both guidance and validation.
/// 
/// # Returns
/// 
/// * `Ok(DocumentationStandards)` - The current documentation standards
/// * `Err(ValidationError)` - Should not occur in normal operation
/// 
/// # Example
/// 
/// ```rust
/// use claude_doc_advisor::get_documentation_standards;
/// 
/// let standards = get_documentation_standards().unwrap();
/// println!("Required fields: {:?}", standards.required_frontmatter_fields);
/// ```
pub fn get_documentation_standards() -> ValidationResult<DocumentationStandards> {
    Ok(DocumentationStandards::default_standards())
}

/// Validates a document's compliance with documentation standards
/// 
/// This function reads a markdown document, parses its YAML frontmatter,
/// and validates it against the current documentation standards. It performs
/// comprehensive checks including:
/// 
/// - Presence of required frontmatter fields
/// - Date format validation (YYYY-MM-DD)
/// - Tag format validation (#kebab-case)
/// - Filename convention checking
/// 
/// # Arguments
/// 
/// * `path` - Path to the markdown document to validate
/// 
/// # Returns
/// 
/// * `Ok(ComplianceResult)` - Validation results with issues and suggestions
/// * `Err(ValidationError)` - If the file cannot be read or parsed
/// 
/// # Example
/// 
/// ```rust
/// use claude_doc_advisor::validate_document_compliance;
/// 
/// match validate_document_compliance("my-doc.md") {
///     Ok(result) => {
///         if result.is_compliant {
///             println!("✓ Document is compliant");
///         } else {
///             for issue in &result.issues {
///                 println!("• {}", issue);
///             }
///         }
///     }
///     Err(e) => eprintln!("Validation error: {}", e),
/// }
/// ```
pub fn validate_document_compliance<P: AsRef<Path>>(path: P) -> ValidationResult<ComplianceResult> {
    let path = path.as_ref();
    let standards = get_documentation_standards()?;
    let parser = FrontmatterParser::new()?;
    
    // Parse the document
    let parsed = parser.parse_file(path)?;
    
    let mut issues = Vec::new();
    let mut suggestions = Vec::new();
    
    // Check for frontmatter presence
    if !parsed.has_frontmatter {
        issues.push(validator::ComplianceIssue::error(
            validator::IssueCategory::MissingFrontmatter,
            "Document is missing YAML frontmatter. Add frontmatter with required fields.".to_string()
        ));
        suggestions.push("Add YAML frontmatter at the top of the document with --- delimiters".to_string());
        
        // If no frontmatter, we can't check individual fields
        return Ok(validator::ComplianceResult::non_compliant(issues).with_suggestions(suggestions));
    }
    
    // Check required fields
    for required_field in &standards.required_frontmatter_fields {
        if !parsed.has_field(required_field) {
            issues.push(validator::ComplianceIssue::error(
                validator::IssueCategory::MissingRequiredField(required_field.clone()),
                format!("Required field '{required_field}' is missing from frontmatter")
            ));
        }
    }
    
    // Validate date fields
    for date_field in ["created_at", "updated_at"] {
        if parsed.has_field(date_field) {
            if let Err(date_error) = parsed.validate_date_field(date_field) {
                issues.push(validator::ComplianceIssue::error(
                    validator::IssueCategory::InvalidDateFormat,
                    format!("Field '{}' has invalid date format: {}", date_field, date_error.user_message())
                ));
            }
        }
    }
    
    // Validate tag format
    if let Some(tags) = parsed.get_string_array("tags") {
        for tag in &tags {
            if !tag.starts_with('#') {
                issues.push(validator::ComplianceIssue::warning(
                    validator::IssueCategory::InvalidTagFormat,
                    format!("Tag '{tag}' should start with '#' prefix")
                ));
            } else if tag.contains(' ') || tag.contains('_') {
                issues.push(validator::ComplianceIssue::warning(
                    validator::IssueCategory::InvalidTagFormat,
                    format!("Tag '{tag}' should use kebab-case format (avoid spaces and underscores)")
                ));
            }
        }
    }
    
    // Check filename conventions
    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        if !filename.ends_with(".md") {
            issues.push(validator::ComplianceIssue::warning(
                validator::IssueCategory::FilenameConvention,
                format!("Filename '{filename}' should have .md extension")
            ));
        }
        
        if filename.contains(' ') || filename.contains('_') {
            suggestions.push(format!("Consider renaming '{filename}' to use kebab-case format"));
        }
    }
    
    // Add general suggestions
    if parsed.has_field("title") && parsed.has_field("description") {
        if let (Some(title), Some(desc)) = (parsed.get_string("title"), parsed.get_string("description")) {
            if title == desc {
                suggestions.push("Consider making the description more detailed than the title".to_string());
            }
        }
    }
    
    // Determine compliance
    let is_compliant = issues.iter().all(|issue| issue.severity != validator::IssueSeverity::Error);
    
    let mut result = if is_compliant && !issues.is_empty() {
        // Has warnings but no errors - still compliant but with issues
        let mut compliant_result = validator::ComplianceResult::compliant();
        compliant_result.issues = issues;
        compliant_result
    } else if is_compliant {
        validator::ComplianceResult::compliant()
    } else {
        validator::ComplianceResult::non_compliant(issues)
    };
    
    if !suggestions.is_empty() {
        result = result.with_suggestions(suggestions);
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_get_documentation_standards() {
        let standards = get_documentation_standards().unwrap();
        
        assert!(!standards.required_frontmatter_fields.is_empty());
        assert_eq!(standards.date_format, "YYYY-MM-DD");
        assert!(!standards.guidance_text.is_empty());
        assert!(standards.guidance_text.contains("YAML frontmatter"));
    }

    #[test]
    fn test_validate_compliant_document() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "---").unwrap();
        writeln!(temp_file, "title: \"Test Document\"").unwrap();
        writeln!(temp_file, "created_at: \"2025-08-25\"").unwrap();
        writeln!(temp_file, "updated_at: \"2025-08-25\"").unwrap();
        writeln!(temp_file, "tags: ['#test', '#guide']").unwrap();
        writeln!(temp_file, "description: \"A comprehensive test document\"").unwrap();
        writeln!(temp_file, "---").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "# Test Document").unwrap();
        writeln!(temp_file, "Content here...").unwrap();

        let result = validate_document_compliance(temp_file.path()).unwrap();
        
        assert!(result.is_compliant);
        assert_eq!(result.errors().len(), 0);
    }

    #[test]
    fn test_validate_document_missing_frontmatter() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# Test Document").unwrap();
        writeln!(temp_file, "This document has no frontmatter.").unwrap();

        let result = validate_document_compliance(temp_file.path()).unwrap();
        
        assert!(!result.is_compliant);
        assert_eq!(result.errors().len(), 1);
        assert!(result.issues.iter().any(|i| matches!(i.category, validator::IssueCategory::MissingFrontmatter)));
    }

    #[test]
    fn test_validate_document_missing_required_fields() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "---").unwrap();
        writeln!(temp_file, "title: \"Test Document\"").unwrap();
        // Missing created_at, updated_at, tags, description
        writeln!(temp_file, "---").unwrap();
        writeln!(temp_file, "Content").unwrap();

        let result = validate_document_compliance(temp_file.path()).unwrap();
        
        assert!(!result.is_compliant);
        assert!(result.errors().len() >= 3); // At least missing created_at, updated_at, tags, description
    }

    #[test]
    fn test_validate_document_invalid_date_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "---").unwrap();
        writeln!(temp_file, "title: \"Test Document\"").unwrap();
        writeln!(temp_file, "created_at: \"08-25-2025\"").unwrap(); // Wrong format
        writeln!(temp_file, "updated_at: \"2025-08-25\"").unwrap();
        writeln!(temp_file, "tags: ['#test']").unwrap();
        writeln!(temp_file, "description: \"Test\"").unwrap();
        writeln!(temp_file, "---").unwrap();
        writeln!(temp_file, "Content").unwrap();

        let result = validate_document_compliance(temp_file.path()).unwrap();
        
        assert!(!result.is_compliant);
        assert!(result.issues.iter().any(|i| matches!(i.category, validator::IssueCategory::InvalidDateFormat)));
    }

    #[test]
    fn test_validate_document_invalid_tags() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "---").unwrap();
        writeln!(temp_file, "title: \"Test Document\"").unwrap();
        writeln!(temp_file, "created_at: \"2025-08-25\"").unwrap();
        writeln!(temp_file, "updated_at: \"2025-08-25\"").unwrap();
        writeln!(temp_file, "tags: ['test', '#good-tag', '#bad_tag']").unwrap(); // Mixed formats
        writeln!(temp_file, "description: \"Test\"").unwrap();
        writeln!(temp_file, "---").unwrap();
        writeln!(temp_file, "Content").unwrap();

        let result = validate_document_compliance(temp_file.path()).unwrap();
        
        // Should have warnings for tag format issues
        assert!(result.warnings().len() >= 2); // 'test' missing #, 'bad_tag' has underscore
        assert!(result.issues.iter().any(|i| matches!(i.category, validator::IssueCategory::InvalidTagFormat)));
    }

    #[test]
    fn test_validate_nonexistent_file() {
        let result = validate_document_compliance("nonexistent.md");
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::FileNotFound { .. }));
    }

    #[test]
    fn test_api_contract_stability() {
        // Ensure the public API doesn't change accidentally
        let _: ValidationResult<DocumentationStandards> = get_documentation_standards();
        let _: ValidationResult<ComplianceResult> = validate_document_compliance("test.md");
    }
}