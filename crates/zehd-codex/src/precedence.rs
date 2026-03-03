use zehd_tome::TokenKind;

use crate::ast::BinaryOp;

/// Returns (left_bp, right_bp) for infix operators.
/// Left < right = left-associative.
/// Returns None for non-infix tokens.
pub(crate) fn infix_binding_power(kind: &TokenKind) -> Option<(u8, u8)> {
    match kind {
        TokenKind::PipePipe => Some((1, 2)),
        TokenKind::AmpAmp => Some((3, 4)),
        TokenKind::EqEq | TokenKind::BangEq => Some((5, 6)),
        TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq => Some((7, 8)),
        TokenKind::Plus | TokenKind::Minus => Some((9, 10)),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((11, 12)),
        _ => None,
    }
}

/// Returns right_bp for prefix (unary) operators.
pub(crate) fn prefix_binding_power(kind: &TokenKind) -> Option<u8> {
    match kind {
        TokenKind::Bang | TokenKind::Minus => Some(13),
        _ => None,
    }
}

/// Returns left_bp for postfix operators.
/// Postfix has the highest precedence.
pub(crate) fn postfix_binding_power(kind: &TokenKind) -> Option<u8> {
    match kind {
        TokenKind::Question | TokenKind::Dot | TokenKind::LeftBracket | TokenKind::LeftParen => {
            Some(15)
        }
        _ => None,
    }
}

/// Maps a token kind to a BinaryOp. Returns None for non-binary-op tokens.
pub(crate) fn token_to_binary_op(kind: &TokenKind) -> Option<BinaryOp> {
    match kind {
        TokenKind::Plus => Some(BinaryOp::Add),
        TokenKind::Minus => Some(BinaryOp::Sub),
        TokenKind::Star => Some(BinaryOp::Mul),
        TokenKind::Slash => Some(BinaryOp::Div),
        TokenKind::Percent => Some(BinaryOp::Mod),
        TokenKind::EqEq => Some(BinaryOp::Eq),
        TokenKind::BangEq => Some(BinaryOp::NotEq),
        TokenKind::Lt => Some(BinaryOp::Lt),
        TokenKind::Gt => Some(BinaryOp::Gt),
        TokenKind::LtEq => Some(BinaryOp::LtEq),
        TokenKind::GtEq => Some(BinaryOp::GtEq),
        TokenKind::AmpAmp => Some(BinaryOp::And),
        TokenKind::PipePipe => Some(BinaryOp::Or),
        _ => None,
    }
}
