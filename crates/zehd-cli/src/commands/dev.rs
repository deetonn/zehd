use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use owo_colors::OwoColorize;

use crate::cli::DevArgs;
use crate::config::ZehdConfig;

pub fn run(args: DevArgs) -> Result<()> {
    let config_path = Path::new("zehd.toml");
    if !config_path.exists() {
        bail!(
            "No {} found in the current directory. Are you in a zehd project?",
            "zehd.toml".bold()
        );
    }

    let raw = fs::read_to_string(config_path)?;
    let config: ZehdConfig = toml::from_str(&raw)?;

    let port = args.port.unwrap_or(config.server.port);

    println!();
    println!(
        "  {} {}",
        "zehd".cyan().bold(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed()
    );
    println!();
    println!("  {}  http://{}:{}", "→".green(), config.server.host, port);
    println!("  {}  {}", "routes".dimmed(), config.paths.routes);
    println!("  {}  {}", "lib".dimmed(), config.paths.lib);
    println!("  {}  {}", "static".dimmed(), config.paths.static_dir);
    println!();
    println!(
        "  {}",
        "Dev server is not yet implemented. Coming soon!".yellow()
    );
    println!();

    Ok(())
}
