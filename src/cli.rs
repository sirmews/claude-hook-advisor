//! CLI interface and main entry point

use crate::hooks::run_as_hook;
use crate::installer::run_installer;
use anyhow::Result;
use clap::{Arg, Command};

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
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();
    let replace_mode = matches.get_flag("replace");

    if matches.get_flag("hook") {
        run_as_hook(config_path, replace_mode)
    } else if matches.get_flag("install") {
        run_installer(config_path)
    } else if matches.get_flag("install-hooks") {
        crate::installer::install_claude_hooks()
    } else if matches.get_flag("uninstall-hooks") {
        crate::installer::uninstall_claude_hooks()
    } else {
        println!("Claude Hook Advisor");
        println!("Use --hook flag to run as a Claude Code hook");
        println!("Use --install flag to set up configuration for this project");
        println!("Use --install-hooks flag to install hooks directly into Claude Code settings");
        println!("Use --uninstall-hooks flag to remove hooks from Claude Code settings");
        Ok(())
    }
}