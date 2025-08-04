//! Command pattern matching and project type detection

use crate::types::Config;
use std::collections::HashMap;

/// Creates a properly formatted TOML file with project-specific command mappings
/// and common safety alternatives. Uses the toml crate for proper serialization.
/// 
/// # Arguments
/// * `project_type` - The type of project ("Node.js", "Python", "Rust", etc.)
/// 
/// # Returns
/// * `String` - Complete TOML configuration content with header and mappings
pub fn generate_config_for_project(project_type: &str) -> String {
    let commands = get_commands_for_project_type(project_type);
    let config = Config { commands };
    
    let header = format!(
        "# Claude Hook Advisor Configuration\n# Auto-generated for {project_type} project\n\n"
    );
    
    match toml::to_string_pretty(&config) {
        Ok(toml_content) => format!("{header}{toml_content}"),
        Err(_) => {
            // Fallback to basic config if serialization fails
            format!("{header}[commands]\n# Basic configuration\n")
        }
    }
}

/// Creates command mappings based on project type.
/// 
/// Returns a HashMap of command mappings tailored to the specific project type.
/// Includes both project-specific tools and common safety/modern alternatives.
/// 
/// # Arguments
/// * `project_type` - The type of project ("Node.js", "Python", "Rust", etc.)
/// 
/// # Returns
/// * `HashMap<String, String>` - Map from original commands to replacement commands
pub fn get_commands_for_project_type(project_type: &str) -> HashMap<String, String> {
    let mut commands = HashMap::new();
    
    match project_type {
        "Node.js" => {
            commands.insert("npm".to_string(), "bun".to_string());
            commands.insert("yarn".to_string(), "bun".to_string());
            commands.insert("pnpm".to_string(), "bun".to_string());
            commands.insert("npx".to_string(), "bunx".to_string());
            commands.insert("npm start".to_string(), "bun dev".to_string());
            commands.insert("npm test".to_string(), "bun test".to_string());
            commands.insert("npm run build".to_string(), "bun run build".to_string());
        }
        "Python" => {
            commands.insert("pip".to_string(), "uv pip".to_string());
            commands.insert("pip install".to_string(), "uv add".to_string());
            commands.insert("pip uninstall".to_string(), "uv remove".to_string());
            commands.insert("python".to_string(), "uv run python".to_string());
            commands.insert("python -m".to_string(), "uv run python -m".to_string());
        }
        "Rust" => {
            commands.insert("cargo check".to_string(), "cargo clippy".to_string());
            commands.insert("cargo test".to_string(), "cargo test -- --nocapture".to_string());
        }
        "Go" => {
            commands.insert("go run".to_string(), "go run -race".to_string());
            commands.insert("go test".to_string(), "go test -v".to_string());
        }
        "Java" => {
            commands.insert("mvn".to_string(), "./mvnw".to_string());
            commands.insert("gradle".to_string(), "./gradlew".to_string());
        }
        "Docker" => {
            commands.insert("docker".to_string(), "podman".to_string());
            commands.insert("docker-compose".to_string(), "podman-compose".to_string());
        }
        _ => {
            commands.insert("cat".to_string(), "bat".to_string());
            commands.insert("ls".to_string(), "eza".to_string());
            commands.insert("grep".to_string(), "rg".to_string());
            commands.insert("find".to_string(), "fd".to_string());
        }
    }
    
    // Add common safety and modern tool mappings for all project types
    commands.insert("curl".to_string(), "curl -L".to_string());
    commands.insert("rm".to_string(), "trash".to_string());
    commands.insert("rm -rf".to_string(), "echo 'Use trash command for safety'".to_string());
    
    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_config_for_project() {
        // Test each project type generates valid TOML (using actual project type names)
        let node_config = generate_config_for_project("Node.js");
        assert!(node_config.contains("[commands]"));
        assert!(node_config.contains("npm"));
        
        let python_config = generate_config_for_project("Python");
        assert!(python_config.contains("[commands]"));
        assert!(python_config.contains("pip"));
        
        let rust_config = generate_config_for_project("Rust");
        assert!(rust_config.contains("[commands]"));
        assert!(rust_config.contains("cargo"));
        
        let unknown_config = generate_config_for_project("unknown");
        assert!(unknown_config.contains("[commands]"));
        assert!(unknown_config.contains("cat"));
    }

    #[test]
    fn test_get_commands_for_project_type() {
        // Test Node.js mappings
        let node_commands = get_commands_for_project_type("Node.js");
        assert!(node_commands.contains_key("npm"));
        assert!(node_commands.contains_key("yarn"));
        assert_eq!(node_commands.get("npm"), Some(&"bun".to_string()));
        
        // Test Python mappings
        let python_commands = get_commands_for_project_type("Python");
        assert!(python_commands.contains_key("pip"));
        assert_eq!(python_commands.get("pip"), Some(&"uv pip".to_string()));
        
        // Test unknown project type (falls back to general commands)
        let unknown_commands = get_commands_for_project_type("unknown");
        assert!(!unknown_commands.is_empty()); // Should return generic commands
    }
}