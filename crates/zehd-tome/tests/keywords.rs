mod helpers;
use helpers::*;
use zehd_tome::TokenKind;

#[test]
fn all_keywords() {
    let cases = vec![
        ("let", TokenKind::Let),
        ("const", TokenKind::Const),
        ("fn", TokenKind::Fn),
        ("if", TokenKind::If),
        ("else", TokenKind::Else),
        ("match", TokenKind::Match),
        ("for", TokenKind::For),
        ("in", TokenKind::In),
        ("while", TokenKind::While),
        ("get", TokenKind::Get),
        ("post", TokenKind::Post),
        ("put", TokenKind::Put),
        ("patch", TokenKind::Patch),
        ("delete", TokenKind::Delete),
        ("init", TokenKind::Init),
        ("error", TokenKind::Error),
        ("type", TokenKind::Type),
        ("enum", TokenKind::Enum),
        ("import", TokenKind::Import),
        ("from", TokenKind::From),
        ("return", TokenKind::Return),
        ("break", TokenKind::Break),
        ("continue", TokenKind::Continue),
        ("self", TokenKind::SelfKw),
        ("true", TokenKind::True),
        ("false", TokenKind::False),
        ("None", TokenKind::None),
        ("Some", TokenKind::Some),
        ("Ok", TokenKind::Ok),
        ("Err", TokenKind::Err),
    ];

    for (text, expected) in cases {
        assert_single(text, expected);
    }
}

#[test]
fn keyword_prefixed_identifier_is_not_keyword() {
    // "letters" starts with "let" but is an identifier
    assert_single("letters", TokenKind::Identifier);
    assert_single("constant", TokenKind::Identifier);
    assert_single("iffy", TokenKind::Identifier);
    assert_single("format", TokenKind::Identifier);
    assert_single("matching", TokenKind::Identifier);
    assert_single("types", TokenKind::Identifier);
    assert_single("self_", TokenKind::Identifier);
    assert_single("import_map", TokenKind::Identifier);
    assert_single("getter", TokenKind::Identifier);
    assert_single("posted", TokenKind::Identifier);
    assert_single("deleted", TokenKind::Identifier);
    assert_single("patcher", TokenKind::Identifier);
}

#[test]
fn keywords_are_case_sensitive() {
    assert_single("Let", TokenKind::Identifier);
    assert_single("LET", TokenKind::Identifier);
    assert_single("IF", TokenKind::Identifier);
    assert_single("TRUE", TokenKind::Identifier);
    assert_single("FALSE", TokenKind::Identifier);
    // But None, Some, Ok, Err are capitalized keywords
    assert_single("None", TokenKind::None);
    assert_single("Some", TokenKind::Some);
    assert_single("Ok", TokenKind::Ok);
    assert_single("Err", TokenKind::Err);
    // Lowercase variants are identifiers
    assert_single("none", TokenKind::Identifier);
    assert_single("some", TokenKind::Identifier);
    assert_single("ok", TokenKind::Identifier);
    assert_single("err", TokenKind::Identifier);
}

#[test]
fn keyword_is_keyword_method() {
    assert!(TokenKind::Let.is_keyword());
    assert!(TokenKind::True.is_keyword());
    assert!(TokenKind::None.is_keyword());
    assert!(!TokenKind::Identifier.is_keyword());
    assert!(!TokenKind::Integer(0).is_keyword());
    assert!(!TokenKind::Plus.is_keyword());
}
