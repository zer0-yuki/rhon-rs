use crate::common::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseDiagnosticKind {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseDiagnostic {
    kind: ParseDiagnosticKind,
    span: Span,
}
