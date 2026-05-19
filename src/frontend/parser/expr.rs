use crate::common::{
    arena::{ArenaPtrMut, ArenaPtr},
    span::Span,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixOp {
    Pos,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfixOp {
    Add,
    Sub,
    Mul,
    Div,
}

pub type ExprPtr<'arena, 'ecx> = ArenaPtr<'arena, Expr<'ecx>>;
pub type ExprPtrMut<'arena, 'ecx> = ArenaPtrMut<'arena, Expr<'ecx>>;

// ── Expr / ExprKind ───────────────────────────────────────────────────────

/// The kind of an expression node.
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind<'arena> {
    /// Numeric literal.
    Number(f64),
    /// String literal.
    String(String),
    /// Variable reference.
    Var(String),
    /// Prefix unary expression (e.g. `-x`, `+x`).
    Prefix(PrefixOp, ExprPtr<'arena, 'arena>),
    /// Infix binary expression (e.g. `a + b`).
    Infix(InfixOp, ExprPtr<'arena, 'arena>, ExprPtr<'arena, 'arena>),
    /// Function application (e.g. `f x`).
    App(ExprPtr<'arena, 'arena>, ExprPtr<'arena, 'arena>),
    /// Placeholder for incomplete parse.
    Unknown,
    /// Sentinel produced during error recovery.
    Err,
}

/// An arena-allocated expression node.
///
/// Child nodes are held as [`ExprPtr`] handles, which remain valid as
/// long as the owning [`Arena`](crate::common::arena::Arena) is alive.
#[derive(Debug, Clone, PartialEq)]
pub struct Expr<'arena> {
    pub kind: ExprKind<'arena>,
    pub span: Span,
}

impl<'arena> Expr<'arena> {
    pub fn new(kind: ExprKind<'arena>, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn unknown() -> Self {
        Self {
            kind: ExprKind::Unknown,
            span: Default::default(),
        }
    }

    pub fn err() -> Self {
        Self {
            kind: ExprKind::Err,
            span: Default::default(),
        }
    }
}
