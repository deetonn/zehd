use zehd_tome::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    E001, // Unexpected token
    E002, // Expected specific token
    E003, // Expected expression
    E004, // Expected statement
    E005, // Expected identifier
    E010, // Expected type annotation
    E011, // Expected function body
    E012, // Expected block
    E013, // Invalid import syntax
    E014, // Invalid type definition
    E015, // Invalid enum definition
    E016, // Invalid attribute syntax
    E017, // Invalid parameter list
    E020, // Invalid LHS of assignment
    E021, // Expected closing delimiter
    E022, // Invalid match arm
    E023, // Invalid pattern
    E024, // Expected arrow in match arm
    E025, // Invalid object literal field
    E026, // Invalid interpolated string
    E030, // Duplicate HTTP method block
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::E001 => write!(f, "E001"),
            ErrorCode::E002 => write!(f, "E002"),
            ErrorCode::E003 => write!(f, "E003"),
            ErrorCode::E004 => write!(f, "E004"),
            ErrorCode::E005 => write!(f, "E005"),
            ErrorCode::E010 => write!(f, "E010"),
            ErrorCode::E011 => write!(f, "E011"),
            ErrorCode::E012 => write!(f, "E012"),
            ErrorCode::E013 => write!(f, "E013"),
            ErrorCode::E014 => write!(f, "E014"),
            ErrorCode::E015 => write!(f, "E015"),
            ErrorCode::E016 => write!(f, "E016"),
            ErrorCode::E017 => write!(f, "E017"),
            ErrorCode::E020 => write!(f, "E020"),
            ErrorCode::E021 => write!(f, "E021"),
            ErrorCode::E022 => write!(f, "E022"),
            ErrorCode::E023 => write!(f, "E023"),
            ErrorCode::E024 => write!(f, "E024"),
            ErrorCode::E025 => write!(f, "E025"),
            ErrorCode::E026 => write!(f, "E026"),
            ErrorCode::E030 => write!(f, "E030"),
        }
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub severity: Severity,
    pub code: ErrorCode,
    pub message: String,
    pub primary_span: Span,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub suggestions: Vec<Suggestion>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{severity}[{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    pub fn error(code: ErrorCode, message: impl Into<String>, span: Span) -> ParseErrorBuilder {
        ParseErrorBuilder {
            severity: Severity::Error,
            code,
            message: message.into(),
            primary_span: span,
            labels: Vec::new(),
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn warning(code: ErrorCode, message: impl Into<String>, span: Span) -> ParseErrorBuilder {
        ParseErrorBuilder {
            severity: Severity::Warning,
            code,
            message: message.into(),
            primary_span: span,
            labels: Vec::new(),
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }
}

pub struct ParseErrorBuilder {
    severity: Severity,
    code: ErrorCode,
    message: String,
    primary_span: Span,
    labels: Vec<Label>,
    notes: Vec<String>,
    suggestions: Vec<Suggestion>,
}

impl ParseErrorBuilder {
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

    pub fn build(self) -> ParseError {
        ParseError {
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
