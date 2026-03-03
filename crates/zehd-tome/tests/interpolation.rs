mod helpers;
use helpers::*;
use zehd_tome::TokenKind;

#[test]
fn simple_interpolation() {
    // $"Hello, {name}!"
    let source = r#"$"Hello, {name}!""#;
    let k = kinds(source);
    assert_eq!(
        k,
        vec![
            TokenKind::InterpolatedStringStart,
            TokenKind::StringFragment, // "Hello, "
            TokenKind::InterpolatedExprStart,
            TokenKind::Identifier, // name
            TokenKind::InterpolatedExprEnd,
            TokenKind::StringFragment, // "!"
            TokenKind::InterpolatedStringEnd,
        ]
    );
}

#[test]
fn interpolation_lexemes() {
    let source = r#"$"Hello, {name}!""#;
    let result = lex_ok(source);
    let toks: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .map(|t| (t.kind.clone(), t.lexeme(result.source)))
        .collect();

    assert_eq!(toks[0], (TokenKind::InterpolatedStringStart, "$\""));
    assert_eq!(toks[1], (TokenKind::StringFragment, "Hello, "));
    assert_eq!(toks[2], (TokenKind::InterpolatedExprStart, "{"));
    assert_eq!(toks[3], (TokenKind::Identifier, "name"));
    assert_eq!(toks[4], (TokenKind::InterpolatedExprEnd, "}"));
    assert_eq!(toks[5], (TokenKind::StringFragment, "!"));
    assert_eq!(toks[6], (TokenKind::InterpolatedStringEnd, "\""));
}

#[test]
fn interpolation_at_start() {
    // $"{x} done"
    let source = r#"$"{x} done""#;
    let k = kinds(source);
    assert_eq!(
        k,
        vec![
            TokenKind::InterpolatedStringStart,
            TokenKind::InterpolatedExprStart,
            TokenKind::Identifier,
            TokenKind::InterpolatedExprEnd,
            TokenKind::StringFragment,
            TokenKind::InterpolatedStringEnd,
        ]
    );
}

#[test]
fn interpolation_at_end() {
    // $"value: {x}"
    let source = r#"$"value: {x}""#;
    let k = kinds(source);
    assert_eq!(
        k,
        vec![
            TokenKind::InterpolatedStringStart,
            TokenKind::StringFragment,
            TokenKind::InterpolatedExprStart,
            TokenKind::Identifier,
            TokenKind::InterpolatedExprEnd,
            TokenKind::InterpolatedStringEnd,
        ]
    );
}

#[test]
fn interpolation_only_expr() {
    // $"{x}"
    let source = r#"$"{x}""#;
    let k = kinds(source);
    assert_eq!(
        k,
        vec![
            TokenKind::InterpolatedStringStart,
            TokenKind::InterpolatedExprStart,
            TokenKind::Identifier,
            TokenKind::InterpolatedExprEnd,
            TokenKind::InterpolatedStringEnd,
        ]
    );
}

#[test]
fn interpolation_with_expression() {
    // $"Total: {items.len() * price}"
    let source = r#"$"Total: {items.len() * price}""#;
    let k = kinds(source);
    assert_eq!(
        k,
        vec![
            TokenKind::InterpolatedStringStart,
            TokenKind::StringFragment,
            TokenKind::InterpolatedExprStart,
            TokenKind::Identifier, // items
            TokenKind::Dot,
            TokenKind::Identifier, // len
            TokenKind::LeftParen,
            TokenKind::RightParen,
            TokenKind::Star,
            TokenKind::Identifier, // price
            TokenKind::InterpolatedExprEnd,
            TokenKind::InterpolatedStringEnd,
        ]
    );
}

#[test]
fn multiple_interpolations() {
    // $"{a} and {b}"
    let source = r#"$"{a} and {b}""#;
    let k = kinds(source);
    assert_eq!(
        k,
        vec![
            TokenKind::InterpolatedStringStart,
            TokenKind::InterpolatedExprStart,
            TokenKind::Identifier,
            TokenKind::InterpolatedExprEnd,
            TokenKind::StringFragment, // " and "
            TokenKind::InterpolatedExprStart,
            TokenKind::Identifier,
            TokenKind::InterpolatedExprEnd,
            TokenKind::InterpolatedStringEnd,
        ]
    );
}

#[test]
fn nested_interpolation() {
    // $"outer {$"inner {x}"}"
    let source = r#"$"outer {$"inner {x}"}""#;
    let k = kinds(source);
    assert_eq!(
        k,
        vec![
            TokenKind::InterpolatedStringStart,  // outer $"
            TokenKind::StringFragment,           // "outer "
            TokenKind::InterpolatedExprStart,    // {
            TokenKind::InterpolatedStringStart,  // inner $"
            TokenKind::StringFragment,           // "inner "
            TokenKind::InterpolatedExprStart,    // {
            TokenKind::Identifier,               // x
            TokenKind::InterpolatedExprEnd,      // }
            TokenKind::InterpolatedStringEnd,    // "
            TokenKind::InterpolatedExprEnd,      // }
            TokenKind::InterpolatedStringEnd,    // "
        ]
    );
}

#[test]
fn interpolation_empty_string() {
    // $""
    let source = r#"$"""#;
    let k = kinds(source);
    assert_eq!(
        k,
        vec![
            TokenKind::InterpolatedStringStart,
            TokenKind::InterpolatedStringEnd,
        ]
    );
}

#[test]
fn interpolation_with_escape() {
    // $"line1\nline2"
    let source = "$\"line1\\nline2\"";
    let k = kinds(source);
    assert_eq!(
        k,
        vec![
            TokenKind::InterpolatedStringStart,
            TokenKind::StringFragment,
            TokenKind::InterpolatedStringEnd,
        ]
    );
}

#[test]
fn adjacent_interpolations() {
    // $"{a}{b}"
    let source = r#"$"{a}{b}""#;
    let k = kinds(source);
    assert_eq!(
        k,
        vec![
            TokenKind::InterpolatedStringStart,
            TokenKind::InterpolatedExprStart,
            TokenKind::Identifier,
            TokenKind::InterpolatedExprEnd,
            TokenKind::InterpolatedExprStart,
            TokenKind::Identifier,
            TokenKind::InterpolatedExprEnd,
            TokenKind::InterpolatedStringEnd,
        ]
    );
}
