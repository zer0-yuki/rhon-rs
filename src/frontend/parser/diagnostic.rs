use crate::{common::span::Span, frontend::lexer::TokenKind};

/// The category of a parse diagnostic.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseDiagnosticKind {
    /// Expected one of these token descriptions, but found another.
    UnexpectedToken {
        expected: &'static [&'static str],
        found: TokenKind,
    },
    /// The given token kind cannot start an expression.
    NotAnExpression { found: TokenKind },
    /// A left-paren `(` was never closed.
    UnclosedLParen,
    /// A right-paren `)` appeared without a matching `(`.
    UnclosedRParen,
    /// The parser expected a binding (`name = body;`) but didn't find one.
    NotABinding,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseDiagnostic {
    pub kind: ParseDiagnosticKind,
    pub span: Span,
}
