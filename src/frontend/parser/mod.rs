use crate::{
    common::span::Span,
    frontend::{
        lexer::{Token, TokenKind},
        parser::{
            associativity::Associativity,
            diagnostic::{ParseDiagnostic, ParseDiagnosticKind},
            expr::{Expr, ExprKind, InfixOp, PrefixOp},
            precedence::{BindingPower, Precedence},
            token_stream::TokenStream,
        },
    },
};

pub mod associativity;
mod diagnostic;
mod expr;
pub mod precedence;
pub mod token_stream;

#[derive(Debug, Clone)]
pub struct Supercombinator {
    pub name: String,
    pub args: Vec<String>,
    pub body: Box<Expr>,
}

pub struct Parser<'src, T>
where
    T: TokenStream,
{
    tokens: &'src mut T,
    diagnostics: Vec<ParseDiagnostic>,
}

impl<'src, T> Parser<'src, T>
where
    T: TokenStream,
{
    pub fn new(tokens: &'src mut T) -> Self {
        Self {
            tokens,
            diagnostics: Vec::new(),
        }
    }

    // ---- diagnostics ----

    pub fn diagnostics(&self) -> &[ParseDiagnostic] {
        &self.diagnostics
    }

    fn report(&mut self, kind: ParseDiagnosticKind, span: Span) {
        self.diagnostics.push(ParseDiagnostic { kind, span });
    }

    // ---- lexer ----

    fn eat(&mut self) -> Token {
        self.tokens.advance()
    }

    /// Peek current token with no cost.
    fn peek(&self) -> &Token {
        self.tokens.current()
    }

    // ---- parsing ----

    pub fn parse(&mut self) -> Vec<Supercombinator> {
        let mut scs = Vec::new();
        while !self.peek().kind.is_eof() {
            if let Some(sc) = self.parse_sc() {
                scs.push(sc);
            }
        }
        scs
    }

    fn parse_sc(&mut self) -> Option<Supercombinator> {
        let mut is_ident_first = true;

        // Parse an ident as name
        let name = loop {
            let cur = self.eat();
            match cur.kind {
                TokenKind::Eof | TokenKind::SemiColon => {
                    self.report(ParseDiagnosticKind::NotABinding, cur.span);
                    return None;
                }
                TokenKind::Ident(name) => {
                    // only if meet an ident we can normally parse an SC
                    break name;
                }
                _ => {
                    is_ident_first = false;
                }
            }
        };

        if !is_ident_first {
            self.report(ParseDiagnosticKind::NotABinding, Default::default());
        }

        // Parse args and equal token
        let mut args: Vec<String> = Vec::new();
        loop {
            let cur = self.eat();
            match cur.kind {
                TokenKind::Eof | TokenKind::SemiColon => {
                    self.report(
                        ParseDiagnosticKind::UnexpectedToken {
                            expected: &["ident", "equal"],
                            found: cur.kind,
                        },
                        cur.span,
                    );
                    return Some(Supercombinator {
                        name,
                        args,
                        body: Box::new(Expr::ERR),
                    });
                }
                TokenKind::Equal => {
                    break;
                }
                TokenKind::Ident(name) => {
                    args.push(name);
                }
                _ => {
                    self.report(
                        ParseDiagnosticKind::UnexpectedToken {
                            expected: &["ident", "equal"],
                            found: cur.kind,
                        },
                        cur.span,
                    );
                }
            }
        }

        let body = self.parse_expr();

        // Finally expect a semicolon
        let mut is_semicolon = true;
        let cur = loop {
            let cur = self.eat();
            match &cur.kind {
                TokenKind::Eof => {
                    self.report(
                        ParseDiagnosticKind::UnexpectedToken {
                            expected: &["semicolon"],
                            found: cur.kind,
                        },
                        cur.span,
                    );
                    return Some(Supercombinator { name, args, body });
                }
                TokenKind::SemiColon => break cur,
                _ => {
                    is_semicolon = false;
                    self.eat();
                }
            };
        };
        if !is_semicolon {
            self.report(
                ParseDiagnosticKind::UnexpectedToken {
                    expected: &["semicolon"],
                    found: cur.kind,
                },
                cur.span,
            );
        }

        Some(Supercombinator { name, args, body })
    }

    fn parse_expr(&mut self) -> Box<Expr> {
        self.parse_expr_bp(BindingPower::lowest())
    }

    /// Core Pratt parser — parse an expression with the given minimum binding power.
    ///
    /// Returns an [`ExprPtr`] handle into the arena.
    /// The handle is valid as long as the parser is alive.
    fn parse_expr_bp(&mut self, min_bp: BindingPower) -> Box<Expr> {
        let cur = self.eat();
        let mut left = self.parse_prefix(cur);
        let mut seen_non_assoc = false;

        loop {
            let precedence = Precedence::try_from_infix(&self.peek().kind);
            let precedence = match precedence {
                Some(p) => p,
                None => break, // Because it is not an operator
            };

            // Non-associative operators (e.g. ==, <, >) cannot be chained
            // without explicit grouping.
            if seen_non_assoc && matches!(precedence.associativity(), Associativity::None) {
                self.report(ParseDiagnosticKind::NonAssociativeChain, self.peek().span);
                // Error recovery: stop at this level to avoid cascading errors.
                break;
            }

            let (lbp, _) = precedence.to_infix_bp();
            if min_bp >= lbp {
                break;
            }

            if matches!(precedence, Precedence::Call) {
                left = self.parse_app(left);
            } else {
                left = self.parse_infix(left);
            }

            if matches!(precedence.associativity(), Associativity::None) {
                seen_non_assoc = true;
            }
        }

        left
    }

    fn parse_app(&mut self, left: Box<Expr>) -> Box<Expr> {
        let (_, rbp) = Precedence::Call.to_infix_bp();
        let right = self.parse_expr_bp(rbp);
        let span = left.span.merge(right.span);
        Box::new(Expr::new(ExprKind::App(left, right), span))
    }

    /// Parse a token as the **start** of an expression.
    fn parse_prefix(&mut self, token: Token) -> Box<Expr> {
        match token.kind {
            TokenKind::Number(n) => Box::new(Expr::new(ExprKind::Number(n), token.span)),

            TokenKind::String(s) => Box::new(Expr::new(ExprKind::String(s), token.span)),

            TokenKind::Ident(name) => Box::new(Expr::new(ExprKind::Var(name), token.span)),

            TokenKind::Plus => {
                let right = self.parse_expr_bp(BindingPower::prefix());
                let span = token.span.merge(right.span);
                Box::new(Expr::new(ExprKind::Prefix(PrefixOp::Pos, right), span))
            }

            TokenKind::Minus => {
                let right = self.parse_expr_bp(BindingPower::prefix());
                let span = token.span.merge(right.span);
                Box::new(Expr::new(ExprKind::Prefix(PrefixOp::Neg, right), span))
            }

            TokenKind::LParen => {
                let inner = self.parse_expr_bp(BindingPower::lowest());
                let next = self.eat();
                if next.kind != TokenKind::RParen {
                    self.report(ParseDiagnosticKind::UnclosedLParen, token.span);
                }
                inner
            }

            TokenKind::RParen => {
                self.report(ParseDiagnosticKind::UnclosedRParen, token.span);
                Box::new(Expr::ERR)
            }

            _ => {
                self.report(
                    ParseDiagnosticKind::NotAnExpression { found: token.kind },
                    token.span,
                );
                Box::new(Expr::ERR)
            }
        }
    }

    /// Parse an infix (left denotation) operator and its right-hand side.
    fn parse_infix(&mut self, left: Box<Expr>) -> Box<Expr> {
        // Copy everything we need from peek() before any mutable call.
        let op_tok = self.eat();

        match &op_tok.kind {
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                let op = match &op_tok.kind {
                    TokenKind::Plus => InfixOp::Add,
                    TokenKind::Minus => InfixOp::Sub,
                    TokenKind::Star => InfixOp::Mul,
                    TokenKind::Slash => InfixOp::Div,

                    _ => unreachable!(),
                };
                // Safe to unwrap, because it is called when an infix op peeked
                let (_, rbp) = Precedence::try_from_infix(&op_tok.kind)
                    .unwrap()
                    .to_infix_bp();
                let right = self.parse_expr_bp(rbp);
                let span = op_tok.span.merge(left.span).merge(right.span);
                Box::new(Expr::new(ExprKind::Infix(op, left, right), span))
            }
            TokenKind::Dot => {
                // TODO: Maybe need a new ExprKind
                unimplemented!()
            }
            _ => unreachable!("Not an infix operator but got in `parse_infix`: {:?}", left),
        }
    }
}

