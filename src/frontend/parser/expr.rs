use std::fmt;

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

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Number(f64),
    String(String),

    Var(String),

    Prefix(PrefixOp, Box<Expr>),
    Infix(InfixOp, Box<Expr>, Box<Expr>),

    App(Box<Expr>, Box<Expr>),

    Err,
}

impl ExprKind {
    pub fn into_expr(self) -> Expr {
        Expr {
            kind: self,
            span: Span::DUMMY,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} @ {:?}", self.kind, self.span)
    }
}

impl Expr {
    pub const ERR: Expr = Expr {
        kind: ExprKind::Err,
        span: Span::DUMMY,
    };

    pub fn new(kind: ExprKind, span: Span) -> Self {
        Self { kind, span }
    }
}
