use std::path::PathBuf;

/// Options passed from the CLI to start the server.
pub struct ServerOptions {
    /// Host to bind to (e.g. "0.0.0.0").
    pub host: String,
    /// Port to listen on (e.g. 3000).
    pub port: u16,
    /// Absolute path to the routes directory.
    pub routes_dir: PathBuf,
    /// Root project directory (contains main.z, zehd.toml, etc.).
    pub project_dir: PathBuf,
    /// Maximum number of concurrent in-flight requests (OOM safety net).
    pub max_requests: usize,
    /// Whether to log each request to stdout.
    pub request_logging: bool,
}
