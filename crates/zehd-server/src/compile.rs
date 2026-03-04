use zehd_rune::module::CompiledModule;
use zehd_rune::registry::NativeRegistry;
use zehd_sigil::ModuleTypes;

use crate::discover::DiscoveredRoute;
use crate::error::RouteCompileError;

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
) -> (Vec<CompiledRoute>, Vec<RouteCompileError>) {
    let mut compiled = Vec::new();
    let mut errors = Vec::new();

    for route in routes {
        match compile_one(&route, module_types, native_registry) {
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
fn compile_one(
    route: &DiscoveredRoute,
    module_types: &ModuleTypes,
    native_registry: &NativeRegistry,
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
        zehd_rune::compile(&parse_result.program, check_result, native_registry);
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
