pub mod error;
pub mod frame;
pub mod vm;

use error::RuntimeError;
use zehd_rune::chunk::Chunk;
use zehd_rune::module::CompiledModule;

// ── Context ────────────────────────────────────────────────────

/// Execution context passed to the VM.
pub struct Context {
    pub module: CompiledModule,
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
