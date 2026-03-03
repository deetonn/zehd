mod cli;
mod commands;
mod config;
mod templates;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New(args) => commands::new::run(args),
        Commands::Dev(args) => commands::dev::run(args),
    }
}
