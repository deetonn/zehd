use crate::token::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexErrorKind {
    UnterminatedString,
    UnterminatedBlockComment,
    UnexpectedCharacter(char),
    InvalidNumberSuffix(String),
    InvalidEscapeSequence(char),
    EmptyTimeLiteral(String),
    NumberOverflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub span: Span,
}

impl LexError {
    pub fn new(kind: LexErrorKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn message(&self) -> String {
        match &self.kind {
            LexErrorKind::UnterminatedString => "unterminated string literal".into(),
            LexErrorKind::UnterminatedBlockComment => "unterminated block comment".into(),
            LexErrorKind::UnexpectedCharacter(c) => format!("unexpected character '{c}'"),
            LexErrorKind::InvalidNumberSuffix(s) => {
                format!("invalid suffix '{s}' after number literal")
            }
            LexErrorKind::InvalidEscapeSequence(c) => {
                format!("invalid escape sequence '\\{c}'")
            }
            LexErrorKind::EmptyTimeLiteral(s) => {
                format!("time suffix '{s}' requires a numeric value")
            }
            LexErrorKind::NumberOverflow => "number literal overflows".into(),
        }
    }
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for LexError {}
