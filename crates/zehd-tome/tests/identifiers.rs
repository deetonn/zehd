mod helpers;
use helpers::*;
use zehd_tome::TokenKind;

#[test]
fn simple_identifier() {
    assert_single("foo", TokenKind::Identifier);
    assert_single("bar", TokenKind::Identifier);
    assert_single("x", TokenKind::Identifier);
}

#[test]
fn identifier_with_digits() {
    assert_single("foo123", TokenKind::Identifier);
    assert_single("x1", TokenKind::Identifier);
}

#[test]
fn identifier_with_underscores() {
    assert_single("foo_bar", TokenKind::Identifier);
    assert_single("_private", TokenKind::Identifier);
    assert_single("__dunder", TokenKind::Identifier);
    assert_single("a_b_c", TokenKind::Identifier);
}

#[test]
fn underscore_alone_is_special() {
    assert_single("_", TokenKind::Underscore);
}

#[test]
fn underscore_prefixed_is_identifier() {
    assert_single("_a", TokenKind::Identifier);
    assert_single("_1", TokenKind::Identifier);
    assert_single("_foo", TokenKind::Identifier);
}

#[test]
fn camel_case() {
    assert_single("camelCase", TokenKind::Identifier);
    assert_single("PascalCase", TokenKind::Identifier);
}

#[test]
fn screaming_snake() {
    assert_single("SCREAMING_SNAKE", TokenKind::Identifier);
    assert_single("API_KEY", TokenKind::Identifier);
}

#[test]
fn identifier_lexeme() {
    let result = lex_ok("myVariable");
    let toks: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].lexeme(result.source), "myVariable");
}
