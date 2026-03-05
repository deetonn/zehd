use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tower_lsp::lsp_types::Url;

#[derive(Deserialize)]
struct LspConfig {
    #[serde(default)]
    paths: LspPathsConfig,
}

#[derive(Deserialize)]
struct LspPathsConfig {
    #[serde(default = "default_modules")]
    modules: Vec<String>,
}

impl Default for LspPathsConfig {
    fn default() -> Self {
        Self {
            modules: default_modules(),
        }
    }
}

fn default_modules() -> Vec<String> {
    vec!["lib".to_string()]
}

/// Walk up from a file URI's directory looking for `zehd.toml`.
/// Returns the directory containing the config file.
pub fn find_project_root(file_uri: &Url) -> Option<PathBuf> {
    let path = file_uri.to_file_path().ok()?;
    let mut dir = path.parent()?;
    loop {
        if dir.join("zehd.toml").is_file() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// Parse `zehd.toml` and return `(namespace, abs_path)` pairs for module directories.
/// Falls back to `["lib"]` default if no config found.
pub fn load_module_dirs(project_root: &Path) -> Vec<(String, PathBuf)> {
    let config_path = project_root.join("zehd.toml");
    let module_names = if let Ok(content) = fs::read_to_string(&config_path) {
        match toml::from_str::<LspConfig>(&content) {
            Ok(cfg) => cfg.paths.modules,
            Err(_) => default_modules(),
        }
    } else {
        default_modules()
    };

    module_names
        .into_iter()
        .map(|name| {
            let abs_path = project_root.join(&name);
            (name, abs_path)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn find_project_root_walks_up() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("routes");
        fs::create_dir_all(&sub).unwrap();
        fs::write(dir.path().join("zehd.toml"), "").unwrap();
        let file = sub.join("index.z");
        fs::write(&file, "").unwrap();

        let uri = Url::from_file_path(&file).unwrap();
        let root = find_project_root(&uri).unwrap();
        assert_eq!(root, dir.path());
    }

    #[test]
    fn load_module_dirs_default() {
        let dir = tempfile::tempdir().unwrap();
        // No zehd.toml → default ["lib"]
        let dirs = load_module_dirs(dir.path());
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].0, "lib");
        assert_eq!(dirs[0].1, dir.path().join("lib"));
    }

    #[test]
    fn load_module_dirs_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("zehd.toml"),
            "[paths]\nmodules = [\"lib\", \"shared\"]\n",
        )
        .unwrap();
        let dirs = load_module_dirs(dir.path());
        assert_eq!(dirs.len(), 2);
        assert_eq!(dirs[0].0, "lib");
        assert_eq!(dirs[1].0, "shared");
    }
}
