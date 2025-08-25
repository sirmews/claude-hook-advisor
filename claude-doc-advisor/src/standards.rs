//! Documentation standards definitions and rules

use serde::{Deserialize, Serialize};

/// Documentation standards configuration based on TEMPLATE.md requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationStandards {
    /// Required fields that must be present in YAML frontmatter
    pub required_frontmatter_fields: Vec<String>,
    /// Expected date format for created_at and updated_at fields
    pub date_format: String,
    /// Rules for tag formatting and validation
    pub tag_format_rules: TagFormatRules,
    /// Filename conventions and validation rules
    pub filename_conventions: FilenameRules,
    /// Human-readable guidance text to display to users
    pub guidance_text: String,
}

/// Rules for validating tag format in documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagFormatRules {
    /// Whether tags must start with '#' prefix
    pub require_hash_prefix: bool,
    /// Whether tags should use kebab-case formatting
    pub prefer_kebab_case: bool,
    /// List of recommended tag categories
    pub recommended_categories: Vec<String>,
}

/// Rules for validating filename conventions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilenameRules {
    /// Preferred case style for filenames (kebab-case, snake_case, etc.)
    pub case_style: String,
    /// Required file extension
    pub required_extension: String,
    /// Whether descriptive names are preferred over generic ones
    pub prefer_descriptive_names: bool,
}

impl DocumentationStandards {
    /// Creates default documentation standards based on TEMPLATE.md
    pub fn default_standards() -> Self {
        Self {
            required_frontmatter_fields: vec![
                "title".to_string(),
                "created_at".to_string(),
                "updated_at".to_string(),
                "tags".to_string(),
                "description".to_string(),
            ],
            date_format: "YYYY-MM-DD".to_string(),
            tag_format_rules: TagFormatRules::default_rules(),
            filename_conventions: FilenameRules::kebab_case(),
            guidance_text: Self::generate_guidance_text(),
        }
    }
    
    /// Generates human-readable guidance text for users
    fn generate_guidance_text() -> String {
        let current_date = chrono::Utc::now().format("%Y-%m-%d");
        format!(
            "Documentation Standards:\n\
            • Required YAML frontmatter: title, created_at, updated_at, tags, description\n\
            • Date format: YYYY-MM-DD (example: {current_date})\n\
            • Tags must start with # and use kebab-case (#project-name, #guide)\n\
            • Filenames should use kebab-case.md\n\
            • Include Purpose and Content Structure sections\n\
            • Keep documents focused and concise"
        )
    }
}

impl TagFormatRules {
    /// Creates default tag format rules
    pub fn default_rules() -> Self {
        Self {
            require_hash_prefix: true,
            prefer_kebab_case: true,
            recommended_categories: vec![
                // Technology tags
                "react".to_string(),
                "python".to_string(),
                "claude-code".to_string(),
                // Document type tags
                "reference".to_string(),
                "guide".to_string(),
                "tutorial".to_string(),
                "troubleshooting".to_string(),
                // Topic area tags
                "security".to_string(),
                "api".to_string(),
                "deployment".to_string(),
                "configuration".to_string(),
            ],
        }
    }
}

impl FilenameRules {
    /// Creates filename rules for kebab-case convention
    pub fn kebab_case() -> Self {
        Self {
            case_style: "kebab-case".to_string(),
            required_extension: ".md".to_string(),
            prefer_descriptive_names: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_standards_creation() {
        let standards = DocumentationStandards::default_standards();
        
        // Check required fields are present
        assert!(standards.required_frontmatter_fields.contains(&"title".to_string()));
        assert!(standards.required_frontmatter_fields.contains(&"created_at".to_string()));
        assert!(standards.required_frontmatter_fields.contains(&"updated_at".to_string()));
        assert!(standards.required_frontmatter_fields.contains(&"tags".to_string()));
        assert!(standards.required_frontmatter_fields.contains(&"description".to_string()));
        
        // Check date format
        assert_eq!(standards.date_format, "YYYY-MM-DD");
        
        // Check guidance text contains key elements
        assert!(standards.guidance_text.contains("YAML frontmatter"));
        assert!(standards.guidance_text.contains("YYYY-MM-DD"));
        assert!(standards.guidance_text.contains("kebab-case"));
    }

    #[test]
    fn test_tag_format_rules() {
        let rules = TagFormatRules::default_rules();
        
        assert!(rules.require_hash_prefix);
        assert!(rules.prefer_kebab_case);
        assert!(!rules.recommended_categories.is_empty());
        assert!(rules.recommended_categories.contains(&"guide".to_string()));
    }

    #[test]
    fn test_filename_rules() {
        let rules = FilenameRules::kebab_case();
        
        assert_eq!(rules.case_style, "kebab-case");
        assert_eq!(rules.required_extension, ".md");
        assert!(rules.prefer_descriptive_names);
    }

    #[test]
    fn test_guidance_text_includes_current_date() {
        let standards = DocumentationStandards::default_standards();
        let current_year = chrono::Utc::now().format("%Y").to_string();
        
        // Guidance text should include current year as example
        assert!(standards.guidance_text.contains(&current_year));
    }
}