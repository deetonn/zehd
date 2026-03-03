mod helpers;
use helpers::*;
use zehd_tome::{lex, TokenKind};

#[test]
fn empty_input() {
    let result = lex_ok("");
    assert_eq!(result.tokens.len(), 1); // just Eof
    assert_eq!(result.tokens[0].kind, TokenKind::Eof);
}

#[test]
fn whitespace_only() {
    let result = lex_ok("   \t\n\r\n  ");
    assert_eq!(result.tokens.len(), 1); // just Eof
    assert_eq!(result.tokens[0].kind, TokenKind::Eof);
}

#[test]
fn only_comments() {
    let result = lex_ok("// comment\n/* block */");
    assert_eq!(result.tokens.len(), 1); // just Eof
}

#[test]
fn many_spaces_between_tokens() {
    let k = kinds("let     x     =     42");
    assert_eq!(
        k,
        vec![
            TokenKind::Let,
            TokenKind::Identifier,
            TokenKind::Eq,
            TokenKind::Integer(42),
        ]
    );
}

#[test]
fn no_spaces_between_tokens() {
    // Tokens that don't need whitespace separation
    let k = kinds("(42)");
    assert_eq!(
        k,
        vec![
            TokenKind::LeftParen,
            TokenKind::Integer(42),
            TokenKind::RightParen,
        ]
    );
}

#[test]
fn crlf_line_endings() {
    let k = kinds("let\r\nx");
    assert_eq!(k, vec![TokenKind::Let, TokenKind::Identifier]);
}

#[test]
fn tabs_as_whitespace() {
    let k = kinds("let\tx");
    assert_eq!(k, vec![TokenKind::Let, TokenKind::Identifier]);
}

#[test]
fn mixed_delimiters() {
    let k = kinds("({[]})");
    assert_eq!(
        k,
        vec![
            TokenKind::LeftParen,
            TokenKind::LeftBrace,
            TokenKind::LeftBracket,
            TokenKind::RightBracket,
            TokenKind::RightBrace,
            TokenKind::RightParen,
        ]
    );
}

#[test]
fn consecutive_operators() {
    let k = kinds("!!");
    assert_eq!(k, vec![TokenKind::Bang, TokenKind::Bang]);
}

#[test]
fn float_not_confused_with_dot_access() {
    // "a.b" should be Identifier Dot Identifier, not a float
    let k = kinds("a.b");
    assert_eq!(
        k,
        vec![TokenKind::Identifier, TokenKind::Dot, TokenKind::Identifier]
    );
}

#[test]
fn number_followed_by_dot_identifier() {
    // "42.x" — 42 is an integer, then .x
    let k = kinds("42 .x");
    assert_eq!(
        k,
        vec![
            TokenKind::Integer(42),
            TokenKind::Dot,
            TokenKind::Identifier,
        ]
    );
}

#[test]
fn dollar_without_quote_is_error() {
    let result = lex("$x");
    // $ alone should be unexpected
    assert!(!result.errors.is_empty());
}

#[test]
fn eof_token_always_present() {
    let result = lex_ok("42");
    assert_eq!(result.tokens.last().unwrap().kind, TokenKind::Eof);

    let result = lex_ok("");
    assert_eq!(result.tokens.last().unwrap().kind, TokenKind::Eof);
}

#[test]
fn hash_then_bracket_are_separate() {
    let k = kinds("#[");
    assert_eq!(k, vec![TokenKind::Hash, TokenKind::LeftBracket]);
}

#[test]
fn two_dots_are_two_dots() {
    // .. is not a valid token — it's two Dots
    let k = kinds("..");
    assert_eq!(k, vec![TokenKind::Dot, TokenKind::Dot]);
}

#[test]
fn four_dots() {
    // .... is DotDotDot + Dot
    let k = kinds("....");
    assert_eq!(k, vec![TokenKind::DotDotDot, TokenKind::Dot]);
}

#[test]
fn unterminated_interpolated_string() {
    let result = lex("$\"hello");
    assert!(!result.errors.is_empty());
    assert!(result
        .errors
        .iter()
        .any(|e| matches!(e.kind, zehd_tome::LexErrorKind::UnterminatedString)));
}

#[test]
fn catch_all_route_syntax() {
    // [...path] as it appears in route file names
    let k = kinds("[...path]");
    assert_eq!(
        k,
        vec![
            TokenKind::LeftBracket,
            TokenKind::DotDotDot,
            TokenKind::Identifier,
            TokenKind::RightBracket,
        ]
    );
}

#[test]
fn negative_number_is_minus_then_int() {
    let k = kinds("-42");
    assert_eq!(k, vec![TokenKind::Minus, TokenKind::Integer(42)]);
}

#[test]
fn semicolons_everywhere() {
    let k = kinds(";;;");
    assert_eq!(
        k,
        vec![
            TokenKind::Semicolon,
            TokenKind::Semicolon,
            TokenKind::Semicolon,
        ]
    );
}
