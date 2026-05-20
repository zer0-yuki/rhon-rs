use crate::frontend::lexer::TokenKind;

/// Operator precedence, from loosest to tightest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[rustfmt::skip]
pub enum Precedence {
    /// Dummy, not used for returns.
    Lowest     = 0,
    LogicalOr  = 10,
    LogicalAnd = 20,
    BitwiseOr  = 30,
    BitwiseXor = 40,
    BitwiseAnd = 50,
    Equality   = 60,
    Relational = 70,
    Sum        = 80,
    Product    = 90,
    Prefix     = 100,
    Exponent   = 110,
    Call       = 120,
    Member     = 130,
}

/// Return the **left** binding power of a token when it acts as an infix
/// operator.
///
/// Returns `None` when the token cannot appear as an infix at all.
pub(crate) fn infix_left_bp(kind: &TokenKind) -> Option<Precedence> {
    use Precedence::*;

    Some(match kind {
        // Operators that bind as infix operators.
        TokenKind::Plus | TokenKind::Minus => Sum,
        TokenKind::Star | TokenKind::Slash => Product,
        // Function application (juxtaposition).
        TokenKind::Number(_) | TokenKind::String(_) | TokenKind::Ident(_) | TokenKind::LParen => {
            Call
        }
        // Everything else is not an infix operator.
        _ => return None,
    })
}

/// Return the **right** binding power of a token — the precedence level
/// passed to the recursive `parse_expr_bp` call after consuming this
/// infix operator.
///
/// For left-associative operators, this equals `left_bp`.
/// For right-associative operators, this would be one level *lower* than `left_bp`.
#[inline]
pub(crate) fn infix_right_bp(kind: &TokenKind) -> Precedence {
    infix_left_bp(kind).unwrap_or(Precedence::Lowest)
}
