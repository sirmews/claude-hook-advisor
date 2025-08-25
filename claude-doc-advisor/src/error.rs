//! Error types for documentation validation

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during documentation validation
#[derive(Debug, Error)]
pub enum ValidationError {
    /// File not found at the specified path
    #[error("File not found: {path}")]
    FileNotFound {
        path: PathBuf,
    },
    
    /// IO error occurred while reading file
    #[error("IO error reading file '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    /// YAML frontmatter is invalid or malformed
    #[error("Invalid YAML frontmatter: {reason}")]
    InvalidFrontmatter {
        reason: String,
    },
    
    /// Error parsing YAML content
    #[error("YAML parsing error: {source}")]
    YamlParse {
        #[from]
        source: serde_yaml::Error,
    },
    
    /// Regular expression compilation failed
    #[error("Regex compilation error: {source}")]
    Regex {
        #[from]
        source: regex::Error,
    },
    
    /// Document encoding is not valid UTF-8
    #[error("Invalid UTF-8 encoding in file: {path}")]
    InvalidEncoding {
        path: PathBuf,
    },
    
    /// Frontmatter delimiters are missing or malformed
    #[error("Missing or malformed frontmatter delimiters (---) in file: {path}")]
    MalformedDelimiters {
        path: PathBuf,
    },
    
    /// Date parsing error for created_at/updated_at fields
    #[error("Invalid date format in field '{field}': expected YYYY-MM-DD, got '{value}'")]
    InvalidDateFormat {
        field: String,
        value: String,
    },
    
    /// Required field is missing from frontmatter
    #[error("Required field '{field}' is missing from frontmatter")]
    MissingRequiredField {
        field: String,
    },
    
    /// Generic validation error for custom rules
    #[error("Validation failed: {message}")]
    ValidationFailed {
        message: String,
    },
}

impl ValidationError {
    /// Creates a file not found error
    pub fn file_not_found<P: Into<PathBuf>>(path: P) -> Self {
        Self::FileNotFound {
            path: path.into(),
        }
    }
    
