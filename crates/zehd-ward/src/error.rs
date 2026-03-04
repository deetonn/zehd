use std::fmt;
use zehd_tome::Span;

// ── Error Codes ────────────────────────────────────────────────

/// Runtime error codes (R100–R199).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeErrorCode {
    /// Division by zero.
    R100,
    /// Stack underflow.
    R110,
    /// Unexpected type on stack.
    R120,
    /// Not a function.
    R121,
    /// Undefined global.
    R130,
    /// Unknown opcode.
    R140,
    /// Argument count mismatch.
    R150,
    /// Call stack overflow.
    R151,
    /// Function index out of bounds.
    R152,
    /// Unwrap failed (None or Err).
    R160,
    /// Index out of bounds.
    R161,
    /// Internal VM error.
    R190,
}

impl fmt::Display for RuntimeErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RuntimeErrorCode::R100 => "R100",
            RuntimeErrorCode::R110 => "R110",
            RuntimeErrorCode::R120 => "R120",
            RuntimeErrorCode::R121 => "R121",
            RuntimeErrorCode::R130 => "R130",
            RuntimeErrorCode::R140 => "R140",
            RuntimeErrorCode::R150 => "R150",
            RuntimeErrorCode::R151 => "R151",
            RuntimeErrorCode::R152 => "R152",
            RuntimeErrorCode::R160 => "R160",
            RuntimeErrorCode::R161 => "R161",
            RuntimeErrorCode::R190 => "R190",
        };
        write!(f, "{s}")
    }
}

// ── RuntimeError ───────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub code: RuntimeErrorCode,
    pub message: String,
    pub span: Option<Span>,
    pub notes: Vec<String>,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "runtime error[{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for RuntimeError {}

impl RuntimeError {
    pub fn err(
        code: RuntimeErrorCode,
        message: impl Into<String>,
    ) -> RuntimeErrorBuilder {
        RuntimeErrorBuilder {
            code,
            message: message.into(),
            span: None,
            notes: Vec::new(),
        }
    }
}

// ── Builder ────────────────────────────────────────────────────

pub struct RuntimeErrorBuilder {
    code: RuntimeErrorCode,
    message: String,
    span: Option<Span>,
    notes: Vec<String>,
}

impl RuntimeErrorBuilder {
    pub fn span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn note(mut self, message: impl Into<String>) -> Self {
        self.notes.push(message.into());
        self
    }

    pub fn build(self) -> RuntimeError {
        RuntimeError {
            code: self.code,
            message: self.message,
            span: self.span,
            notes: self.notes,
        }
    }
}
