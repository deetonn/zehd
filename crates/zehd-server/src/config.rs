use std::path::PathBuf;

/// Options passed from the CLI to start the server.
pub struct ServerOptions {
    /// Host to bind to (e.g. "0.0.0.0").
    pub host: String,
    /// Port to listen on (e.g. 3000).
    pub port: u16,
    /// Absolute path to the routes directory.
    pub routes_dir: PathBuf,
}
