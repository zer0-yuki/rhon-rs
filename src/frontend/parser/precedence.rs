use crate::frontend::{lexer::TokenKind, parser::associativity::Associativity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BindingPower(u8);

impl BindingPower {
    pub fn prefix() -> Self {
        (Precedence::Prefix as u8).into()
    }

    pub fn lowest() -> Self {
        (Precedence::Lowest as u8).into()
    }
}

impl From<u8> for BindingPower {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

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
    /// Only used by [`Precedence::prefix_bp`].
    Prefix     = 100,
    Exponent   = 110,
    Call       = 120,
    Member     = 130,
}

impl Precedence {
    pub fn try_from_infix(kind: &TokenKind) -> Option<Self> {
        use Precedence::*;

        Some(match kind {
            // Operators that bind as infix operators.
            TokenKind::Plus | TokenKind::Minus => Sum,
            TokenKind::Star | TokenKind::Slash => Product,
            TokenKind::Dot => Member,
            // Function application (juxtaposition).
            TokenKind::Number(_)
            | TokenKind::String(_)
            | TokenKind::Ident(_)
            | TokenKind::LParen => Call,
            // Everything else is not an infix operator.
            _ => return None,
        })
    }

    pub fn associativity(&self) -> Associativity {
        use Precedence::*;

        match self {
            Lowest | Equality | Relational => Associativity::None,
            Exponent => Associativity::Right,
            _ => Associativity::Left,
        }
    }

    pub fn to_infix_bp(&self) -> (BindingPower, BindingPower) {
        use Associativity::*;

        let bp = *self as u8;
        let (lbp, rbp) = match self.associativity() {
            Left => (bp, bp + 1),
            Right => (bp + 1, bp),
            None => (bp, bp + 1),
        };
        (lbp.into(), rbp.into())
    }
}

// /// Return the infix `(left, right)` binding power with [`Associativity`] of a token.
// ///
// /// Returns [`None`] when the token cannot appear as an infix at all.
// pub fn infix_bp_assoc(kind: &TokenKind) -> Option<(BindingPower, BindingPower, Associativity)> {
//     Precedence::try_from_infix(kind).map(|p| {
//         let (lbp, rbp) = p.to_infix_bp();
//         let assoc = p.associativity();
//         (lbp, rbp, assoc)
//     })
// }
