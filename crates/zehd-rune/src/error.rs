use std::fmt;
use zehd_tome::Span;

// ── Error Codes ────────────────────────────────────────────────

/// Compile error codes (C100–C119).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileErrorCode {
    /// Internal: unexpected AST node during compilation.
    C100,
    /// Too many constants (> u16::MAX).
    C101,
    /// Too many locals (> u16::MAX).
    C102,
    /// Jump offset overflow (> u16::MAX).
    C103,
    /// Too many function parameters (> u8::MAX).
    C104,
    /// Unsupported feature (deferred to future phase).
    C110,
}

impl fmt::Display for CompileErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CompileErrorCode::C100 => "C100",
            CompileErrorCode::C101 => "C101",
            CompileErrorCode::C102 => "C102",
            CompileErrorCode::C103 => "C103",
            CompileErrorCode::C104 => "C104",
            CompileErrorCode::C110 => "C110",
        };
        write!(f, "{s}")
    }
}

// ── Severity ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

// ── Labels & Suggestions ───────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Applicability {
    MachineApplicable,
    MaybeIncorrect,
    HasPlaceholders,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    pub message: String,
    pub span: Span,
    pub replacement: String,
    pub applicability: Applicability,
}

// ── CompileError ───────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct CompileError {
    pub severity: Severity,
    pub code: CompileErrorCode,
    pub message: String,
    pub primary_span: Span,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub suggestions: Vec<Suggestion>,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{severity}[{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for CompileError {}

impl CompileError {
    pub fn error(
        code: CompileErrorCode,
        message: impl Into<String>,
        span: Span,
    ) -> CompileErrorBuilder {
        CompileErrorBuilder {
            severity: Severity::Error,
            code,
            message: message.into(),
            primary_span: span,
            labels: Vec::new(),
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn warning(
        code: CompileErrorCode,
        message: impl Into<String>,
        span: Span,
    ) -> CompileErrorBuilder {
        CompileErrorBuilder {
            severity: Severity::Warning,
            code,
            message: message.into(),
            primary_span: span,
            labels: Vec::new(),
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    pub fn is_warning(&self) -> bool {
        self.severity == Severity::Warning
    }
}

// ── Builder ────────────────────────────────────────────────────

pub struct CompileErrorBuilder {
    severity: Severity,
    code: CompileErrorCode,
    message: String,
    primary_span: Span,
    labels: Vec<Label>,
    notes: Vec<String>,
    suggestions: Vec<Suggestion>,
}

impl CompileErrorBuilder {
    pub fn label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label {
            span,
            message: message.into(),
        });
        self
    }

    pub fn note(mut self, message: impl Into<String>) -> Self {
        self.notes.push(message.into());
        self
    }

    #[allow(dead_code)]
    pub fn suggestion(
        mut self,
        message: impl Into<String>,
        span: Span,
        replacement: impl Into<String>,
        applicability: Applicability,
    ) -> Self {
        self.suggestions.push(Suggestion {
            message: message.into(),
            span,
            replacement: replacement.into(),
            applicability,
        });
        self
    }

    pub fn build(self) -> CompileError {
        CompileError {
            severity: self.severity,
            code: self.code,
            message: self.message,
            primary_span: self.primary_span,
            labels: self.labels,
            notes: self.notes,
            suggestions: self.suggestions,
        }
    }
}
