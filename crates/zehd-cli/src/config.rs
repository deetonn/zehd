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
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PathsConfig {
    #[serde(default = "default_routes")]
    pub routes: String,

    #[serde(default = "default_lib")]
    pub lib: String,

    #[serde(rename = "static", default = "default_static")]
    pub static_dir: String,

    #[serde(default)]
    pub ignore: IgnoreConfig,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            routes: default_routes(),
            lib: default_lib(),
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

fn default_routes() -> String {
    "./routes".to_string()
}

fn default_lib() -> String {
    "./lib".to_string()
}

fn default_static() -> String {
    "./public".to_string()
}
