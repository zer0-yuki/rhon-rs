use crate::frontend::lexer::Token;

pub trait TokenStream {
    fn advance(&mut self) -> Token;
    fn current(&self) -> &Token;
}

pub struct SliceStream {
    tokens: Vec<Token>,
}

impl SliceStream {
    pub fn new<T>(tokens: T) -> Self
    where
        T: DoubleEndedIterator<Item = Token>,
    {
        Self {
            tokens: tokens.rev().collect(),
        }
    }
}

impl TokenStream for SliceStream {
    fn advance(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token::EOF)
    }

    fn current(&self) -> &Token {
        &self.tokens.last().unwrap_or(&Token::EOF)
    }
}
