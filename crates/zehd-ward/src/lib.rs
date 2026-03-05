pub mod error;
pub mod frame;
pub mod vm;

use std::sync::Arc;

use error::RuntimeError;
use zehd_rune::chunk::Chunk;
use zehd_rune::module::CompiledModule;
use zehd_rune::value::Value;

/// A native (Rust-implemented) function callable from zehd bytecode.
pub type NativeFn = fn(&[Value]) -> Result<Value, RuntimeError>;

// ── Module Function ───────────────────────────────────────────

/// A user-defined module function callable via `CallModule`.
#[derive(Clone)]
pub struct ModuleFunction {
    /// Index into the module's `CompiledModule.functions` vec.
    pub func_index: u16,
    /// The compiled module this function belongs to.
    pub compiled_module: Arc<CompiledModule>,
    /// Snapshot of the module's globals (after server_init).
    pub globals: Arc<Vec<Value>>,
}

// ── Context ────────────────────────────────────────────────────

/// Execution context passed to the VM.
pub struct Context {
    pub module: CompiledModule,
    /// Native functions indexed by NativeFnId.
    pub native_fns: Arc<Vec<NativeFn>>,
    /// User module functions indexed by ModuleFnId.
    pub module_fns: Arc<Vec<ModuleFunction>>,
}

// ── VmBackend Trait ────────────────────────────────────────────

/// Trait for VM backends (stack VM now, Cranelift JIT later).
pub trait VmBackend {
    /// Execute a chunk in the given context and return the result.
    fn execute(
        &mut self,
        chunk: &Chunk,
        context: &Context,
    ) -> Result<zehd_rune::value::Value, RuntimeError>;
}