#[cfg(test)]
mod test {
    use std::{iter, slice};

    use crate::{common::assert::assert_len_eq, frontend::parser::token_stream::SliceStream};

    use super::*;

    /// Assert `actual` equal to `expected` recursively, skipping span check.
    fn assert_expr_kind_eq(actual: &Expr, expected: &Expr) {
        use ExprKind::*;
        match (&actual.kind, &expected.kind) {
            (Number(a), Number(e)) => assert_eq!(a, e),
            (String(a), String(e)) => assert_eq!(a, e),
            (Var(a), Var(e)) => assert_eq!(a, e),
            (Prefix(op_a, a), Prefix(op_e, e)) => {
                assert_eq!(op_a, op_e);
                assert_expr_kind_eq(&a, &e);
            }
            (Infix(op_a, a1, a2), Infix(op_e, e1, e2)) => {
                assert_eq!(op_a, op_e);
                assert_expr_kind_eq(&a1, &e1);
                assert_expr_kind_eq(&a2, &e2);
            }
            (App(a1, a2), App(e1, e2)) => {
                assert_expr_kind_eq(&a1, &e1);
                assert_expr_kind_eq(&a2, &e2);
            }
            (Err, Err) => {}
            _ => {
                panic!(
                    "Different expression kind:\n   actual: {:?}\n expected: {:?}",
                    actual, expected
                );
            }
        }
    }

