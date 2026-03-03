mod helpers;
use helpers::*;
use zehd_tome::TokenKind;

// ── Integers ─────────────────────────────────────────────────────

#[test]
fn integer_zero() {
    assert_single("0", TokenKind::Integer(0));
}

#[test]
fn integer_positive() {
    assert_single("42", TokenKind::Integer(42));
    assert_single("1234567890", TokenKind::Integer(1234567890));
}

#[test]
fn integer_large() {
    assert_single("9999999999", TokenKind::Integer(9999999999));
}

// ── Floats ───────────────────────────────────────────────────────

#[test]
fn float_simple() {
    assert_single("3.14", TokenKind::Float(3.14));
}

#[test]
fn float_zero() {
    assert_single("0.0", TokenKind::Float(0.0));
}

#[test]
fn float_leading_zero() {
    assert_single("0.5", TokenKind::Float(0.5));
}

#[test]
fn float_multiple_decimals() {
    assert_single("123.456", TokenKind::Float(123.456));
}

// ── Time Literals ────────────────────────────────────────────────

#[test]
fn time_milliseconds() {
    assert_single("500ms", TokenKind::TimeLiteral(500));
    assert_single("0ms", TokenKind::TimeLiteral(0));
    assert_single("1ms", TokenKind::TimeLiteral(1));
}

#[test]
fn time_seconds() {
    assert_single("30s", TokenKind::TimeLiteral(30_000));
    assert_single("1s", TokenKind::TimeLiteral(1_000));
    assert_single("60s", TokenKind::TimeLiteral(60_000));
}

#[test]
fn time_minutes() {
    assert_single("5m", TokenKind::TimeLiteral(300_000));
    assert_single("1m", TokenKind::TimeLiteral(60_000));
}

#[test]
fn time_hours() {
    assert_single("1h", TokenKind::TimeLiteral(3_600_000));
    assert_single("24h", TokenKind::TimeLiteral(86_400_000));
}

#[test]
fn time_literal_disambiguation_with_space() {
    // 60 s → Integer(60) + Identifier("s")
    let k = kinds("60 s");
    assert_eq!(k, vec![TokenKind::Integer(60), TokenKind::Identifier]);
}

#[test]
fn time_literal_disambiguation_no_space() {
    // 60s → TimeLiteral(60000)
    assert_single("60s", TokenKind::TimeLiteral(60_000));
}

// ── Strings ──────────────────────────────────────────────────────

#[test]
fn simple_string() {
    let result = lex_ok(r#""hello""#);
    let toks: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].kind, TokenKind::String);
    assert_eq!(toks[0].lexeme(result.source), r#""hello""#);
}

#[test]
fn empty_string() {
    let result = lex_ok(r#""""#);
    let toks: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].kind, TokenKind::String);
    assert_eq!(toks[0].lexeme(result.source), r#""""#);
}

#[test]
fn string_with_escapes() {
    let result = lex_ok(r#""hello\nworld""#);
    let toks: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].kind, TokenKind::String);
}

#[test]
fn string_with_escaped_quote() {
    let result = lex_ok(r#""say \"hello\"""#);
    let toks: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].kind, TokenKind::String);
}

#[test]
fn string_with_all_escapes() {
    let result = lex_ok(r#""\n\t\r\\\"\0""#);
    assert!(result.errors.is_empty());
    let k = kinds(r#""\n\t\r\\\"\0""#);
    assert_eq!(k, vec![TokenKind::String]);
}
