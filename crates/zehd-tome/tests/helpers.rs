#![allow(dead_code)]

use zehd_tome::{lex, LexResult, Token, TokenKind};

/// Lex source and assert no errors.
pub fn lex_ok(source: &str) -> LexResult<'_> {
    let result = lex(source);
    assert!(
        result.errors.is_empty(),
        "expected no errors, got: {:?}",
        result.errors
    );
    result
}

/// Lex source and return just the token kinds (excluding Eof).
pub fn kinds(source: &str) -> Vec<TokenKind> {
    let result = lex_ok(source);
    result
        .tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .map(|t| t.kind.clone())
        .collect()
}

/// Lex source and return (kind, lexeme) pairs (excluding Eof).
pub fn tokens_with_text<'a>(result: &'a LexResult<'a>) -> Vec<(&'a TokenKind, &'a str)> {
    result
        .tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .map(|t| (&t.kind, t.lexeme(result.source)))
        .collect()
}

/// Assert that a single-token input produces the expected kind.
pub fn assert_single(source: &str, expected: TokenKind) {
    let k = kinds(source);
    assert_eq!(k, vec![expected], "source: {source:?}");
}

/// Get the tokens (non-Eof) from a successful lex.
pub fn tokens(source: &str) -> Vec<Token> {
    let result = lex_ok(source);
    result
        .tokens
        .into_iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect()
}
