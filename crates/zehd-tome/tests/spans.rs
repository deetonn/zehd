mod helpers;
use helpers::*;
use zehd_tome::{Span, TokenKind};

#[test]
fn single_token_span() {
    let toks = tokens("let");
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].span, Span::new(0, 3));
}

#[test]
fn multiple_tokens_span() {
    let toks = tokens("let x = 42;");
    // let
    assert_eq!(toks[0].span, Span::new(0, 3));
    assert_eq!(toks[0].kind, TokenKind::Let);
    // x
    assert_eq!(toks[1].span, Span::new(4, 5));
    assert_eq!(toks[1].kind, TokenKind::Identifier);
    // =
    assert_eq!(toks[2].span, Span::new(6, 7));
    assert_eq!(toks[2].kind, TokenKind::Eq);
    // 42
    assert_eq!(toks[3].span, Span::new(8, 10));
    assert_eq!(toks[3].kind, TokenKind::Integer(42));
    // ;
    assert_eq!(toks[4].span, Span::new(10, 11));
    assert_eq!(toks[4].kind, TokenKind::Semicolon);
}

#[test]
fn multiline_spans() {
    let source = "let x\nlet y";
    let toks = tokens(source);
    // let
    assert_eq!(toks[0].span, Span::new(0, 3));
    // x
    assert_eq!(toks[1].span, Span::new(4, 5));
    // second let (after \n)
    assert_eq!(toks[2].span, Span::new(6, 9));
    // y
    assert_eq!(toks[3].span, Span::new(10, 11));
}

#[test]
fn string_span_includes_quotes() {
    let source = r#""hello""#;
    let toks = tokens(source);
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].span, Span::new(0, 7)); // includes both quotes
}

#[test]
fn two_char_operator_span() {
    let toks = tokens("==");
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].span, Span::new(0, 2));
}

#[test]
fn time_literal_span() {
    let toks = tokens("60s");
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].span, Span::new(0, 3));
}

#[test]
fn dotdotdot_span() {
    let toks = tokens("...");
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].span, Span::new(0, 3));
}

#[test]
fn eof_span_at_end() {
    let result = lex_ok("abc");
    let eof = result.tokens.last().unwrap();
    assert_eq!(eof.kind, TokenKind::Eof);
    assert_eq!(eof.span, Span::new(3, 3));
}

#[test]
fn span_lexeme_extraction() {
    let source = "const name = 42;";
    let result = lex_ok(source);
    let toks: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    assert_eq!(toks[0].lexeme(source), "const");
    assert_eq!(toks[1].lexeme(source), "name");
    assert_eq!(toks[2].lexeme(source), "=");
    assert_eq!(toks[3].lexeme(source), "42");
    assert_eq!(toks[4].lexeme(source), ";");
}

#[test]
fn span_len() {
    let span = Span::new(5, 10);
    assert_eq!(span.len(), 5);
    assert!(!span.is_empty());
}

#[test]
fn span_empty() {
    let span = Span::new(5, 5);
    assert_eq!(span.len(), 0);
    assert!(span.is_empty());
}
