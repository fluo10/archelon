mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "archelon", about = "Markdown-based task and note manager")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage entries
    Entry {
        #[command(subcommand)]
        action: commands::entry::EntryCommand,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Entry { action } => commands::entry::run(action)?,
    }

    Ok(())
}
