use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use zehd_codex::ast::ItemKind;
use zehd_rune::module::CompiledModule;
use zehd_rune::registry::{ModuleFnRegistry, NativeRegistry};
use zehd_sigil::ModuleTypes;
use zehd_ward::vm::StackVm;
use zehd_ward::{ModuleFunction, NativeFn, VmBackend};

use crate::discover::{DiscoveredModule, DiscoveredRoute};
use crate::error::{RouteCompileError, StartupError};

/// A successfully compiled route.
pub struct CompiledRoute {
    pub url_path: String,
    pub module: CompiledModule,
}

/// Compile a set of discovered routes through the full pipeline.
///
/// Returns `(compiled, errors)`. If `errors` is non-empty, the caller
/// should refuse to start the server.
pub fn compile_routes(
    routes: Vec<DiscoveredRoute>,
    module_types: &ModuleTypes,
    native_registry: &NativeRegistry,
    module_fn_registry: &ModuleFnRegistry,
) -> (Vec<CompiledRoute>, Vec<RouteCompileError>) {
    let mut compiled = Vec::new();
    let mut errors = Vec::new();

    for route in routes {
        match compile_one(&route, module_types, native_registry, module_fn_registry) {
            Ok(module) => {
                compiled.push(CompiledRoute {
                    url_path: route.url_path,
                    module,
                });
            }
            Err(messages) => {
                errors.push(RouteCompileError {
                    file_path: route.file_path,
                    url_path: route.url_path,
                    messages,
                });
            }
        }
    }

    (compiled, errors)
}

/// Run a single source through parse → check → compile.
/// Returns the compiled module or a list of error messages.
pub fn compile_one(
    route: &DiscoveredRoute,
    module_types: &ModuleTypes,
    native_registry: &NativeRegistry,
    module_fn_registry: &ModuleFnRegistry,
) -> Result<CompiledModule, Vec<String>> {
    // Phase 1: Parse
    let parse_result = zehd_codex::parse(&route.source);
    if !parse_result.is_ok() {
        let messages: Vec<String> = parse_result
            .errors
            .iter()
            .map(|e| e.to_string())
            .collect();
        return Err(messages);
    }

    // Phase 2: Type check
    let check_result =
        zehd_sigil::check(&parse_result.program, &route.source, module_types);
    if check_result.has_errors() {
        let messages: Vec<String> = check_result
            .errors
            .iter()
            .filter(|e| e.is_error())
            .map(|e| e.to_string())
            .collect();
        return Err(messages);
    }

    // Phase 3: Compile to bytecode
    let compile_result =
        zehd_rune::compile(&parse_result.program, check_result, native_registry, module_fn_registry);
    if compile_result.has_errors() {
        let messages: Vec<String> = compile_result
            .errors
            .iter()
            .filter(|e| e.is_error())
            .map(|e| e.to_string())
            .collect();
        return Err(messages);
    }

    Ok(compile_result.module)
}

// ── Module Compilation ──────────────────────────────────────────

/// Result of compiling all user modules.
pub struct CompiledModules {
    pub module_types: ModuleTypes,
    pub module_fn_registry: ModuleFnRegistry,
    pub module_fns: Vec<ModuleFunction>,
}

/// Compile all discovered user modules in dependency order.
///
/// 1. Parse all modules to extract import dependencies
/// 2. Topological sort with cycle detection
/// 3. Compile each in order, accumulating types and function registrations
pub fn compile_modules(
    discovered: Vec<DiscoveredModule>,
    base_module_types: &ModuleTypes,
    native_registry: &NativeRegistry,
    native_fns: &Arc<Vec<NativeFn>>,
) -> Result<CompiledModules, StartupError> {
    if discovered.is_empty() {
        return Ok(CompiledModules {
            module_types: base_module_types.clone(),
            module_fn_registry: ModuleFnRegistry::default(),
            module_fns: vec![],
        });
    }

    // Build a map from module_path to discovered module for quick lookup.
    let module_map: HashMap<&str, &DiscoveredModule> = discovered
        .iter()
        .map(|m| (m.module_path.as_str(), m))
        .collect();

    // Parse all modules and extract inter-module dependencies.
    let mut parsed: HashMap<String, zehd_codex::ParseResult> = HashMap::new();
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();

    for module in &discovered {
        let parse_result = zehd_codex::parse(&module.source);
        if !parse_result.is_ok() {
            let msg = parse_result
                .errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(StartupError::ModuleCompileFailed {
                module_path: module.module_path.clone(),
                message: msg,
            });
        }

        // Extract imports that point to other user modules.
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
                if module_map.contains_key(import_path.as_str()) {
                    module_deps.push(import_path);
                }
            }
        }

        deps.insert(module.module_path.clone(), module_deps);
        parsed.insert(module.module_path.clone(), parse_result);
    }

    // Topological sort.
    let all_paths: Vec<String> = discovered.iter().map(|m| m.module_path.clone()).collect();
    let sorted = topological_sort(&all_paths, &deps)?;

    // Compile in dependency order.
    let mut accumulated_types = base_module_types.clone();
    let mut module_fn_registry = ModuleFnRegistry::default();
    let mut module_fns: Vec<ModuleFunction> = Vec::new();

    for module_path in &sorted {
        let dm = module_map[module_path.as_str()];
        let pr = &parsed[module_path];

        // Type check with accumulated types.
        let check_result =
            zehd_sigil::check(&pr.program, &dm.source, &accumulated_types);
        if check_result.has_errors() {
            let msg = check_result
                .errors
                .iter()
                .filter(|e| e.is_error())
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(StartupError::ModuleCompileFailed {
                module_path: module_path.clone(),
                message: msg,
            });
        }

        // Extract exports: top-level functions → look up types from scopes.
        let mut exports: HashMap<String, zehd_sigil::types::Type> = HashMap::new();
        for item in &pr.program.items {
            if let ItemKind::Function(f) = &item.kind {
                if let Some((_, sym)) = check_result.scopes.lookup(0, &f.name.name) {
                    exports.insert(f.name.name.clone(), sym.ty.clone());
                }
            }
        }

        // Add exports to accumulated module types.
        accumulated_types.insert(module_path.clone(), exports);

        // Compile to bytecode.
        let compile_result = zehd_rune::compile(
            &pr.program,
            check_result,
            native_registry,
            &module_fn_registry,
        );
        if compile_result.has_errors() {
            let msg = compile_result
                .errors
                .iter()
                .filter(|e| e.is_error())
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(StartupError::ModuleCompileFailed {
                module_path: module_path.clone(),
                message: msg,
            });
        }

        let compiled_module = Arc::new(compile_result.module);

        // Run server_init on a temp VM to snapshot globals.
        let temp_context = zehd_ward::Context {
            module: (*compiled_module).clone(),
            native_fns: Arc::clone(native_fns),
            module_fns: Arc::new(module_fns.clone()),
        };
        let mut vm = StackVm::new();
        if let Some(ref init_chunk) = compiled_module.server_init {
            vm.execute(init_chunk, &temp_context).map_err(|e| {
                StartupError::InitFailed {
                    url_path: module_path.clone(),
                    message: e.message,
                }
            })?;
        }
        let globals = Arc::new(vm.globals().to_vec());

        // Register each exported function.
        for item in &pr.program.items {
            if let ItemKind::Function(f) = &item.kind {
                // Find the function index in the compiled module.
                if let Some(func_index) = compiled_module
                    .functions
                    .iter()
                    .position(|fe| fe.name == f.name.name)
                {
                    let fn_id = module_fns.len() as u16;
                    module_fn_registry.register(module_path, &f.name.name, fn_id);
                    module_fns.push(ModuleFunction {
                        func_index: func_index as u16,
                        compiled_module: Arc::clone(&compiled_module),
                        globals: Arc::clone(&globals),
                    });
                }
            }
        }
    }

    Ok(CompiledModules {
        module_types: accumulated_types,
        module_fn_registry,
        module_fns,
    })
}

