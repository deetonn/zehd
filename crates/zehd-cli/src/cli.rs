use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "zehd",
    about = "The zehd programming language — a language that is a web server",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new zehd project
    New(NewArgs),

    /// Start the development server
    Dev(DevArgs),
}

#[derive(Parser)]
pub struct NewArgs {
    /// Project name (alphanumeric, hyphens, underscores)
    pub name: Option<String>,
}

#[derive(Parser)]
pub struct DevArgs {
    /// Port to run the dev server on
    #[arg(short, long)]
    pub port: Option<u16>,
}
