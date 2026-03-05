use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ZehdConfig {
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub paths: PathsConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_max_requests")]
    pub max_requests: usize,

    #[serde(default = "default_request_logging")]
    pub request_logging: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
            max_requests: default_max_requests(),
            request_logging: default_request_logging(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PathsConfig {
    #[serde(default = "default_routes")]
    pub routes: String,

    #[serde(default = "default_modules")]
    pub modules: Vec<String>,

    #[serde(rename = "static", default = "default_static")]
    pub static_dir: String,

    #[serde(default)]
    pub ignore: IgnoreConfig,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            routes: default_routes(),
            modules: default_modules(),
            static_dir: default_static(),
            ignore: IgnoreConfig::default(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct IgnoreConfig {
    #[serde(default)]
    pub dirs: Vec<String>,
}

fn default_port() -> u16 {
    3000
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_max_requests() -> usize {
    1024
}

fn default_request_logging() -> bool {
    true
}

fn default_routes() -> String {
    "./routes".to_string()
}

fn default_modules() -> Vec<String> {
    vec!["lib".to_string()]
}

fn default_static() -> String {
    "./public".to_string()
}

// ── Shared Config Loading ─────────────────────────────────────

/// Load `zehd.toml` from the current directory and resolve paths.
///
/// Returns `(config, project_dir, routes_dir, module_dirs)`.
pub fn load_project_config() -> anyhow::Result<ProjectConfig> {
    use std::fs;
    use std::path::Path;

    use anyhow::bail;
    use owo_colors::OwoColorize;

    let config_path = Path::new("zehd.toml");
    if !config_path.exists() {
        bail!(
            "No {} found in the current directory. Are you in a zehd project?",
            "zehd.toml".bold()
        );
    }

    let raw = fs::read_to_string(config_path)?;
    let config: ZehdConfig = toml::from_str(&raw)?;

    let project_dir = std::env::current_dir()?;

    let routes_dir = project_dir.join(&config.paths.routes);
    let routes_dir = routes_dir.canonicalize().unwrap_or(routes_dir);

    let module_dirs: Vec<(String, std::path::PathBuf)> = config
        .paths
        .modules
        .iter()
        .map(|dir_name| {
            let abs_path = project_dir.join(dir_name);
            let abs_path = abs_path.canonicalize().unwrap_or(abs_path);
            (dir_name.clone(), abs_path)
        })
        .collect();

    Ok(ProjectConfig {
        config,
        project_dir,
        routes_dir,
        module_dirs,
    })
}

/// Resolved project configuration.
pub struct ProjectConfig {
    pub config: ZehdConfig,
    pub project_dir: std::path::PathBuf,
    pub routes_dir: std::path::PathBuf,
    pub module_dirs: Vec<(String, std::path::PathBuf)>,
}
