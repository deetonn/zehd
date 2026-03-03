use std::fmt;
use zehd_tome::Span;

// ── Error Codes ─────────────────────────────────────────────────

/// Type error codes (T100–T169).
///
/// Distinct from parser's E-codes; same diagnostic infrastructure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeErrorCode {
    // ── Resolve errors (T100–T109) ──────────────────────────────
    T100, // Undefined variable
    T101, // Undefined type
    T102, // Duplicate definition
    T103, // Unresolved import
    T104, // Undefined field access
    T105, // Undefined enum variant

    // ── Type mismatch errors (T110–T129) ────────────────────────
    T110, // Type mismatch (expected X, found Y)
    T111, // Incompatible types in binary operation
    T112, // Incompatible types in if branches
    T113, // Non-boolean condition
    T114, // Wrong number of arguments
    T115, // Not a function (call on non-callable)
    T116, // Not indexable
    T117, // Try (?) on non-Result type
    T118, // Return type mismatch
    T119, // Incompatible match arm types
    T120, // Non-exhaustive match
    T121, // Wrong number of type arguments

    // ── Semantic errors (T130–T149) ─────────────────────────────
    T130, // Assignment to immutable variable (const)
    T131, // Break outside loop
    T132, // Continue outside loop
    T133, // Self outside handler context
    T134, // Invalid assignment target
    T135, // Missing return value

    // ── Warnings (T150–T169) ────────────────────────────────────
    T150, // Unreachable code
    T151, // Unreachable match arm
    T152, // Unused variable
    T153, // Unused import
    T154, // Unused function
}

impl fmt::Display for TypeErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TypeErrorCode::T100 => "T100",
            TypeErrorCode::T101 => "T101",
            TypeErrorCode::T102 => "T102",
            TypeErrorCode::T103 => "T103",
            TypeErrorCode::T104 => "T104",
            TypeErrorCode::T105 => "T105",
            TypeErrorCode::T110 => "T110",
            TypeErrorCode::T111 => "T111",
            TypeErrorCode::T112 => "T112",
            TypeErrorCode::T113 => "T113",
            TypeErrorCode::T114 => "T114",
            TypeErrorCode::T115 => "T115",
            TypeErrorCode::T116 => "T116",
            TypeErrorCode::T117 => "T117",
            TypeErrorCode::T118 => "T118",
            TypeErrorCode::T119 => "T119",
            TypeErrorCode::T120 => "T120",
            TypeErrorCode::T121 => "T121",
            TypeErrorCode::T130 => "T130",
            TypeErrorCode::T131 => "T131",
            TypeErrorCode::T132 => "T132",
            TypeErrorCode::T133 => "T133",
            TypeErrorCode::T134 => "T134",
            TypeErrorCode::T135 => "T135",
            TypeErrorCode::T150 => "T150",
            TypeErrorCode::T151 => "T151",
            TypeErrorCode::T152 => "T152",
            TypeErrorCode::T153 => "T153",
            TypeErrorCode::T154 => "T154",
        };
        write!(f, "{s}")
    }
}

// ── Severity ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

// ── Labels & Suggestions ────────────────────────────────────────

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

// ── TypeError ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub severity: Severity,
    pub code: TypeErrorCode,
    pub message: String,
    pub primary_span: Span,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub suggestions: Vec<Suggestion>,
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{severity}[{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for TypeError {}

impl TypeError {
    pub fn error(code: TypeErrorCode, message: impl Into<String>, span: Span) -> TypeErrorBuilder {
        TypeErrorBuilder {
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
        code: TypeErrorCode,
        message: impl Into<String>,
        span: Span,
    ) -> TypeErrorBuilder {
        TypeErrorBuilder {
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

// ── Builder ─────────────────────────────────────────────────────

pub struct TypeErrorBuilder {
    severity: Severity,
    code: TypeErrorCode,
    message: String,
    primary_span: Span,
    labels: Vec<Label>,
    notes: Vec<String>,
    suggestions: Vec<Suggestion>,
}

impl TypeErrorBuilder {
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

    pub fn build(self) -> TypeError {
        TypeError {
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
