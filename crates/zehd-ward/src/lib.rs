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

// ── Context ────────────────────────────────────────────────────

/// Execution context passed to the VM.
pub struct Context {
    pub module: CompiledModule,
    /// Native functions indexed by NativeFnId.
    pub native_fns: Arc<Vec<NativeFn>>,
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
