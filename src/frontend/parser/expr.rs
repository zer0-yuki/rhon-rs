use crate::common::span::Span;

pub enum ExprKind<'ecx> {
    Number(f64),
    String(String),

    Ident(String),
    Prefix(PrefixOpKind, &'ecx Expr<'ecx>, &'ecx Expr<'ecx>),
    Infix(InfixOpKind, &'ecx Expr<'ecx>),

    App(&'ecx Expr<'ecx>, &'ecx Expr<'ecx>),
}

pub enum PrefixOpKind {
    Pos,
    Neg,
}

pub enum InfixOpKind {
    Add,
    Sub,
    Mul,
    Div,
}

pub struct Expr<'ecx> {
    kind: ExprKind<'ecx>,
    span: Span,
}

impl<'ecx> Expr<'ecx> {
    pub fn new(kind: ExprKind<'ecx>, span: Span) -> Self {
        Self { kind, span }
    }
}
