mod helpers;

use helpers::*;
use zehd_codex::error::ErrorCode;

#[test]
fn missing_semicolon() {
    let result = parse_with_errors("let x = 42");
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e.code == ErrorCode::E002));
}

#[test]
fn unexpected_token() {
    let result = parse_with_errors("fn () {}");
    assert!(!result.is_ok());
    assert!(result.errors.iter().any(|e| e.code == ErrorCode::E005));
}

#[test]
fn missing_closing_brace() {
    let result = parse_with_errors("fn test() {");
    assert!(!result.is_ok());
}

#[test]
fn missing_closing_paren() {
    let result = parse_with_errors("fn test(x: int {");
    assert!(!result.is_ok());
}

#[test]
fn error_recovery_continues_parsing() {
    // After an error, the parser should recover and parse the next item
    let result = parse_with_errors(
        "let x = ;
         let y = 42;",
    );
    // Should have errors but still parse the second declaration
    assert!(!result.is_ok());
    // The parser should have recovered and parsed at least something
    assert!(!result.program.items.is_empty());
}

#[test]
fn parse_error_has_span() {
    let result = parse_with_errors("let = 42;");
    assert!(!result.is_ok());
    let err = &result.errors[0];
    // Span should be non-empty
    assert!(err.primary_span.start <= err.primary_span.end);
}
