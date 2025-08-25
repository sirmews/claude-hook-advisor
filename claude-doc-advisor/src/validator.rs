//! Document compliance validation logic and result types

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result of document compliance validation
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceResult {
    /// Whether the document meets all standards
    pub is_compliant: bool,
    /// List of issues found during validation
    pub issues: Vec<ComplianceIssue>,
    /// Suggestions for improving compliance
    pub suggestions: Vec<String>,
}

/// Individual compliance issue found in a document
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceIssue {
    /// Severity level of the issue
    pub severity: IssueSeverity,
    /// Category/type of the issue
    pub category: IssueCategory,
    /// Human-readable description of the issue
    pub message: String,
    /// Line number where the issue was found (if applicable)
    pub line_number: Option<usize>,
}

/// Severity level for compliance issues
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Must fix - prevents compliance
    Error,
    /// Should fix - affects consistency but doesn't prevent compliance
    Warning,
}

/// Categories of compliance issues
#[derive(Debug, Serialize, Deserialize)]
pub enum IssueCategory {
    /// YAML frontmatter is missing entirely
    MissingFrontmatter,
    /// A required field is missing from frontmatter
    MissingRequiredField(String),
    /// Date format doesn't match expected pattern
    InvalidDateFormat,
    /// Tag format doesn't follow conventions
    InvalidTagFormat,
    /// Filename doesn't follow naming conventions
    FilenameConvention,
    /// Document structure issues (missing sections, etc.)
    StructureIssue,
}

impl ComplianceResult {
    /// Creates a new compliant result with no issues
    pub fn compliant() -> Self {
        Self {
            is_compliant: true,
            issues: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Creates a new non-compliant result with issues
    pub fn non_compliant(issues: Vec<ComplianceIssue>) -> Self {
        Self {
            is_compliant: false,
            issues,
            suggestions: Vec::new(),
        }
    }

    /// Adds a suggestion to the result
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Adds multiple suggestions to the result
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    /// Generates a summary of the compliance result
    pub fn summary(&self) -> String {
        if self.is_compliant {
            "Document meets all standards".to_string()
        } else {
            let error_count = self.issues.iter()
                .filter(|i| i.severity == IssueSeverity::Error)
                .count();
            let warning_count = self.issues.iter()
                .filter(|i| i.severity == IssueSeverity::Warning)
                .count();
                
            match (error_count, warning_count) {
                (0, 0) => "No issues found".to_string(),
                (e, 0) => format!("{} error{}", e, if e == 1 { "" } else { "s" }),
                (0, w) => format!("{} warning{}", w, if w == 1 { "" } else { "s" }),
                (e, w) => format!("{} error{}, {} warning{}", 
                    e, if e == 1 { "" } else { "s" },
                    w, if w == 1 { "" } else { "s" }),
            }
        }
    }

    /// Returns all error-level issues
    pub fn errors(&self) -> Vec<&ComplianceIssue> {
        self.issues.iter()
            .filter(|issue| issue.severity == IssueSeverity::Error)
            .collect()
    }

    /// Returns all warning-level issues
    pub fn warnings(&self) -> Vec<&ComplianceIssue> {
        self.issues.iter()
            .filter(|issue| issue.severity == IssueSeverity::Warning)
            .collect()
    }
}

impl ComplianceIssue {
    /// Creates a new error-level compliance issue
    pub fn error(category: IssueCategory, message: String) -> Self {
        Self {
            severity: IssueSeverity::Error,
            category,
            message,
            line_number: None,
        }
    }

    /// Creates a new warning-level compliance issue
    pub fn warning(category: IssueCategory, message: String) -> Self {
        Self {
            severity: IssueSeverity::Warning,
            category,
            message,
            line_number: None,
        }
    }

    /// Sets the line number for this issue
    pub fn at_line(mut self, line_number: usize) -> Self {
        self.line_number = Some(line_number);
        self
    }
}

impl fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueSeverity::Error => write!(f, "error"),
            IssueSeverity::Warning => write!(f, "warning"),
        }
    }
}

