use crate::{
    common::span::Span,
    frontend::lexer::{TokenKind, TokenType},
};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseDiagnosticKind {
    UnexpectedToken {
        expected: &'static [TokenType],
        found: TokenKind,
    },
    NotAnExpression {
        found: TokenKind,
    },
    UnclosedLParen,
    UnclosedRParen,
    NonAssociativeChain,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseDiagnostic {
    pub kind: ParseDiagnosticKind,
    pub span: Span,
}
