//! YAML frontmatter parsing utilities

use crate::error::{ValidationError, ValidationResult};
use regex::Regex;
use serde_yaml::Value;
use std::collections::HashMap;
use std::path::Path;

/// Parsed frontmatter with content separation
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    /// YAML frontmatter as key-value pairs
    pub frontmatter: HashMap<String, Value>,
    /// Document content after frontmatter
    pub content: String,
    /// Whether frontmatter was present
    pub has_frontmatter: bool,
}

/// YAML frontmatter parser
pub struct FrontmatterParser {
    /// Regex for detecting frontmatter boundaries
    delimiter_regex: Regex,
}

impl FrontmatterParser {
    /// Creates a new frontmatter parser
    pub fn new() -> ValidationResult<Self> {
        let delimiter_regex = Regex::new(r"^---\s*$")?;
        Ok(Self {
            delimiter_regex,
        })
    }

    /// Parses a document file and extracts frontmatter
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> ValidationResult<ParsedDocument> {
        let path = path.as_ref();
        
        // Read file content
        let content = std::fs::read_to_string(path)
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => ValidationError::file_not_found(path),
                _ => ValidationError::io_error(path, e),
            })?;

        self.parse_content(&content, Some(path))
    }

    /// Parses document content and extracts frontmatter
    pub fn parse_content(&self, content: &str, path: Option<&Path>) -> ValidationResult<ParsedDocument> {
        // Check if content starts with frontmatter delimiter
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() || !self.delimiter_regex.is_match(lines[0]) {
            // No frontmatter present
            return Ok(ParsedDocument {
                frontmatter: HashMap::new(),
                content: content.to_string(),
                has_frontmatter: false,
            });
        }

        // Find the closing delimiter
        let mut closing_delimiter_line = None;
        for (i, line) in lines.iter().enumerate().skip(1) {
            if self.delimiter_regex.is_match(line) {
                closing_delimiter_line = Some(i);
                break;
            }
        }

        let closing_line = closing_delimiter_line.ok_or_else(|| {
            if let Some(p) = path {
                ValidationError::malformed_delimiters(p)
            } else {
                ValidationError::invalid_frontmatter("Missing closing frontmatter delimiter (---)")
            }
        })?;

        // Extract frontmatter content
        let frontmatter_lines = &lines[1..closing_line];
        let frontmatter_content = frontmatter_lines.join("\n");

        // Parse YAML frontmatter
        let frontmatter_value: Value = serde_yaml::from_str(&frontmatter_content)
            .map_err(|e| ValidationError::YamlParse { source: e })?;

        // Convert to HashMap for easier access
        let frontmatter = match frontmatter_value {
            Value::Mapping(map) => {
                let mut result = HashMap::new();
                for (key, value) in map {
                    if let Value::String(key_str) = key {
                        result.insert(key_str, value);
                    }
                }
                result
            }
            _ => {
                return Err(ValidationError::invalid_frontmatter(
                    "Frontmatter must be a YAML mapping (key-value pairs)"
                ));
            }
        };

        // Extract document content (everything after closing delimiter)
        let content_lines = if closing_line + 1 < lines.len() {
            &lines[closing_line + 1..]
        } else {
            &[]
        };
        let document_content = content_lines.join("\n");

        Ok(ParsedDocument {
            frontmatter,
            content: document_content,
            has_frontmatter: true,
        })
    }
}

impl Default for FrontmatterParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default frontmatter parser")
    }
}

impl ParsedDocument {
    /// Gets a frontmatter field as a string
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.frontmatter.get(key).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            _ => None,
        })
    }

    /// Gets a frontmatter field as a vector of strings
    pub fn get_string_array(&self, key: &str) -> Option<Vec<String>> {
        self.frontmatter.get(key).and_then(|v| match v {
            Value::Sequence(seq) => {
                let mut result = Vec::new();
                for item in seq {
                    match item {
                        Value::String(s) => result.push(s.clone()),
                        _ => return None, // Mixed types not supported
                    }
                }
                Some(result)
            }
            _ => None,
        })
    }

    /// Checks if a required field exists
    pub fn has_field(&self, key: &str) -> bool {
        self.frontmatter.contains_key(key)
    }

    /// Gets all frontmatter field names
    pub fn field_names(&self) -> Vec<String> {
        self.frontmatter.keys().cloned().collect()
    }

    /// Validates a date field format (YYYY-MM-DD)
    pub fn validate_date_field(&self, field_name: &str) -> ValidationResult<()> {
        if let Some(date_str) = self.get_string(field_name) {
            if !is_valid_date_format(&date_str) {
                return Err(ValidationError::invalid_date_format(field_name, &date_str));
            }
        }
        Ok(())
    }
}

