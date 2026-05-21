use crate::common::span::Span;

/// The category of a token, with associated data for literals.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Number(f64),
    String(String),

    // Identifiers
    Ident(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Dot,

    // Delimiters
    LParen,
    RParen,
    Colon,
    SemiColon,
    LBrace,
    RBrace,
    Comma,
    Equal,

    // Sentinel
    Eof,

    // Dummy
    Err,
}

impl TokenKind {
    pub fn into_token(self) -> Token {
        Token {
            kind: self,
            span: Span::DUMMY,
        }
    }

    pub fn is_eof(&self) -> bool {
        matches!(self, Self::Eof)
    }

    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err)
    }
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

    pub const EOF: Token = Token {
        kind: TokenKind::Eof,
        span: Span::DUMMY,
    };

    pub const ERR: Token = Token {
        kind: TokenKind::Err,
        span: Span::DUMMY,
    };
}
