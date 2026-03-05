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

// ── Module Discovery ──────────────────────────────────────────

/// A discovered module file with its namespace and module path.
#[derive(Debug)]
pub struct DiscoveredModule {
    /// The namespace (directory name), e.g. "lib".
    pub namespace: String,
    /// The full module path, e.g. "lib::auth::password".
    pub module_path: String,
    /// Absolute path to the .z file.
    pub file_path: PathBuf,
    /// File contents.
    pub source: String,
}

/// Discover all module files under the configured module directories.
///
/// Each entry in `module_dirs` is `(namespace, abs_path)`, e.g. `("lib", "/project/lib")`.
/// Walks each directory recursively for `.z` files and converts to module paths.
pub fn discover_modules(
    module_dirs: &[(String, PathBuf)],
) -> Result<Vec<DiscoveredModule>, StartupError> {
    let mut modules = Vec::new();

    for (namespace, dir) in module_dirs {
        if !dir.is_dir() {
            continue; // Skip non-existent module dirs silently
        }
        walk_module_dir(namespace, dir, dir, &mut modules)?;
    }

    modules.sort_by(|a, b| a.module_path.cmp(&b.module_path));
    Ok(modules)
}

fn walk_module_dir(
    namespace: &str,
    base: &Path,
    dir: &Path,
    modules: &mut Vec<DiscoveredModule>,
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
            walk_module_dir(namespace, base, &path, modules)?;
            continue;
        }

        // Only process .z files
        if path.extension().and_then(|e| e.to_str()) != Some("z") {
            continue;
        }

        let module_path = file_path_to_module_path(namespace, base, &path);
        let source = fs::read_to_string(&path).map_err(|source| StartupError::IoError {
            path: path.clone(),
            source,
        })?;

        modules.push(DiscoveredModule {
            namespace: namespace.to_string(),
            module_path,
            file_path: path,
            source,
        });
    }

    Ok(())
}

/// Convert a file path relative to the module dir into a module path.
///
/// - `lib/auth.z` with namespace "lib" → `"lib::auth"`
/// - `lib/auth/password.z` with namespace "lib" → `"lib::auth::password"`
fn file_path_to_module_path(namespace: &str, base: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(base).unwrap_or(file);
    let stem = relative.with_extension("");
    let mut path = String::from(namespace);

    for component in stem.components() {
        let part = component.as_os_str().to_str().unwrap_or("");
        path.push_str("::");
        path.push_str(part);
    }

    path
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

    // ── Module Discovery Tests ────────────────────────────────

    fn module_path(ns: &str, base: &str, file: &str) -> String {
        file_path_to_module_path(ns, &PathBuf::from(base), &PathBuf::from(file))
    }

    #[test]
    fn module_path_simple() {
        assert_eq!(module_path("lib", "lib", "lib/auth.z"), "lib::auth");
    }

    #[test]
    fn module_path_nested() {
        assert_eq!(
            module_path("lib", "lib", "lib/auth/password.z"),
            "lib::auth::password"
        );
    }

    #[test]
    fn module_path_deep() {
        assert_eq!(
            module_path("lib", "lib", "lib/a/b/c.z"),
            "lib::a::b::c"
        );
    }

    #[test]
    fn discover_modules_finds_z_files() {
        let dir = tempfile::tempdir().unwrap();
        let lib_dir = dir.path().join("lib");
        fs::create_dir_all(lib_dir.join("auth")).unwrap();
        fs::write(lib_dir.join("math.z"), "fn add(a: int, b: int): int { a + b }").unwrap();
        fs::write(lib_dir.join("auth/hash.z"), "fn hash(s: string): string { s }").unwrap();
        fs::write(lib_dir.join("readme.md"), "# ignore me").unwrap();

        let module_dirs = vec![("lib".to_string(), lib_dir)];
        let modules = discover_modules(&module_dirs).unwrap();

        assert_eq!(modules.len(), 2);
        let paths: Vec<&str> = modules.iter().map(|m| m.module_path.as_str()).collect();
        assert!(paths.contains(&"lib::auth::hash"));
        assert!(paths.contains(&"lib::math"));
    }

    #[test]
    fn discover_modules_skips_missing_dir() {
        let module_dirs = vec![("lib".to_string(), PathBuf::from("/nonexistent/lib"))];
        let modules = discover_modules(&module_dirs).unwrap();
        assert!(modules.is_empty());
    }
}
