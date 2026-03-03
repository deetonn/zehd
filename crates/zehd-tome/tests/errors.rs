use zehd_tome::{lex, LexErrorKind, TokenKind};

#[test]
fn unterminated_string() {
    let result = lex(r#""hello"#);
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].kind, LexErrorKind::UnterminatedString);
}

#[test]
fn unterminated_string_newline() {
    let result = lex("\"hello\n");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].kind, LexErrorKind::UnterminatedString);
}

#[test]
fn unexpected_character() {
    let result = lex("@");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(
        result.errors[0].kind,
        LexErrorKind::UnexpectedCharacter('@')
    );
}

#[test]
fn invalid_number_suffix() {
    let result = lex("60something");
    assert_eq!(result.errors.len(), 1);
    assert!(matches!(
        &result.errors[0].kind,
        LexErrorKind::InvalidNumberSuffix(s) if s == "something"
    ));
}

#[test]
fn invalid_number_suffix_single_char() {
    let result = lex("42x");
    assert_eq!(result.errors.len(), 1);
    assert!(matches!(
        &result.errors[0].kind,
        LexErrorKind::InvalidNumberSuffix(s) if s == "x"
    ));
}

#[test]
fn single_ampersand_error() {
    let result = lex("&");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(
        result.errors[0].kind,
        LexErrorKind::UnexpectedCharacter('&')
    );
}

#[test]
fn single_pipe_error() {
    let result = lex("|");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(
        result.errors[0].kind,
        LexErrorKind::UnexpectedCharacter('|')
    );
}

#[test]
fn invalid_escape_sequence() {
    let result = lex(r#""\q""#);
    assert_eq!(result.errors.len(), 1);
    assert_eq!(
        result.errors[0].kind,
        LexErrorKind::InvalidEscapeSequence('q')
    );
    // The string token is still emitted despite the error
    let non_eof: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    assert_eq!(non_eof.len(), 1);
    assert_eq!(non_eof[0].kind, TokenKind::String);
}

#[test]
fn error_recovery_continues_lexing() {
    // An unterminated string at the start, followed by valid tokens
    let result = lex("@ 42");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(
        result.errors[0].kind,
        LexErrorKind::UnexpectedCharacter('@')
    );
    // The 42 should still be lexed
    let non_eof: Vec<_> = result
        .tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Eof)
        .collect();
    assert_eq!(non_eof.len(), 1);
    assert_eq!(non_eof[0].kind, TokenKind::Integer(42));
}

#[test]
fn multiple_errors() {
    let result = lex("@ ^ ~");
    assert_eq!(result.errors.len(), 3);
}

#[test]
fn error_message_display() {
    let result = lex("@");
    assert_eq!(result.errors[0].message(), "unexpected character '@'");

    let result = lex(r#""unterminated"#);
    assert_eq!(result.errors[0].message(), "unterminated string literal");
}

#[test]
fn unterminated_block_comment_error() {
    let result = lex("/* never closed");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(
        result.errors[0].kind,
        LexErrorKind::UnterminatedBlockComment
    );
}
