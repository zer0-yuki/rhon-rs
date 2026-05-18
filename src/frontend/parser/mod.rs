use crate::frontend::{
    lexer::{Lexer, Token},
    parser::diagnostic::ParseDiagnostic,
};

mod diagnostic;
mod expr;

pub struct Parser<'src> {
    lexer: Lexer<'src>,
    diagnostics: Vec<ParseDiagnostic>,
}

impl<'src> Parser<'src> {
    pub fn new(lexer: Lexer<'src>) -> Self {
        Self {
            lexer,
            diagnostics: vec![],
        }
    }

    pub fn diagnostics(&self) -> &[ParseDiagnostic] {
        &self.diagnostics
    }

    // raw method ----------------

    fn eat(&mut self) -> Token {
        self.lexer.advance()
    }

    fn peek(&self) -> &Token {
        self.lexer.cur()
    }
}
