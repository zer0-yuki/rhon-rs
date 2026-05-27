mod diagnostic;
mod token;

pub use diagnostic::LexDiagnostic;
pub use token::{Token, TokenKind, TokenType};

use crate::{
    common::span::Span,
    frontend::{lexer::diagnostic::LexDiagnosticKind, parser::token_stream::TokenStream},
};

pub struct Lexer<'src> {
    source: &'src str,
    /// Current line number (1-based).
    linebreaks: Vec<usize>,
    /// Current byte position in [`source`](Self::source).
    pos: usize,
    /// Whether the previous token was preceded by whitespace.
    /// Used to disambiguate `+`/`-` as sign vs operator.
    prev_is_whitespace: bool,

    current: Token,
    next: Token,

    diagnostics: Vec<LexDiagnostic>,
}

impl<'src> TokenStream for Lexer<'src> {
    fn current(&self) -> &Token {
        &self.current
    }

    /// Advance to the next token, returning the *previous* current token.
    fn advance(&mut self) -> Token {
        let next = self.next_token();
        let prev_next = std::mem::replace(&mut self.next, next);
        let prev_cur = std::mem::replace(&mut self.current, prev_next);
        prev_cur
    }
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        let mut lexer = Self {
            source,
            pos: 0,
            linebreaks: vec![],
            prev_is_whitespace: false,
            current: Token::EOF,
            next: Token::EOF,
            diagnostics: Vec::new(),
        };
        lexer.current = lexer.next_token();
        lexer.next = lexer.next_token();
        lexer
    }

    pub fn next(&self) -> &Token {
        &self.next
    }

    pub fn diagnostics(&self) -> &[LexDiagnostic] {
        &self.diagnostics
    }

    pub fn report(&mut self, kind: LexDiagnosticKind, start: usize) {
        self.diagnostics.push(LexDiagnostic {
            kind,
            span: self.make_span(start),
        })
    }

    fn peek_char(&self) -> char {
        self.source
            .as_bytes() // read as u8 array
            .get(self.pos) // index safely
            .copied() // convert optional addr to optional u8
            .unwrap_or(0) // default to null char
            as char
    }

    fn advance_char(&mut self) -> char {
        let c = self.peek_char();
        if c != '\0' {
            self.pos += 1;
        }
        c
    }

    fn tag_linebreak(&mut self) {
        self.linebreaks.push(self.pos)
    }

    fn skip_whitespace(&mut self) {
        self.prev_is_whitespace = false;
        loop {
            match self.peek_char() {
                '\n' => {
                    self.prev_is_whitespace = true;
                    self.tag_linebreak();
                    self.advance_char();
                }
                ' ' | '\t' | '\r' => {
                    self.prev_is_whitespace = true;
                    self.advance_char();
                }
                _ => break,
            }
        }
    }

    fn make_span(&self, start: usize) -> Span {
        Span::new(start, self.pos)
    }

    fn read_number(&mut self, start: usize) -> Token {
        while self.peek_char().is_ascii_digit() {
            self.advance_char();
        }
        // TODO: support float (dot + digits)
        let lexeme = self.slice_from(start);
        let value: f64 = lexeme.parse().unwrap_or(0.0);
        Token::new(TokenKind::Number(value), self.make_span(start))
    }

    fn read_ident(&mut self, start: usize) -> Token {
        while self.peek_char().is_ascii_alphanumeric() || self.peek_char() == '_' {
            self.advance_char();
        }
        let lexeme = self.slice_from(start);
        Token::new(TokenKind::Ident(lexeme.to_owned()), self.make_span(start))
    }

    fn read_string(&mut self, start: usize) -> Token {
        loop {
            match self.peek_char() {
                '"' => {
                    self.advance_char(); // consume closing quote
                    let span = self.slice_from(start);
                    // Strip surrounding quotes for the value
                    let inner = span[1..span.len() - 1].to_owned();
                    return Token::new(TokenKind::String(inner), self.make_span(start));
                }
                '\0' => {
                    self.report(LexDiagnosticKind::UnclosedStringLiteral, start);
                    return Token::new(TokenKind::String(String::new()), self.make_span(start));
                }
                '\n' => {
                    self.tag_linebreak();
                    self.advance_char();
                }
                _ => {
                    self.advance_char();
                }
            }
        }
    }

    fn slice_from(&self, start: usize) -> &'src str {
        &self.source[start..self.pos]
    }

    /// Produce the next token.
    ///
    /// The lexing logic is right here.
    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let start = self.pos;
        let c = self.advance_char();

        match c {
            '\0' => Token::EOF,

            '+' => {
                if self.peek_char().is_ascii_digit() && (self.prev_is_whitespace || start == 0) {
                    // Positive number literal: + is part of the number
                    self.read_number(start)
                } else {
                    Token::new(TokenKind::Plus, self.make_span(start))
                }
            }
            '-' => {
                if self.peek_char().is_ascii_digit() && (self.prev_is_whitespace || start == 0) {
                    // Negative number literal: - is part of the number
                    self.read_number(start)
                } else {
                    Token::new(TokenKind::Minus, self.make_span(start))
                }
            }
            '*' => Token::new(TokenKind::Star, self.make_span(start)),
            '/' => Token::new(TokenKind::Slash, self.make_span(start)),
            '(' => Token::new(TokenKind::LParen, self.make_span(start)),
            ')' => Token::new(TokenKind::RParen, self.make_span(start)),
            '=' => Token::new(TokenKind::Equal, self.make_span(start)),
            ':' => Token::new(TokenKind::Colon, self.make_span(start)),
            ';' => Token::new(TokenKind::SemiColon, self.make_span(start)),
            '{' => Token::new(TokenKind::LBrace, self.make_span(start)),
            '}' => Token::new(TokenKind::RBrace, self.make_span(start)),
            ',' => Token::new(TokenKind::Comma, self.make_span(start)),
            '.' => Token::new(TokenKind::Dot, self.make_span(start)),

            '"' => self.read_string(start),

            c if c.is_ascii_digit() => self.read_number(start),

            c if c.is_ascii_alphabetic() || c == '_' => self.read_ident(start),

            _ => {
                self.report(LexDiagnosticKind::UnknownChar, start);
                // Error recovery: skip the unknown char and produce a dummy token
                Token::ERR
            }
        }
    }
}

