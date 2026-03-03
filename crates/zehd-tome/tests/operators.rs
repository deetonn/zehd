mod helpers;
use helpers::*;
use zehd_tome::TokenKind;

#[test]
fn single_char_operators() {
    assert_single("+", TokenKind::Plus);
    assert_single("-", TokenKind::Minus);
    assert_single("*", TokenKind::Star);
    assert_single("/", TokenKind::Slash);
    assert_single("%", TokenKind::Percent);
    assert_single("!", TokenKind::Bang);
    assert_single("=", TokenKind::Eq);
    assert_single("?", TokenKind::Question);
    assert_single(".", TokenKind::Dot);
    assert_single("<", TokenKind::Lt);
    assert_single(">", TokenKind::Gt);
}

#[test]
fn two_char_operators() {
    assert_single("==", TokenKind::EqEq);
    assert_single("!=", TokenKind::BangEq);
    assert_single("<=", TokenKind::LtEq);
    assert_single(">=", TokenKind::GtEq);
    assert_single("&&", TokenKind::AmpAmp);
    assert_single("||", TokenKind::PipePipe);
    assert_single("=>", TokenKind::FatArrow);
    assert_single("::", TokenKind::ColonColon);
}

#[test]
fn disambiguation_eq_eqeq_fatarrow() {
    let k = kinds("= == =>");
    assert_eq!(k, vec![TokenKind::Eq, TokenKind::EqEq, TokenKind::FatArrow]);
}

#[test]
fn disambiguation_lt_lteq() {
    let k = kinds("< <=");
    assert_eq!(k, vec![TokenKind::Lt, TokenKind::LtEq]);
}

#[test]
fn disambiguation_gt_gteq() {
    let k = kinds("> >=");
    assert_eq!(k, vec![TokenKind::Gt, TokenKind::GtEq]);
}

#[test]
fn disambiguation_bang_bangeq() {
    let k = kinds("! !=");
    assert_eq!(k, vec![TokenKind::Bang, TokenKind::BangEq]);
}

#[test]
fn disambiguation_colon_coloncolon() {
    let k = kinds(": ::");
    assert_eq!(k, vec![TokenKind::Colon, TokenKind::ColonColon]);
}

#[test]
fn dot_vs_dotdotdot() {
    let k = kinds(". ...");
    assert_eq!(k, vec![TokenKind::Dot, TokenKind::DotDotDot]);
}

#[test]
fn operators_without_spaces() {
    let k = kinds("a+b");
    assert_eq!(
        k,
        vec![TokenKind::Identifier, TokenKind::Plus, TokenKind::Identifier]
    );
}

#[test]
fn all_delimiters() {
    assert_single("{", TokenKind::LeftBrace);
    assert_single("}", TokenKind::RightBrace);
    assert_single("(", TokenKind::LeftParen);
    assert_single(")", TokenKind::RightParen);
    assert_single("[", TokenKind::LeftBracket);
    assert_single("]", TokenKind::RightBracket);
    assert_single(";", TokenKind::Semicolon);
    assert_single(",", TokenKind::Comma);
    assert_single(":", TokenKind::Colon);
}

#[test]
fn special_tokens() {
    assert_single("#", TokenKind::Hash);
    assert_single("...", TokenKind::DotDotDot);
}
