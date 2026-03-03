mod cursor;
pub mod error;
pub mod token;
pub mod lexer;

pub use error::{LexError, LexErrorKind};
pub use token::{keyword_from_str, Span, Token, TokenKind};

use lexer::Lexer;

/// Result of lexing a source string.
pub struct LexResult<'a> {
    pub tokens: Vec<Token>,
    pub errors: Vec<LexError>,
    pub source: &'a str,
}

impl<'a> LexResult<'a> {
    /// Returns `true` if the lex produced no errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Lex a source string into tokens.
///
/// Always returns a result — errors are collected, not fatal. The token stream
/// ends with [`TokenKind::Eof`].
pub fn lex(source: &str) -> LexResult<'_> {
    let lexer = Lexer::new(source);
    let (tokens, errors) = lexer.lex();
    LexResult {
        tokens,
        errors,
        source,
    }
}
