use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

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

    /// Start the language server (LSP) on stdin/stdout
    Lsp,

    /// Dump lexer tokens for a file
    Tokens(FileArgs),

    /// Pretty-print the AST for a file
    Ast(AstArgs),

    /// Disassemble compiled bytecode for a file
    Bytecode(FileArgs),

    /// Type-check a file or project without running
    Check(CheckArgs),

    /// List all discovered routes and their HTTP methods
    Routes,

    /// Execute a .z file as a standalone script
    Run(FileArgs),
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

#[derive(Args)]
pub struct FileArgs {
    /// Path to a .z file
    pub file: PathBuf,
}

#[derive(Args)]
pub struct AstArgs {
    /// Path to a .z file
    pub file: PathBuf,

    /// Show types from the type checker
    #[arg(long)]
    pub typed: bool,
}

#[derive(Args)]
pub struct CheckArgs {
    /// Path to a .z file (omit to check entire project)
    pub file: Option<PathBuf>,
}
