use crate::frontend::lexer::Token;

pub trait TokenStream {
    fn advance(&mut self) -> Token;
    fn current(&self) -> &Token;
}

pub struct SliceStream {
    tokens: Vec<Token>,
    pos: usize,
}

impl SliceStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }
}

impl TokenStream for SliceStream {
    fn advance(&mut self) -> Token {
        let cur = self.current().clone(); // TODO: clone is not good
        self.pos += 1;
        cur
    }

    fn current(&self) -> &Token {
        if self.pos >= self.tokens.len() {
            &Token::EOF
        } else {
            &self.tokens[self.pos]
        }
    }
}
