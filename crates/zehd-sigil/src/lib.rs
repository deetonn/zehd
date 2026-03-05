pub mod builtin_methods;
pub mod checker;
pub mod error;
pub mod infer;
pub mod optimize;
pub mod resolve;
pub mod scope;
pub mod types;

use std::collections::HashMap;

use zehd_codex::ast::{NodeId, Program};

use checker::{Checker, TypeTable};
use error::TypeError;
use optimize::Optimizer;
use resolve::Resolver;
use scope::ScopeArena;
use types::Type;

/// Maps module path (e.g. `"std"`, `"std::log"`) to its exported names and types.
pub type ModuleTypes = HashMap<String, HashMap<String, Type>>;

use types::{FunctionType, StructType};

/// Build the type signatures for the zehd standard library.
///
/// This is used by both the server (for compilation) and the LSP (for
/// diagnostics). Implementations live in `zehd-server/src/std_lib.rs`.
pub fn std_module_types() -> ModuleTypes {
    let mut m = ModuleTypes::new();

    // std — top-level standard library functions
    m.insert(
        "std".to_string(),
        HashMap::from([
            (
                "env".to_string(),
                Type::Function(FunctionType {
                    type_params: vec![],
                    type_param_vars: vec![],
                    params: vec![Type::String],
                    return_type: Box::new(Type::Option(Box::new(Type::String))),
                }),
            ),
            (
                "provide".to_string(),
                Type::Function(FunctionType {
                    type_params: vec![],
                    type_param_vars: vec![],
                    params: vec![], // special-cased in checker
                    return_type: Box::new(Type::Unit),
                }),
            ),
            (
                "inject".to_string(),
                Type::Function(FunctionType {
                    type_params: vec![],
                    type_param_vars: vec![],
                    params: vec![], // special-cased in checker
                    return_type: Box::new(Type::Unit), // overridden per call site
                }),
            ),
        ]),
    );

    // std::log — logging functions
    m.insert(
        "std::log".to_string(),
        HashMap::from([
            (
                "info".to_string(),
                Type::Function(FunctionType {
                    type_params: vec![],
                    type_param_vars: vec![],
                    params: vec![Type::String],
                    return_type: Box::new(Type::Unit),
                }),
            ),
            (
                "warn".to_string(),
                Type::Function(FunctionType {
                    type_params: vec![],
                    type_param_vars: vec![],
                    params: vec![Type::String],
                    return_type: Box::new(Type::Unit),
                }),
            ),
        ]),
    );

    // std::http — Request and Response types
    m.insert(
        "std::http".to_string(),
        HashMap::from([
            (
                "Request".to_string(),
                Type::Struct(StructType {
                    name: Some("Request".to_string()),
                    fields: vec![
                        ("method".to_string(), Type::String),
                        ("path".to_string(), Type::String),
                        ("headers".to_string(), Type::Map(Box::new(Type::String), Box::new(Type::String))),
                        ("body".to_string(), Type::String),
                        ("query".to_string(), Type::String),
                    ],
                    type_params: vec![],
                }),
            ),
            (
                "Response".to_string(),
                Type::Struct(StructType {
                    name: Some("Response".to_string()),
                    fields: vec![
                        ("status".to_string(), Type::Int),
                    ],
                    type_params: vec![],
                }),
            ),
        ]),
    );

    m
}

// ── Public API ──────────────────────────────────────────────────

/// Result of type checking and optimizing a zehd program.
pub struct CheckResult {
    /// NodeId → Type mapping for every expression.
    pub types: TypeTable,
    /// The scope tree built during resolution.
    pub scopes: ScopeArena,
    /// All diagnostics (errors and warnings).
    pub errors: Vec<TypeError>,
    /// The optimized program, if optimization succeeded.
    pub optimized_program: Option<Program>,
    /// NodeId → method_id for built-in method calls.
    pub method_calls: HashMap<NodeId, u16>,
}

impl CheckResult {
    /// Returns `true` if there are no errors (warnings are ok).
    pub fn is_ok(&self) -> bool {
        !self.has_errors()
    }

    /// Returns `true` if any error-severity diagnostic was reported.
    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| e.is_error())
    }

    /// Returns only the warning-severity diagnostics.
    pub fn warnings(&self) -> Vec<&TypeError> {
        self.errors.iter().filter(|e| e.is_warning()).collect()
    }
}

/// Type-check and optimize a parsed zehd program.
///
/// This is the main entry point for semantic analysis. It runs three passes:
///
/// 1. **Resolve** — name resolution, scope building, forward references
/// 2. **Check** — type inference and checking
/// 3. **Optimize** — constant folding, dead code elimination, const inlining
///
/// The input `program` is not consumed. The optimized version (if any) is
/// returned in `CheckResult.optimized_program`.
pub fn check(program: &Program, _source: &str, module_types: &ModuleTypes) -> CheckResult {
    // Pass 1: Resolve names.
    let resolver = Resolver::new();
    let resolve_result = resolver.resolve(program);

    // Pass 2: Type check.
    let checker = Checker::new(resolve_result, module_types.clone());
    let checker_result = checker.check(program);

    let mut errors = checker_result.errors;
    let scopes = checker_result.scopes;
    let types = checker_result.types;
    let method_calls = checker_result.method_calls;

    // Pass 3: Optimize (clone the program since we mutate).
    let mut optimized = program.clone();
    let optimizer = Optimizer::new();
    let opt_warnings = optimizer.optimize(&mut optimized, &scopes);
    errors.extend(opt_warnings);

    CheckResult {
        types,
        scopes,
        errors,
        optimized_program: Some(optimized),
        method_calls,
    }
}
