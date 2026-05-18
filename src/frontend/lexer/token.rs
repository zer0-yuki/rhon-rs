use crate::common::span::Span;

/// The category of a token, with associated data for literals.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Number(f64),
    String(String),

    // Identifiers / keywords
    Ident(String),

    // Symbols
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Equal,
    Colon,
    SemiColon,
    LBrace,
    RBrace,
    Comma,
    Dot,

    // Sentinel
    Eof,

    // Dummy
    Err,
}

/// A lexical token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn eof() -> Self {
        Self {
            kind: TokenKind::Eof,
            span: Default::default(),
        }
    }

    pub fn err() -> Self {
        Self {
            kind: TokenKind::Err,
            span: Default::default(),
        }
    }
}
