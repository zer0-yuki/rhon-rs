use crate::frontend::lexer::Token;

pub trait TokenStream {
    fn advance(&mut self) -> Token;
    fn current(&self) -> &Token;
}
