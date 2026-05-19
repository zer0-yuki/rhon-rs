//! Binding powers for Pratt parsing.
//!
//! Higher values bind tighter.  The [`Precedence`] enum encodes ordering
//! directly in the type system — the Pratt loop compares precedence
//! levels with `>=`, backed by the derived `Ord`.

use crate::frontend::lexer::TokenKind;

/// Operator precedence, from loosest to tightest.
///
/// The discriminants match the TypeScript reference implementation so
/// numeric values are predictable, but all comparisons use the derived
/// `Ord` (which follows declaration order, i.e. discriminant order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[allow(dead_code)] // reserved for future operators
pub enum Precedence {
    Lowest = 0,
    Assignment = 10,
    LogicalOr = 20,
    LogicalAnd = 30,
    BitwiseOr = 40,
    BitwiseXor = 50,
    BitwiseAnd = 60,
    Equality = 70,
    Relational = 80,
    Sum = 90,
    Product = 100,
    Prefix = 110,
    Call = 120,
    Member = 130,
}

/// Return the **left** binding power of a token when it acts as an infix
/// operator.
///
/// Returns `None` when the token cannot appear as an infix at all.
///
/// The Pratt loop uses this for the "should I break?" check:
/// `min_bp >= left_bp  ⇒  break`.
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
/// For left-associative operators (all current ones), this equals
/// `left_bp`.  For right-associative operators (e.g. `^`), this would
/// be one level *lower* than `left_bp` so that `a ^ b ^ c` groups as
/// `a ^ (b ^ c)`.
#[inline]
pub(crate) fn infix_right_bp(kind: &TokenKind) -> Precedence {
    infix_left_bp(kind).unwrap_or(Precedence::Lowest)
}