    /// Parse tokens.
    fn parse<T>(tokens: &mut T) -> (Vec<Supercombinator>, Parser<'_, T>)
    where
        T: TokenStream,
    {
        let mut parser = Parser::new(tokens);
        (parser.parse(), parser)
    }

    /// Parse tokens and ensure no errors.
    ///
    /// # Panics
    /// If there are parsing errors, it will panic.
    fn parse_ok(tokens: &mut impl TokenStream) -> Vec<Supercombinator> {
        let (sc_defs, p) = parse(tokens);
        assert_eq!(
            p.diagnostics().len(),
            0,
            "Unexpected parse error:\n{:?}",
            p.diagnostics()
        );
        sc_defs
    }

    fn assert_sc_defs_eq<T>(token_kinds: T, sc_defs: &[Supercombinator])
    where
        T: IntoIterator<Item = TokenKind>,
        T::IntoIter: DoubleEndedIterator,
    {
        let tokens = token_kinds.into_iter().map(TokenKind::into_token);
        let actual = parse_ok(&mut SliceStream::new(tokens));

        assert_len_eq(&actual, sc_defs);

        for (a, e) in actual.iter().zip(sc_defs) {
            assert_eq!(a.name, e.name);
            assert_eq!(a.args, e.args);
            assert_expr_kind_eq(&a.body, &e.body);
        }
    }

    fn assert_expr_eq<T>(kinds: T, sc_def: &Supercombinator)
    where
        T: IntoIterator<Item = TokenKind>,
        T::IntoIter: DoubleEndedIterator,
    {
        let kinds = iter::once(TokenKind::Ident("x".into()))
            .chain(iter::once(TokenKind::Equal))
            .chain(kinds)
            .chain(iter::once(TokenKind::SemiColon));
        assert_sc_defs_eq(kinds, slice::from_ref(sc_def))
    }

    #[test]
    fn test() {
        let token_kinds = {
            use TokenKind::*;
            [Number(1.0)]
        };
        let expected = {
            use ExprKind::*;
            Supercombinator {
                name: "x".into(),
                args: vec![],
                body: Box::new(Number(1.0).into_expr()),
            }
        };
        assert_expr_eq(token_kinds, &expected);
    }
}
