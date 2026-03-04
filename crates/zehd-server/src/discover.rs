use std::fs;
use std::path::{Path, PathBuf};

use crate::error::StartupError;

/// A discovered route file with its URL path and source.
#[derive(Debug)]
pub struct DiscoveredRoute {
    /// The URL path this route maps to (e.g. "/api/health").
    pub url_path: String,
    /// Absolute path to the .z file.
    pub file_path: PathBuf,
    /// File contents.
    pub source: String,
}

/// File names to skip during route discovery.
const SKIP_FILES: &[&str] = &["init.z", "error.z", "middleware.z", "layout.z"];

/// Discover all static route files under `routes_dir`.
///
/// Walks the directory recursively, converts file paths to URL paths,
/// and reads each file's contents. Skips special files and dynamic routes.
pub fn discover_routes(routes_dir: &Path) -> Result<Vec<DiscoveredRoute>, StartupError> {
    if !routes_dir.is_dir() {
        return Err(StartupError::RoutesNotFound(routes_dir.to_path_buf()));
    }

    let mut routes = Vec::new();
    walk_dir(routes_dir, routes_dir, &mut routes)?;
    routes.sort_by(|a, b| a.url_path.cmp(&b.url_path));
    Ok(routes)
}

fn walk_dir(
    base: &Path,
    dir: &Path,
    routes: &mut Vec<DiscoveredRoute>,
) -> Result<(), StartupError> {
    let entries = fs::read_dir(dir).map_err(|source| StartupError::IoError {
        path: dir.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| StartupError::IoError {
            path: dir.to_path_buf(),
            source,
        })?;

        let path = entry.path();

        if path.is_dir() {
            walk_dir(base, &path, routes)?;
            continue;
        }

        // Only process .z files
        if path.extension().and_then(|e| e.to_str()) != Some("z") {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Skip special files
        if SKIP_FILES.contains(&file_name) {
            continue;
        }

        // Skip dynamic routes (e.g. [id].z)
        if file_name.starts_with('[') && file_name.ends_with("].z") {
            continue;
        }

        let url_path = file_path_to_url(base, &path);
        let source = fs::read_to_string(&path).map_err(|source| StartupError::IoError {
            path: path.clone(),
            source,
        })?;

        routes.push(DiscoveredRoute {
            url_path,
            file_path: path,
            source,
        });
    }

    Ok(())
}

/// Convert a file path relative to the routes dir into a URL path.
///
/// - `index.z` → `/`
/// - `users/index.z` → `/users`
/// - `api/health.z` → `/api/health`
fn file_path_to_url(base: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(base).unwrap_or(file);
    let stem = relative.with_extension("");
    let mut url = String::from("/");

    for component in stem.components() {
        let part = component.as_os_str().to_str().unwrap_or("");
        if part == "index" {
            // index.z at any level maps to the parent directory
            continue;
        }
        if !url.ends_with('/') {
            url.push('/');
        }
        url.push_str(part);
    }

    url
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn url(base: &str, file: &str) -> String {
        file_path_to_url(&PathBuf::from(base), &PathBuf::from(file))
    }

    #[test]
    fn index_maps_to_root() {
        assert_eq!(url("routes", "routes/index.z"), "/");
    }

    #[test]
    fn nested_index() {
        assert_eq!(url("routes", "routes/users/index.z"), "/users");
    }

    #[test]
    fn named_file() {
        assert_eq!(url("routes", "routes/api/health.z"), "/api/health");
    }

    #[test]
    fn deeply_nested() {
        assert_eq!(
            url("routes", "routes/api/v1/users.z"),
            "/api/v1/users"
        );
    }

    #[test]
    fn discover_skips_special_files() {
        let dir = tempdir_with_files(&[
            "index.z",
            "init.z",
            "error.z",
            "middleware.z",
            "layout.z",
            "about.z",
        ]);
        let routes = discover_routes(dir.path()).unwrap();
        let urls: Vec<&str> = routes.iter().map(|r| r.url_path.as_str()).collect();
        assert!(urls.contains(&"/"));
        assert!(urls.contains(&"/about"));
        assert!(!urls.iter().any(|u| u.contains("init")));
        assert!(!urls.iter().any(|u| u.contains("error")));
        assert!(!urls.iter().any(|u| u.contains("middleware")));
        assert!(!urls.iter().any(|u| u.contains("layout")));
    }

    #[test]
    fn discover_skips_dynamic_routes() {
        let dir = tempdir_with_files(&["index.z", "[id].z"]);
        let routes = discover_routes(dir.path()).unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].url_path, "/");
    }

    #[test]
    fn discover_skips_non_z_files() {
        let dir = tempdir_with_files(&["index.z", "readme.md", "style.css"]);
        let routes = discover_routes(dir.path()).unwrap();
        assert_eq!(routes.len(), 1);
    }

    #[test]
    fn discover_returns_error_for_missing_dir() {
        let result = discover_routes(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    /// Helper: create a temp dir with empty files.
    fn tempdir_with_files(names: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for name in names {
            let path = dir.path().join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&path, "").unwrap();
        }
        dir
    }
}
