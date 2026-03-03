// ── Call Frame ──────────────────────────────────────────────────

/// Identifies which chunk a call frame is executing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkSource {
    /// The main chunk passed to execute().
    Main,
    /// A function from CompiledModule.functions\[index\].
    Function(u16),
}

/// A single activation record on the call stack.
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Which chunk this frame is executing.
    pub source: ChunkSource,
    /// Instruction pointer (byte offset into chunk code).
    pub ip: usize,
    /// Base index in the value stack where this frame's locals begin.
    pub stack_base: usize,
}
