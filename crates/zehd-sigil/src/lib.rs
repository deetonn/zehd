pub mod checker;
pub mod error;
pub mod infer;
pub mod optimize;
pub mod resolve;
pub mod scope;
pub mod types;

use zehd_codex::ast::Program;

use checker::{Checker, TypeTable};
use error::TypeError;
use optimize::Optimizer;
use resolve::Resolver;
use scope::ScopeArena;

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
pub fn check(program: &Program, _source: &str) -> CheckResult {
    // Pass 1: Resolve names.
    let resolver = Resolver::new();
    let resolve_result = resolver.resolve(program);

    // Pass 2: Type check.
    let checker = Checker::new(resolve_result);
    let checker_result = checker.check(program);

    let mut errors = checker_result.errors;
    let scopes = checker_result.scopes;
    let types = checker_result.types;

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
    }
}