    /// Creates an IO error with context about the file path
    pub fn io_error<P: Into<PathBuf>>(path: P, error: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source: error,
        }
    }
    
    /// Creates an invalid frontmatter error
    pub fn invalid_frontmatter<S: Into<String>>(reason: S) -> Self {
        Self::InvalidFrontmatter {
            reason: reason.into(),
        }
    }
    
    /// Creates an invalid encoding error
    pub fn invalid_encoding<P: Into<PathBuf>>(path: P) -> Self {
        Self::InvalidEncoding {
            path: path.into(),
        }
    }
    
    /// Creates a malformed delimiters error
    pub fn malformed_delimiters<P: Into<PathBuf>>(path: P) -> Self {
        Self::MalformedDelimiters {
            path: path.into(),
        }
    }
    
    /// Creates an invalid date format error
    pub fn invalid_date_format<S: Into<String>>(field: S, value: S) -> Self {
        Self::InvalidDateFormat {
            field: field.into(),
            value: value.into(),
        }
    }
    
    /// Creates a missing required field error
    pub fn missing_required_field<S: Into<String>>(field: S) -> Self {
        Self::MissingRequiredField {
            field: field.into(),
        }
    }
    
    /// Creates a generic validation error
    pub fn validation_failed<S: Into<String>>(message: S) -> Self {
        Self::ValidationFailed {
            message: message.into(),
        }
    }
    
    /// Returns whether this error is recoverable (warnings vs errors)
    pub fn is_recoverable(&self) -> bool {
        match self {
            // File system errors are not recoverable
            Self::FileNotFound { .. } | Self::Io { .. } | Self::InvalidEncoding { .. } => false,
            // Parse errors are generally not recoverable
            Self::YamlParse { .. } | Self::Regex { .. } | Self::MalformedDelimiters { .. } => false,
            // Content validation errors might be recoverable depending on context
            Self::InvalidFrontmatter { .. } 
            | Self::InvalidDateFormat { .. }
            | Self::MissingRequiredField { .. }
            | Self::ValidationFailed { .. } => true,
        }
    }
    
    /// Returns a user-friendly error message suitable for display
    pub fn user_message(&self) -> String {
        match self {
            Self::FileNotFound { path } => {
                format!("Could not find the file: {}", path.display())
            }
            Self::Io { path, .. } => {
                format!("Could not read the file: {}", path.display())
            }
            Self::InvalidFrontmatter { reason } => {
                format!("The document's YAML frontmatter is invalid: {reason}")
            }
            Self::YamlParse { source } => {
                format!("Could not parse YAML frontmatter: {source}")
            }
            Self::Regex { .. } => {
                "Internal error: invalid pattern matching".to_string()
            }
            Self::InvalidEncoding { path } => {
                format!("The file contains invalid characters: {}", path.display())
            }
            Self::MalformedDelimiters { path } => {
                format!("The file is missing proper frontmatter delimiters (---): {}", path.display())
            }
            Self::InvalidDateFormat { field, value } => {
                format!("The {field} field has an invalid date format: '{value}'. Expected format: YYYY-MM-DD")
            }
            Self::MissingRequiredField { field } => {
                format!("The required field '{field}' is missing from the document frontmatter")
            }
            Self::ValidationFailed { message } => {
                message.clone()
            }
        }
    }
}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::path::Path;

    #[test]
    fn test_file_not_found_error() {
        let path = Path::new("/nonexistent/file.md");
        let error = ValidationError::file_not_found(path);
        
        match &error {
            ValidationError::FileNotFound { path: error_path } => {
                assert_eq!(error_path, path);
            }
            _ => panic!("Expected FileNotFound error"),
        }
        
        assert!(!error.is_recoverable());
        assert!(error.user_message().contains("Could not find"));
    }

    #[test]
    fn test_io_error() {
        let path = Path::new("/test/file.md");
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let error = ValidationError::io_error(path, io_error);
        
        match &error {
            ValidationError::Io { path: error_path, .. } => {
                assert_eq!(error_path, path);
            }
            _ => panic!("Expected Io error"),
        }
        
        assert!(!error.is_recoverable());
        assert!(error.user_message().contains("Could not read"));
    }

    #[test]
    fn test_invalid_frontmatter_error() {
        let reason = "Missing closing delimiter";
        let error = ValidationError::invalid_frontmatter(reason);
        
        match &error {
            ValidationError::InvalidFrontmatter { reason: error_reason } => {
                assert_eq!(error_reason, reason);
            }
            _ => panic!("Expected InvalidFrontmatter error"),
        }
        
        assert!(error.is_recoverable());
        assert!(error.user_message().contains("YAML frontmatter is invalid"));
    }

    #[test]
    fn test_invalid_date_format_error() {
        let field = "created_at";
        let value = "2025/01/15";
        let error = ValidationError::invalid_date_format(field, value);
        
        match &error {
            ValidationError::InvalidDateFormat { field: error_field, value: error_value } => {
                assert_eq!(error_field, field);
                assert_eq!(error_value, value);
            }
            _ => panic!("Expected InvalidDateFormat error"),
        }
        
        assert!(error.is_recoverable());
        assert!(error.user_message().contains("YYYY-MM-DD"));
    }

    #[test]
    fn test_missing_required_field_error() {
        let field = "title";
        let error = ValidationError::missing_required_field(field);
        
        match &error {
            ValidationError::MissingRequiredField { field: error_field } => {
                assert_eq!(error_field, field);
            }
            _ => panic!("Expected MissingRequiredField error"),
        }
        
        assert!(error.is_recoverable());
        assert!(error.user_message().contains("required field"));
    }

    #[test]
    fn test_error_display() {
        let error = ValidationError::file_not_found("/test/file.md");
        let display_string = format!("{}", error);
        assert!(display_string.contains("File not found"));
        assert!(display_string.contains("/test/file.md"));
    }

    #[test]
    fn test_error_source_chain() {
        let yaml_error = serde_yaml::Error::from(serde_yaml::from_str::<serde_yaml::Value>("invalid: yaml: content").unwrap_err());
        let validation_error = ValidationError::YamlParse { source: yaml_error };
        
        // Test that the source chain works
        assert!(validation_error.source().is_some());
    }

    #[test]
    fn test_validation_failed_error() {
        let message = "Custom validation rule failed";
        let error = ValidationError::validation_failed(message);
        
        assert!(error.is_recoverable());
        assert_eq!(error.user_message(), message);
    }
}