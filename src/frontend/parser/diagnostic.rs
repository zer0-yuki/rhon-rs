use crate::{common::span::Span, frontend::lexer::TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseDiagnosticKind {
    UnexpectedToken {
        expected: &'static [&'static str],
        found: TokenKind,
    },
    NotAnExpression {
        found: TokenKind,
    },
    UnclosedLParen,
    UnclosedRParen,
    NotABinding,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseDiagnostic {
    pub kind: ParseDiagnosticKind,
    pub span: Span,
}
