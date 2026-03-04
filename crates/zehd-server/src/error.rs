use std::fmt;
use std::path::PathBuf;

// ── Route Compile Error ─────────────────────────────────────────

/// A compilation error for a single route file.
#[derive(Debug)]
pub struct RouteCompileError {
    pub file_path: PathBuf,
    pub url_path: String,
    pub messages: Vec<String>,
}

impl fmt::Display for RouteCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}): {} error(s)",
            self.url_path,
            self.file_path.display(),
            self.messages.len()
        )?;
        for msg in &self.messages {
            write!(f, "\n  {msg}")?;
        }
        Ok(())
    }
}

// ── Startup Error ───────────────────────────────────────────────

/// Errors that prevent the server from starting.
#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("routes directory not found: {0}")]
    RoutesNotFound(PathBuf),

    #[error("I/O error reading {path}: {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{count} route(s) failed to compile")]
    CompilationFailed {
        count: usize,
        errors: Vec<RouteCompileError>,
    },

    #[error("server_init failed for {url_path}: {message}")]
    InitFailed { url_path: String, message: String },

    #[error("failed to bind {host}:{port}: {source}")]
    BindError {
        host: String,
        port: u16,
        #[source]
        source: std::io::Error,
    },

    #[error("filesystem watcher failed: {source}")]
    WatcherError {
        #[source]
        source: notify::Error,
    },
}
