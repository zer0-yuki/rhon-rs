use crate::frontend::lexer::Token;

pub trait TokenStream {
    fn advance(&mut self) -> Token;
    fn current(&self) -> &Token;
}

pub struct SliceStream<'tok> {
    tokens: &'tok [Token],
    pos: usize,
}

impl<'tok> SliceStream<'tok> {
    pub fn new(tokens: &'tok [Token]) -> Self {
        Self { tokens, pos: 0 }
    }
}

impl<'tok> TokenStream for SliceStream<'tok> {
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
