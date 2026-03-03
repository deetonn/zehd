mod helpers;
use helpers::*;
use zehd_tome::{lex, LexErrorKind, TokenKind};

#[test]
fn line_comment_skipped() {
    let k = kinds("// this is a comment");
    assert!(k.is_empty());
}

#[test]
fn line_comment_before_code() {
    let k = kinds("// comment\n42");
    assert_eq!(k, vec![TokenKind::Integer(42)]);
}

#[test]
fn line_comment_after_code() {
    let k = kinds("42 // comment");
    assert_eq!(k, vec![TokenKind::Integer(42)]);
}

#[test]
fn multiple_line_comments() {
    let k = kinds("// first\n// second\n42");
    assert_eq!(k, vec![TokenKind::Integer(42)]);
}

#[test]
fn block_comment_skipped() {
    let k = kinds("/* block */");
    assert!(k.is_empty());
}

#[test]
fn block_comment_inline() {
    let k = kinds("42 /* middle */ 43");
    assert_eq!(k, vec![TokenKind::Integer(42), TokenKind::Integer(43)]);
}

#[test]
fn block_comment_multiline() {
    let k = kinds("42 /* \n multi \n line \n */ 43");
    assert_eq!(k, vec![TokenKind::Integer(42), TokenKind::Integer(43)]);
}

#[test]
fn unterminated_block_comment() {
    let result = lex("/* unterminated");
    assert_eq!(result.errors.len(), 1);
    assert_eq!(
        result.errors[0].kind,
        LexErrorKind::UnterminatedBlockComment
    );
}

#[test]
fn slash_is_not_comment() {
    let k = kinds("a / b");
    assert_eq!(
        k,
        vec![TokenKind::Identifier, TokenKind::Slash, TokenKind::Identifier]
    );
}

#[test]
fn empty_block_comment() {
    let k = kinds("/**/");
    assert!(k.is_empty());
}

#[test]
fn block_comment_with_stars() {
    let k = kinds("/*** stars ***/");
    assert!(k.is_empty());
}