impl fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueCategory::MissingFrontmatter => write!(f, "missing frontmatter"),
            IssueCategory::MissingRequiredField(field) => write!(f, "missing required field: {field}"),
            IssueCategory::InvalidDateFormat => write!(f, "invalid date format"),
            IssueCategory::InvalidTagFormat => write!(f, "invalid tag format"),
            IssueCategory::FilenameConvention => write!(f, "filename convention"),
            IssueCategory::StructureIssue => write!(f, "structure issue"),
        }
    }
}

impl fmt::Display for ComplianceIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line_number {
            write!(f, "line {}: {} ({}): {}", line, self.severity, self.category, self.message)
        } else {
            write!(f, "{} ({}): {}", self.severity, self.category, self.message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliant_result() {
        let result = ComplianceResult::compliant();
        
        assert!(result.is_compliant);
        assert!(result.issues.is_empty());
        assert!(result.suggestions.is_empty());
        assert_eq!(result.summary(), "Document meets all standards");
    }

    #[test]
    fn test_non_compliant_result() {
        let issues = vec![
            ComplianceIssue::error(
                IssueCategory::MissingRequiredField("title".to_string()),
                "Title field is required in frontmatter".to_string()
            ),
            ComplianceIssue::warning(
                IssueCategory::InvalidTagFormat,
                "Tags should use kebab-case format".to_string()
            ),
        ];

        let result = ComplianceResult::non_compliant(issues);
        
        assert!(!result.is_compliant);
        assert_eq!(result.issues.len(), 2);
        assert_eq!(result.summary(), "1 error, 1 warning");
    }

    #[test]
    fn test_compliance_issue_creation() {
        let error = ComplianceIssue::error(
            IssueCategory::MissingFrontmatter,
            "Document is missing YAML frontmatter".to_string()
        );
        
        assert_eq!(error.severity, IssueSeverity::Error);
        assert!(matches!(error.category, IssueCategory::MissingFrontmatter));
        assert!(error.message.contains("YAML frontmatter"));
        assert!(error.line_number.is_none());
    }

    #[test]
    fn test_compliance_issue_with_line_number() {
        let issue = ComplianceIssue::warning(
            IssueCategory::InvalidDateFormat,
            "Date should be in YYYY-MM-DD format".to_string()
        ).at_line(5);
        
        assert_eq!(issue.line_number, Some(5));
        assert!(issue.to_string().contains("line 5"));
    }

    #[test]
    fn test_result_with_suggestions() {
        let result = ComplianceResult::compliant()
            .with_suggestion("Consider adding more descriptive tags".to_string())
            .with_suggestions(vec![
                "Use kebab-case for filenames".to_string(),
                "Add Purpose section".to_string(),
            ]);
        
        assert_eq!(result.suggestions.len(), 3);
    }

    #[test]
    fn test_errors_and_warnings_filtering() {
        let issues = vec![
            ComplianceIssue::error(
                IssueCategory::MissingRequiredField("title".to_string()),
                "Missing title".to_string()
            ),
            ComplianceIssue::warning(
                IssueCategory::InvalidTagFormat,
                "Tag format issue".to_string()
            ),
            ComplianceIssue::error(
                IssueCategory::MissingFrontmatter,
                "Missing frontmatter".to_string()
            ),
        ];

        let result = ComplianceResult::non_compliant(issues);
        
        assert_eq!(result.errors().len(), 2);
        assert_eq!(result.warnings().len(), 1);
    }

    #[test]
    fn test_summary_formatting() {
        // Test different combinations of errors and warnings
        let test_cases = vec![
            (vec![], "Document meets all standards"),
            (vec![ComplianceIssue::error(IssueCategory::MissingFrontmatter, "test".to_string())], "1 error"),
            (vec![ComplianceIssue::warning(IssueCategory::InvalidTagFormat, "test".to_string())], "1 warning"),
        ];

        for (issues, expected) in test_cases {
            let result = if issues.is_empty() {
                ComplianceResult::compliant()
            } else {
                ComplianceResult::non_compliant(issues)
            };
            
            assert_eq!(result.summary(), expected);
        }
    }
}