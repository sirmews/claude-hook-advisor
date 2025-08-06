//! CLI interface and main entry point

use crate::config::load_config;
use crate::directory::resolve_directory;
use crate::hooks::run_as_hook;
use crate::installer::run_installer;
use crate::types::Config;
use anyhow::{anyhow, Result};
use clap::{Arg, Command};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Main entry point for the Claude Hook Advisor application.
/// 
/// Parses command-line arguments and dispatches to the appropriate mode:
/// - `--hook`: Run as a Claude Code PreToolUse hook (reads JSON from stdin)
/// - `--install`: Interactive installer to set up project configuration
/// - Default: Show usage information
pub fn run_cli() -> Result<()> {
    let matches = Command::new("claude-hook-advisor")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Advises Claude Code on better command alternatives based on project preferences")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Path to configuration file")
                .default_value(".claude-hook-advisor.toml"),
        )
        .arg(
            Arg::new("hook")
                .long("hook")
                .help("Run as a Claude Code hook (reads JSON from stdin)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("replace")
                .long("replace")
                .help("Replace commands instead of blocking (experimental, not yet supported by Claude Code)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("install")
                .long("install")
                .help("Install and configure Claude Hook Advisor for this project")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("install-hooks")
                .long("install-hooks")
                .help("Install hooks configuration directly into Claude Code settings with backup")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("uninstall-hooks")
                .long("uninstall-hooks")
                .help("Remove Claude Hook Advisor hooks from Claude Code settings with backup")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("add-directory-alias")
                .long("add-directory-alias")
                .help("Add a semantic directory alias (e.g., --add-directory-alias 'docs' '~/Documents/Documentation')")
                .value_names(["ALIAS", "PATH"])
                .num_args(2),
        )
        .arg(
            Arg::new("list-directory-aliases")
                .long("list-directory-aliases")
                .help("List all configured semantic directory aliases")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("resolve-directory")
                .long("resolve-directory")
                .help("Resolve a semantic directory alias to its canonical path")
                .value_name("ALIAS"),
        )
        .arg(
            Arg::new("remove-directory-alias")
                .long("remove-directory-alias")
                .help("Remove a semantic directory alias by name")
                .value_name("ALIAS"),
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config")
        .expect("config argument has default value");
    let replace_mode = matches.get_flag("replace");

    if matches.get_flag("hook") {
        run_as_hook(config_path, replace_mode)
    } else if matches.get_flag("install") {
        run_installer(config_path)
    } else if matches.get_flag("install-hooks") {
        crate::installer::install_claude_hooks()
    } else if matches.get_flag("uninstall-hooks") {
        crate::installer::uninstall_claude_hooks()
    } else if let Some(values) = matches.get_many::<String>("add-directory-alias") {
        let args: Vec<&String> = values.collect();
        add_directory_alias(config_path, args[0], args[1])
    } else if matches.get_flag("list-directory-aliases") {
        list_directory_aliases(config_path)
    } else if let Some(alias) = matches.get_one::<String>("resolve-directory") {
        resolve_directory_alias(config_path, alias)
    } else if let Some(alias) = matches.get_one::<String>("remove-directory-alias") {
        remove_directory_alias(config_path, alias)
    } else {
        println!("Claude Hook Advisor v{}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("Command Mapping:");
        println!("  --hook                    Run as a Claude Code hook");
        println!("  --install                 Interactive installer for project configuration");
        println!("  --install-hooks           Install hooks directly into Claude Code settings");
        println!("  --uninstall-hooks         Remove hooks from Claude Code settings");
        println!();
        println!("Directory Management:");
        println!("  --add-directory-alias <ALIAS> <PATH>     Add semantic directory alias");
        println!("  --list-directory-aliases                 List all directory aliases");
        println!("  --resolve-directory <ALIAS>              Resolve alias to canonical path");
        println!("  --remove-directory-alias <ALIAS>         Remove directory alias");
        println!();
        println!("Configuration:");
        println!("  -c, --config <FILE>       Path to config file [default: .claude-hook-advisor.toml]");
        Ok(())
    }
}

/// Adds a semantic directory alias to the configuration.
/// 
/// Creates or updates the configuration file to include a new directory alias
/// mapping. If the configuration file doesn't exist, creates it with defaults.
/// 
/// # Arguments
/// * `config_path` - Path to the configuration file
/// * `alias` - The alias name (e.g., "docs")
/// * `path` - The directory path (e.g., "~/Documents/Documentation")
/// 
/// # Returns
/// * `Ok(())` - Alias added successfully
/// * `Err` - If file operations or serialization fails
fn add_directory_alias(config_path: &str, alias: &str, path: &str) -> Result<()> {
    let mut config = load_config_or_default(config_path)?;
    
    config.semantic_directories.insert(alias.to_string(), path.to_string());
    
    save_config(&config, config_path)?;
    
    println!("Added directory alias: '{alias}' -> '{path}'");
    Ok(())
}

/// Lists all configured semantic directory aliases.
/// 
/// Loads the configuration and displays all semantic directory mappings
/// in a human-readable format.
/// 
/// # Arguments
/// * `config_path` - Path to the configuration file
/// 
/// # Returns
/// * `Ok(())` - Aliases listed successfully
/// * `Err` - If configuration loading fails
fn list_directory_aliases(config_path: &str) -> Result<()> {
    let config = load_config_or_default(config_path)?;
    
    if config.semantic_directories.is_empty() {
        println!("No directory aliases configured.");
        return Ok(());
    }
    
    println!("Semantic Directory Aliases:");
    for (alias, path) in &config.semantic_directories {
        println!("  {alias} -> {path}");
    }
    
    Ok(())
}

/// Resolves a semantic directory alias to its canonical path.
/// 
/// Attempts to resolve the given alias using the current configuration,
/// including variable substitution and security validation.
/// 
/// # Arguments
/// * `config_path` - Path to the configuration file
/// * `alias` - The alias to resolve
/// 
/// # Returns
/// * `Ok(())` - Resolution successful (path printed)
/// * `Err` - If alias not found or resolution fails
fn resolve_directory_alias(config_path: &str, alias: &str) -> Result<()> {
    let config = load_config_or_default(config_path)?;
    
    match resolve_directory(&config, alias) {
        Ok(resolution) => {
            println!("Alias '{alias}' resolves to:");
            println!("  Canonical path: {}", resolution.canonical_path);
            if !resolution.variables_substituted.is_empty() {
                println!("  Variables substituted:");
                for (var, value) in resolution.variables_substituted {
                    println!("    {var} = {value}");
                }
            }
        }
        Err(e) => {
            return Err(anyhow!("Failed to resolve alias '{}': {}", alias, e));
        }
    }
    
    Ok(())
}

/// Removes a semantic directory alias from the configuration.
/// 
/// Updates the configuration file to remove the specified alias mapping.
/// 
/// # Arguments
/// * `config_path` - Path to the configuration file
/// * `alias` - The alias to remove
/// 
/// # Returns
/// * `Ok(())` - Alias removed successfully
/// * `Err` - If configuration operations fail
fn remove_directory_alias(config_path: &str, alias: &str) -> Result<()> {
    let mut config = load_config_or_default(config_path)?;
    
    if config.semantic_directories.remove(alias).is_some() {
        save_config(&config, config_path)?;
        println!("Removed directory alias: '{alias}'");
    } else {
        println!("Directory alias '{alias}' not found.");
    }
    
    Ok(())
}

/// Loads configuration or returns default if file doesn't exist.
/// 
/// Unlike the standard load_config function, this doesn't print warnings
/// for missing files and creates a proper default configuration.
/// 
/// # Arguments
/// * `config_path` - Path to the configuration file
/// 
/// # Returns
/// * `Ok(Config)` - Loaded or default configuration
/// * `Err` - If file exists but cannot be read/parsed
fn load_config_or_default(config_path: &str) -> Result<Config> {
    if Path::new(config_path).exists() {
        load_config(config_path)
    } else {
        Ok(Config {
            commands: HashMap::new(),
            semantic_directories: HashMap::new(),
            directory_variables: Default::default(),
        })
    }
}

/// Saves configuration to a TOML file.
/// 
/// Serializes the configuration struct to TOML format and writes it to
/// the specified file path.
/// 
/// # Arguments
/// * `config` - Configuration to save
/// * `config_path` - Path where to save the configuration
/// 
/// # Returns
/// * `Ok(())` - Configuration saved successfully
/// * `Err` - If serialization or file writing fails
fn save_config(config: &Config, config_path: &str) -> Result<()> {
    let toml_content = toml::to_string_pretty(config)?;
    fs::write(config_path, toml_content)?;
    Ok(())
}