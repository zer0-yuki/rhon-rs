use crate::common::span::Span;

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

pub type ExprPtr<'arena, 'ecx> = &'arena Expr<'ecx>;

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind<'arena> {
    Number(f64),
    String(String),

    Var(String),

    Prefix(PrefixOp, ExprPtr<'arena, 'arena>),
    Infix(InfixOp, ExprPtr<'arena, 'arena>, ExprPtr<'arena, 'arena>),

    App(ExprPtr<'arena, 'arena>, ExprPtr<'arena, 'arena>),

    Unknown,
    Err,
}

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