/// Topological sort with cycle detection using Kahn's algorithm.
pub fn topological_sort(
    nodes: &[String],
    deps: &HashMap<String, Vec<String>>,
) -> Result<Vec<String>, StartupError> {
    let node_set: HashSet<&str> = nodes.iter().map(|s| s.as_str()).collect();

    // Build in-degree map and adjacency (dep → dependents).
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

    // Start with nodes that have no dependencies.
    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&node, _)| node)
        .collect();
    queue.sort(); // deterministic order

    let mut sorted = Vec::new();

    while let Some(node) = queue.pop() {
        sorted.push(node.to_string());
        if let Some(deps) = dependents.get(node) {
            for &dep in deps {
                let deg = in_degree.get_mut(dep).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push(dep);
                    queue.sort(); // keep deterministic
                }
            }
        }
    }

    if sorted.len() != nodes.len() {
        // Find a cycle for the error message.
        let in_sorted: HashSet<&str> = sorted.iter().map(|s| s.as_str()).collect();
        let cycle_nodes: Vec<&str> = nodes
            .iter()
            .map(|s| s.as_str())
            .filter(|n| !in_sorted.contains(n))
            .collect();
        return Err(StartupError::CircularDependency {
            cycle: cycle_nodes.join(" → "),
        });
    }

    Ok(sorted)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> String {
        v.to_string()
    }

    #[test]
    fn topo_sort_no_deps() {
        let nodes = vec![s("a"), s("b"), s("c")];
        let deps: HashMap<String, Vec<String>> = nodes.iter().map(|n| (n.clone(), vec![])).collect();
        let sorted = topological_sort(&nodes, &deps).unwrap();
        // All nodes present, deterministic alphabetical order
        assert_eq!(sorted, vec!["c", "b", "a"]);
    }

    #[test]
    fn topo_sort_linear_chain() {
        // c depends on b, b depends on a
        let nodes = vec![s("a"), s("b"), s("c")];
        let deps = HashMap::from([
            (s("a"), vec![]),
            (s("b"), vec![s("a")]),
            (s("c"), vec![s("b")]),
        ]);
        let sorted = topological_sort(&nodes, &deps).unwrap();
        assert_eq!(sorted, vec!["a", "b", "c"]);
    }

    #[test]
    fn topo_sort_diamond() {
        // d depends on b,c; b,c both depend on a
        let nodes = vec![s("a"), s("b"), s("c"), s("d")];
        let deps = HashMap::from([
            (s("a"), vec![]),
            (s("b"), vec![s("a")]),
            (s("c"), vec![s("a")]),
            (s("d"), vec![s("b"), s("c")]),
        ]);
        let sorted = topological_sort(&nodes, &deps).unwrap();
        // a must come first, d must come last
        assert_eq!(sorted[0], "a");
        assert_eq!(sorted[3], "d");
    }

    #[test]
    fn topo_sort_cycle_detected() {
        let nodes = vec![s("a"), s("b")];
        let deps = HashMap::from([
            (s("a"), vec![s("b")]),
            (s("b"), vec![s("a")]),
        ]);
        let result = topological_sort(&nodes, &deps);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            StartupError::CircularDependency { cycle } => {
                assert!(cycle.contains("a"));
                assert!(cycle.contains("b"));
            }
            _ => panic!("expected CircularDependency"),
        }
    }
}
