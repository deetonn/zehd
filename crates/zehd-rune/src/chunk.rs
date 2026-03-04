use zehd_tome::Span;

use crate::op::{encode_u16, Op};
use crate::value::Value;

// ── Span Tracking ──────────────────────────────────────────────

/// Maps a range of bytecode offsets to a source span.
#[derive(Debug, Clone, PartialEq)]
pub struct SpanEntry {
    /// Bytecode offset where this span starts.
    pub offset: u32,
    /// Source span for error reporting.
    pub span: Span,
}

// ── Chunk ──────────────────────────────────────────────────────

/// A compiled function or handler body — bytecode + metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    /// The bytecode.
    pub code: Vec<u8>,
    /// Constant pool.
    pub constants: Vec<Value>,
    /// Bytecode offset → source span mapping.
    pub spans: Vec<SpanEntry>,
    /// Debug name (e.g. "get", "main", "add").
    pub name: String,
    /// Number of local variable slots needed.
    pub local_count: u16,
    /// Number of parameters (for functions).
    pub arity: u8,
}

impl Chunk {
    /// Look up the source span for a bytecode offset.
    pub fn span_at(&self, offset: u32) -> Option<Span> {
        // Find the last span entry at or before this offset.
        let mut result = Option::None;
        for entry in &self.spans {
            if entry.offset <= offset {
                result = Some(entry.span);
            } else {
                break;
            }
        }
        result
    }
}

// ── ChunkBuilder ───────────────────────────────────────────────

/// Mutable builder for constructing a Chunk during compilation.
pub struct ChunkBuilder {
    code: Vec<u8>,
    constants: Vec<Value>,
    spans: Vec<SpanEntry>,
    name: String,
    pub local_count: u16,
    pub arity: u8,
}

impl ChunkBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            spans: Vec::new(),
            name: name.into(),
            local_count: 0,
            arity: 0,
        }
    }

    /// Current bytecode offset (next byte to be written).
    pub fn offset(&self) -> usize {
        self.code.len()
    }

    /// Peek at the last emitted byte (for checking if Return was already emitted).
    pub fn last_byte(&self) -> Option<u8> {
        self.code.last().copied()
    }

    /// Record a source span for the current bytecode offset.
    pub fn record_span(&mut self, span: Span) {
        self.spans.push(SpanEntry {
            offset: self.code.len() as u32,
            span,
        });
    }

    // ── Emit helpers ───────────────────────────────────────────

    /// Emit a simple opcode (no operand).
    pub fn emit(&mut self, op: Op, span: Span) {
        self.record_span(span);
        self.code.push(op as u8);
    }

    /// Emit an opcode with a u16 operand.
    pub fn emit_u16(&mut self, op: Op, operand: u16, span: Span) {
        self.record_span(span);
        self.code.push(op as u8);
        let bytes = encode_u16(operand);
        self.code.push(bytes[0]);
        self.code.push(bytes[1]);
    }

    /// Emit an opcode with a u8 operand.
    pub fn emit_u8(&mut self, op: Op, operand: u8, span: Span) {
        self.record_span(span);
        self.code.push(op as u8);
        self.code.push(operand);
    }

    /// Emit an opcode with a u16 operand followed by a u8 operand.
    pub fn emit_u16_u8(&mut self, op: Op, a: u16, b: u8, span: Span) {
        self.record_span(span);
        self.code.push(op as u8);
        let a_bytes = encode_u16(a);
        self.code.push(a_bytes[0]);
        self.code.push(a_bytes[1]);
        self.code.push(b);
    }

    /// Emit an opcode with two u16 operands.
    pub fn emit_u16_u16(&mut self, op: Op, a: u16, b: u16, span: Span) {
        self.record_span(span);
        self.code.push(op as u8);
        let a_bytes = encode_u16(a);
        let b_bytes = encode_u16(b);
        self.code.push(a_bytes[0]);
        self.code.push(a_bytes[1]);
        self.code.push(b_bytes[0]);
        self.code.push(b_bytes[1]);
    }

    /// Emit a jump instruction with a placeholder offset.
    /// Returns the bytecode offset of the placeholder for later patching.
    pub fn emit_jump(&mut self, op: Op, span: Span) -> usize {
        self.record_span(span);
        self.code.push(op as u8);
        let patch_offset = self.code.len();
        self.code.push(0xFF);
        self.code.push(0xFF);
        patch_offset
    }

    /// Patch a previously emitted jump placeholder with the current offset.
    pub fn patch_jump(&mut self, patch_offset: usize) {
        let target = self.code.len();
        let jump = target.saturating_sub(patch_offset + 2);
        let bytes = encode_u16(jump as u16);
        self.code[patch_offset] = bytes[0];
        self.code[patch_offset + 1] = bytes[1];
    }

    /// Emit a backward loop jump to `loop_start`.
    pub fn emit_loop(&mut self, loop_start: usize, span: Span) {
        self.record_span(span);
        self.code.push(Op::Loop as u8);
        // offset = current position + 2 (for the operand bytes) - loop_start
        let offset = self.code.len() + 2 - loop_start;
        let bytes = encode_u16(offset as u16);
        self.code.push(bytes[0]);
        self.code.push(bytes[1]);
    }

    // ── Constants ──────────────────────────────────────────────

    /// Add a constant to the pool, deduplicating if possible.
    /// Returns the constant index.
    pub fn add_constant(&mut self, value: Value) -> u16 {
        // Deduplicate: check if this exact value already exists.
        // Skip dedup for Function values since they're unique references.
        if !matches!(value, Value::Function(_)) {
            for (i, existing) in self.constants.iter().enumerate() {
                if *existing == value {
                    return i as u16;
                }
            }
        }
        let index = self.constants.len() as u16;
        self.constants.push(value);
        index
    }

    /// Emit a Constant opcode, adding the value to the pool.
    pub fn emit_constant(&mut self, value: Value, span: Span) -> u16 {
        let index = self.add_constant(value);
        self.emit_u16(Op::Constant, index, span);
        index
    }

    // ── Build ──────────────────────────────────────────────────

    /// Finalize into an immutable Chunk.
    pub fn build(self) -> Chunk {
        Chunk {
            code: self.code,
            constants: self.constants,
            spans: self.spans,
            name: self.name,
            local_count: self.local_count,
            arity: self.arity,
        }
    }
}