/// Convenience: iterate over tokens (excluding EOF).
impl<'src> Iterator for Lexer<'src> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.advance();
        if token.kind == TokenKind::Eof {
            None
        } else {
            Some(token)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::assert::len_eq::assert_len_eq;

    use super::*;

    /// Collect all tokens (excluding EOF) from source.
    fn lex_all(src: &str) -> Vec<Token> {
        Lexer::new(src).collect()
    }

    fn assert_token_eq(actual: &[Token], expected: &[TokenKind]) {
        assert_len_eq(actual, expected);
        for (a, e) in actual.iter().zip(expected) {
            assert_eq!(&a.kind, e);
        }
    }

    macro_rules! test_expected {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                assert_token_eq(&lex_all($input), &$expected)
            }
        };
    }

    mod single_operators {
        use super::*;

        macro_rules! test_operator {
            ($name:ident, $input:expr, $kind:expr) => {
                test_expected!($name, $input, [$kind]);
            };
        }

        test_operator!(lexes_plus, "+", TokenKind::Plus);
        test_operator!(lexes_minus, "-", TokenKind::Minus);
        test_operator!(lexes_star, "*", TokenKind::Star);
        test_operator!(lexes_slash, "/", TokenKind::Slash);
        test_operator!(lexes_lparen, "(", TokenKind::LParen);
        test_operator!(lexes_rparen, ")", TokenKind::RParen);
        test_operator!(lexes_equal, "=", TokenKind::Equal);
        test_operator!(lexes_colon, ":", TokenKind::Colon);
        test_operator!(lexes_semicolon, ";", TokenKind::SemiColon);
        test_operator!(lexes_lbrace, "{", TokenKind::LBrace);
        test_operator!(lexes_rbrace, "}", TokenKind::RBrace);
        test_operator!(lexes_comma, ",", TokenKind::Comma);
        test_operator!(lexes_dot, ".", TokenKind::Dot);
    }

    mod number_literals {
        use super::*;

        test_expected!(lexes_unsigned_number, "42", [TokenKind::Number(42.0)]);
        test_expected!(lexes_positive_number, "+42", [TokenKind::Number(42.0)]);
        test_expected!(lexes_negative_number, "-42", [TokenKind::Number(-42.0)]);

        mod after_whitespace {
            use super::*;

            test_expected!(lexes_positive_number, " +42", [TokenKind::Number(42.0)]);
            test_expected!(lexes_negative_number, " -42", [TokenKind::Number(-42.0)]);
        }
    }

    mod string_literals {
        use super::*;

        test_expected!(
            lexes_plain_string,
            r#""hello""#,
            [TokenKind::String("hello".into())]
        );
        test_expected!(lexes_empty_string, r#""""#, [TokenKind::String("".into())]);
        test_expected!(
            lexes_string_with_spaces,
            r#""a b c""#,
            [TokenKind::String("a b c".into())]
        );
        test_expected!(
            lexes_string_with_operators,
            r#""a=1+2""#,
            [TokenKind::String("a=1+2".into())]
        );

        #[test]
        fn reports_unclosed_string() {
            let mut lexer = Lexer::new(r#""hello"#);
            let _tokens: Vec<_> = lexer.by_ref().collect();
            assert_eq!(lexer.diagnostics().len(), 1);
            assert!(matches!(
                lexer.diagnostics()[0].kind,
                LexDiagnosticKind::UnclosedStringLiteral
            ));
        }
    }

    mod identifiers {
        use super::*;

        macro_rules! test_identifier {
            ($name:ident, $input:expr) => {
                test_expected!($name, $input, [TokenKind::Ident($input.into())]);
            };
        }

        test_identifier!(lexes_plain_ident, "hello");
        test_identifier!(lexes_ident_with_underscore, "_hello");
        test_identifier!(lexes_ident_with_digits, "he11o");
    }

    mod errors {
        use super::*;

        #[test]
        fn reports_unknown_char() {
            let mut lexer = Lexer::new("#");
            let tokens: Vec<_> = lexer.by_ref().collect();
            assert_eq!(tokens, vec![Token::ERR]);
            assert_eq!(lexer.diagnostics().len(), 1);
            assert!(matches!(
                lexer.diagnostics()[0].kind,
                LexDiagnosticKind::UnknownChar
            ));
        }
    }
}
