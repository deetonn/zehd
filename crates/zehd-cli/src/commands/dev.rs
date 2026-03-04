use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use owo_colors::OwoColorize;

use crate::cli::DevArgs;
use crate::config::ZehdConfig;

pub async fn run(args: DevArgs) -> Result<()> {
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

    // Resolve routes directory to absolute path
    let routes_dir = std::env::current_dir()?.join(&config.paths.routes);
    let routes_dir = routes_dir.canonicalize().unwrap_or(routes_dir);

    let options = zehd_server::config::ServerOptions {
        host: config.server.host,
        port,
        routes_dir,
    };

    zehd_server::start(options).await?;

    Ok(())
}
