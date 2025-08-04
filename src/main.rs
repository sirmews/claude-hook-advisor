//! Claude Hook Advisor binary entry point

use claude_hook_advisor::cli::run_cli;

fn main() -> anyhow::Result<()> {
    run_cli()
}