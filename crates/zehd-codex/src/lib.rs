pub mod ast;
pub mod error;
mod expr;
mod parser;
mod precedence;

use ast::Program;
use error::{ErrorCode, ParseError};
use parser::Parser;

/// Result of parsing a zehd source string.
pub struct ParseResult<'a> {
    pub program: Program,
    pub errors: Vec<ParseError>,
    pub source: &'a str,
}

impl<'a> ParseResult<'a> {
    /// Returns `true` if the parse produced no errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Parse a zehd source string into an AST.
///
/// Always returns a result — errors are collected, not fatal. The program
/// will contain whatever was successfully parsed; the errors list contains
/// all diagnostics.
pub fn parse(source: &str) -> ParseResult<'_> {
    let lex_result = zehd_tome::lex(source);

    // Convert lex errors into parse errors
    let mut errors: Vec<ParseError> = lex_result
        .errors
        .iter()
        .map(|e| {
            ParseError::error(ErrorCode::E001, e.message(), e.span)
                .label(e.span, "lexer error")
                .build()
        })
        .collect();

    let mut parser = Parser::new(&lex_result.tokens, source);
    let program = parser.parse_program();
    errors.append(&mut parser.errors);

    ParseResult {
        program,
        errors,
        source,
    }
}
