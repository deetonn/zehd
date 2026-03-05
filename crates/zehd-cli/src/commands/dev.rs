use anyhow::Result;

use crate::cli::DevArgs;
use crate::config::load_project_config;

pub async fn run(args: DevArgs) -> Result<()> {
    let pc = load_project_config()?;

    let port = args.port.unwrap_or(pc.config.server.port);

    let options = zehd_server::config::ServerOptions {
        host: pc.config.server.host,
        port,
        routes_dir: pc.routes_dir,
        project_dir: pc.project_dir,
        max_requests: pc.config.server.max_requests,
        request_logging: pc.config.server.request_logging,
        module_dirs: pc.module_dirs,
    };

    zehd_server::start(options).await?;

    Ok(())
}