/// Validates if a string matches YYYY-MM-DD format
fn is_valid_date_format(date_str: &str) -> bool {
    // Simple regex for YYYY-MM-DD format
    let date_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$")
        .expect("Date regex should be valid");
    
    if !date_regex.is_match(date_str) {
        return false;
    }

    // Additional validation: try to parse the date
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_document_with_frontmatter() {
        let content = r#"---
title: "Test Document"
created_at: "2025-08-25"
updated_at: "2025-08-25"
tags: ['#test', '#guide']
description: "A test document"
---

# Test Document

This is the document content.
"#;

        let parser = FrontmatterParser::new().unwrap();
        let parsed = parser.parse_content(content, None).unwrap();

        assert!(parsed.has_frontmatter);
        assert_eq!(parsed.get_string("title"), Some("Test Document".to_string()));
        assert_eq!(parsed.get_string("created_at"), Some("2025-08-25".to_string()));
        assert!(parsed.content.contains("# Test Document"));
    }

    #[test]
    fn test_parse_document_without_frontmatter() {
        let content = r#"# Test Document

This is a document without frontmatter.
"#;

        let parser = FrontmatterParser::new().unwrap();
        let parsed = parser.parse_content(content, None).unwrap();

        assert!(!parsed.has_frontmatter);
        assert!(parsed.frontmatter.is_empty());
        assert_eq!(parsed.content, content);
    }

    #[test]
    fn test_parse_document_malformed_frontmatter() {
        let content = r#"---
title: "Test Document"
created_at: "2025-08-25"
# Missing closing delimiter

# Test Document
"#;

        let parser = FrontmatterParser::new().unwrap();
        let result = parser.parse_content(content, None);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::InvalidFrontmatter { .. }));
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let content = r#"---
title: "Test Document"
invalid: yaml: content: here
---

# Test Document
"#;

        let parser = FrontmatterParser::new().unwrap();
        let result = parser.parse_content(content, None);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::YamlParse { .. }));
    }

    #[test]
    fn test_parse_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "---").unwrap();
        writeln!(temp_file, "title: \"Test File\"").unwrap();
        writeln!(temp_file, "created_at: \"2025-08-25\"").unwrap();
        writeln!(temp_file, "---").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "# Test Content").unwrap();

        let parser = FrontmatterParser::new().unwrap();
        let parsed = parser.parse_file(temp_file.path()).unwrap();

        assert!(parsed.has_frontmatter);
        assert_eq!(parsed.get_string("title"), Some("Test File".to_string()));
        assert!(parsed.content.contains("# Test Content"));
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let parser = FrontmatterParser::new().unwrap();
        let result = parser.parse_file("nonexistent.md");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::FileNotFound { .. }));
    }

    #[test]
    fn test_get_string_array() {
        let content = r#"---
tags: ['#tag1', '#tag2', '#tag3']
numbers: [1, 2, 3]
mixed: ['string', 123]
---

Content here
"#;

        let parser = FrontmatterParser::new().unwrap();
        let parsed = parser.parse_content(content, None).unwrap();

        // String array should work
        let tags = parsed.get_string_array("tags");
        assert!(tags.is_some());
        let tags = tags.unwrap();
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&"#tag1".to_string()));

        // Numbers array should not work (type mismatch)
        assert!(parsed.get_string_array("numbers").is_none());
        
        // Mixed array should not work
        assert!(parsed.get_string_array("mixed").is_none());
    }

    #[test]
    fn test_date_validation() {
        let content = r#"---
valid_date: "2025-08-25"
invalid_format: "08-25-2025"
invalid_date: "2025-02-30"
---

Content
"#;

        let parser = FrontmatterParser::new().unwrap();
        let parsed = parser.parse_content(content, None).unwrap();

        // Valid date should pass
        assert!(parsed.validate_date_field("valid_date").is_ok());

        // Invalid format should fail
        assert!(parsed.validate_date_field("invalid_format").is_err());

        // Invalid date should fail
        assert!(parsed.validate_date_field("invalid_date").is_err());

        // Non-existent field should pass (no validation needed)
        assert!(parsed.validate_date_field("nonexistent").is_ok());
    }

    #[test]
    fn test_field_operations() {
        let content = r#"---
title: "Test"
count: 42
active: true
---

Content
"#;

        let parser = FrontmatterParser::new().unwrap();
        let parsed = parser.parse_content(content, None).unwrap();

        // Test field existence
        assert!(parsed.has_field("title"));
        assert!(parsed.has_field("count"));
        assert!(parsed.has_field("active"));
        assert!(!parsed.has_field("missing"));

        // Test field names
        let field_names = parsed.field_names();
        assert_eq!(field_names.len(), 3);
        assert!(field_names.contains(&"title".to_string()));

        // Test different value types as strings
        assert_eq!(parsed.get_string("title"), Some("Test".to_string()));
        assert_eq!(parsed.get_string("count"), Some("42".to_string()));
        assert_eq!(parsed.get_string("active"), Some("true".to_string()));
    }

    #[test]
    fn test_is_valid_date_format() {
        assert!(is_valid_date_format("2025-08-25"));
        assert!(is_valid_date_format("2025-01-01"));
        assert!(is_valid_date_format("2025-12-31"));
        
        assert!(!is_valid_date_format("08-25-2025"));
        assert!(!is_valid_date_format("2025/08/25"));
        assert!(!is_valid_date_format("2025-8-25"));
        assert!(!is_valid_date_format("2025-02-30")); // Invalid date
        assert!(!is_valid_date_format("not-a-date"));
    }
}