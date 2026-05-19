use crate::common::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexDiagnosticKind {
    UnclosedStringLiteral,
    UnknownChar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexDiagnostic {
    pub kind: LexDiagnosticKind,
    pub span: Span,
}
