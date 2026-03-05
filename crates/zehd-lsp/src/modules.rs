use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use zehd_codex::ast::ItemKind;
use zehd_sigil::ModuleTypes;

/// A discovered module file (lightweight — no compilation needed).
#[derive(Debug)]
pub struct DiscoveredModule {
    pub module_path: String,
    pub file_path: PathBuf,
    pub source: String,
}

/// Discover all `.z` files under the configured module directories.
///
/// Each entry in `module_dirs` is `(namespace, abs_path)`, e.g. `("lib", "/project/lib")`.
pub fn discover_modules(module_dirs: &[(String, PathBuf)]) -> Vec<DiscoveredModule> {
    let mut modules = Vec::new();
    for (namespace, dir) in module_dirs {
        if !dir.is_dir() {
            continue;
        }
        walk_module_dir(namespace, dir, dir, &mut modules);
    }
    modules.sort_by(|a, b| a.module_path.cmp(&b.module_path));
    modules
}

fn walk_module_dir(
    namespace: &str,
    base: &Path,
    dir: &Path,
    modules: &mut Vec<DiscoveredModule>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_module_dir(namespace, base, &path, modules);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("z") {
            continue;
        }
        let module_path = file_path_to_module_path(namespace, base, &path);
        let source = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        modules.push(DiscoveredModule {
            module_path,
            file_path: path,
            source,
        });
    }
}

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

/// Parse, type-check, and extract types from discovered modules in dependency order.
///
/// Returns an enriched `ModuleTypes` containing both the base types (std) and
/// all user module exports. On errors (parse failure, cycles), the broken module
/// is skipped — we never break diagnostics for the file being edited.
pub fn extract_module_types(
    discovered: Vec<DiscoveredModule>,
    base_types: &ModuleTypes,
) -> ModuleTypes {
    if discovered.is_empty() {
        return base_types.clone();
    }

    // Build module_path set for dependency filtering.
    let module_set: HashSet<&str> = discovered.iter().map(|m| m.module_path.as_str()).collect();

    // Parse all modules.
    let mut parsed: HashMap<&str, (zehd_codex::ParseResult<'_>, &DiscoveredModule)> =
        HashMap::new();
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();

    for module in &discovered {
        let parse_result = zehd_codex::parse(&module.source);
        if !parse_result.is_ok() {
            // Skip modules that fail to parse.
            continue;
        }

        // Extract inter-module dependencies.
        let mut module_deps = Vec::new();
        for item in &parse_result.program.items {
            if let ItemKind::Import(imp) = &item.kind {
                let import_path = imp
                    .path
                    .segments
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join("::");
                if module_set.contains(import_path.as_str()) {
                    module_deps.push(import_path);
                }
            }
        }

        deps.insert(module.module_path.clone(), module_deps);
        parsed.insert(module.module_path.as_str(), (parse_result, module));
    }

    // Topological sort.
    let all_paths: Vec<String> = parsed.keys().map(|s| s.to_string()).collect();
    let sorted = match topological_sort(&all_paths, &deps) {
        Ok(s) => s,
        Err(_) => {
            // Cycle detected — return base types only.
            return base_types.clone();
        }
    };

    // Type-check in dependency order, accumulating exports.
    let mut accumulated = base_types.clone();

    for module_path in &sorted {
        let Some((pr, dm)) = parsed.get(module_path.as_str()) else {
            continue;
        };

        let check_result = zehd_sigil::check(&pr.program, &dm.source, &accumulated);

        // Extract exports: top-level functions → look up resolved types.
        let mut exports: HashMap<String, zehd_sigil::types::Type> = HashMap::new();
        for item in &pr.program.items {
            if let ItemKind::Function(f) = &item.kind {
                if let Some((_, sym)) = check_result.scopes.lookup(0, &f.name.name) {
                    exports.insert(f.name.name.clone(), sym.ty.clone());
                }
            }
        }

        if !exports.is_empty() {
            accumulated.insert(module_path.clone(), exports);
        }
    }

    accumulated
}

/// Kahn's algorithm topological sort — same logic as zehd-server/src/compile.rs.
fn topological_sort(
    nodes: &[String],
    deps: &HashMap<String, Vec<String>>,
) -> Result<Vec<String>, ()> {
    let node_set: HashSet<&str> = nodes.iter().map(|s| s.as_str()).collect();

    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    for node in nodes {
        in_degree.entry(node.as_str()).or_insert(0);
    }

    for (node, node_deps) in deps {
        for dep in node_deps {
            if node_set.contains(dep.as_str()) {
                *in_degree.entry(node.as_str()).or_insert(0) += 1;
                dependents
                    .entry(dep.as_str())
                    .or_default()
                    .push(node.as_str());
            }
        }
    }

    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&node, _)| node)
        .collect();
    queue.sort();

    let mut sorted = Vec::new();

    while let Some(node) = queue.pop() {
        sorted.push(node.to_string());
        if let Some(deps) = dependents.get(node) {
            for &dep in deps {
                let deg = in_degree.get_mut(dep).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push(dep);
                    queue.sort();
                }
            }
        }
    }

    if sorted.len() != nodes.len() {
        return Err(());
    }

    Ok(sorted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_finds_z_files() {
        let dir = tempfile::tempdir().unwrap();
        let lib = dir.path().join("lib");
        fs::create_dir_all(lib.join("sub")).unwrap();
        fs::write(lib.join("math.z"), "fn add(a: int, b: int): int { a + b; }").unwrap();
        fs::write(lib.join("sub/util.z"), "fn noop(): unit { }").unwrap();
        fs::write(lib.join("readme.md"), "skip").unwrap();

        let dirs = vec![("lib".to_string(), lib)];
        let modules = discover_modules(&dirs);
        assert_eq!(modules.len(), 2);
        let paths: Vec<&str> = modules.iter().map(|m| m.module_path.as_str()).collect();
        assert!(paths.contains(&"lib::math"));
        assert!(paths.contains(&"lib::sub::util"));
    }

    #[test]
    fn discover_skips_missing_dir() {
        let dirs = vec![("lib".to_string(), PathBuf::from("/nonexistent"))];
        let modules = discover_modules(&dirs);
        assert!(modules.is_empty());
    }

    #[test]
    fn topo_sort_no_deps() {
        let nodes = vec!["a".into(), "b".into()];
        let deps = HashMap::from([("a".into(), vec![]), ("b".into(), vec![])]);
        let sorted = topological_sort(&nodes, &deps).unwrap();
        assert_eq!(sorted.len(), 2);
    }

    #[test]
    fn topo_sort_linear() {
        let nodes = vec!["a".into(), "b".into()];
        let deps = HashMap::from([("a".into(), vec![]), ("b".into(), vec!["a".into()])]);
        let sorted = topological_sort(&nodes, &deps).unwrap();
        assert_eq!(sorted, vec!["a", "b"]);
    }

    #[test]
    fn topo_sort_cycle() {
        let nodes = vec!["a".into(), "b".into()];
        let deps = HashMap::from([
            ("a".into(), vec!["b".into()]),
            ("b".into(), vec!["a".into()]),
        ]);
        assert!(topological_sort(&nodes, &deps).is_err());
    }

    #[test]
    fn extract_types_from_empty() {
        let base = zehd_sigil::std_module_types();
        let result = extract_module_types(vec![], &base);
        assert_eq!(result.len(), base.len());
    }
}
