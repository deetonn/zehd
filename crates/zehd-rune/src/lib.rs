pub mod chunk;
pub mod compiler;
pub mod error;
pub mod module;
pub mod op;
pub mod value;

use zehd_codex::ast::Program;
use zehd_sigil::CheckResult;

use compiler::Compiler;
use error::CompileError;
use module::CompiledModule;

// ── Public API ─────────────────────────────────────────────────

/// Result of compiling a type-checked zehd program.
pub struct CompileResult {
    /// The compiled module.
    pub module: CompiledModule,
    /// All diagnostics (errors and warnings).
    pub errors: Vec<CompileError>,
}

impl CompileResult {
    /// Returns `true` if compilation succeeded with no errors.
    pub fn is_ok(&self) -> bool {
        !self.has_errors()
    }

    /// Returns `true` if any error-severity diagnostic was reported.
    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| e.is_error())
    }

    /// Returns only the warning-severity diagnostics.
    pub fn warnings(&self) -> Vec<&CompileError> {
        self.errors.iter().filter(|e| e.is_warning()).collect()
    }
}

/// Compile a type-checked program into bytecode.
///
/// This is the main entry point. Takes a `CheckResult` from `zehd-sigil`
/// and the original parsed `Program`. Uses the optimized program if available.
///
/// # Panics
///
/// Panics if `check_result` contains type errors. Always check
/// `check_result.is_ok()` before calling this.
pub fn compile(program: &Program, check_result: CheckResult) -> CompileResult {
    // Take the optimized program out, falling back to the original.
    let optimized = check_result.optimized_program.clone();
    let target = optimized.as_ref().unwrap_or(program);

    let compiler = Compiler::new(check_result);
    let (module, errors) = compiler.compile(target);

    CompileResult { module, errors }
}
