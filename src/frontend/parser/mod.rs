use crate::{
    common::{arena::Arena, span::Span},
    frontend::{
        lexer::{Lexer, Token, TokenKind},
        parser::{
            associativity::Associativity,
            diagnostic::{ParseDiagnostic, ParseDiagnosticKind},
            expr::{Expr, ExprKind, ExprPtr, InfixOp, PrefixOp},
            precedence::{BindingPower, Precedence},
        },
    },
};

pub mod associativity;
mod diagnostic;
mod expr;
pub mod precedence;

#[derive(Debug, Clone)]
pub struct Supercombinator<'arena> {
    pub name: String,
    pub args: Vec<String>,
    pub body: ExprPtr<'arena, 'arena>,
}

pub struct Parser<'src, 'arena> {
    lexer: Lexer<'src>,
    arena: &'arena Arena<Expr<'arena>>,
    diagnostics: Vec<ParseDiagnostic>,
}

impl<'src, 'arena> Parser<'src, 'arena> {
    pub fn new(lexer: Lexer<'src>, arena: &'arena Arena<Expr<'arena>>) -> Self {
        Self {
            lexer,
            arena,
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
        self.lexer.advance()
    }

    /// Peek current token with no cost.
    fn peek(&self) -> &Token {
        self.lexer.current()
    }

    // ---- arena ----

    fn alloc_expr(&self, expr: Expr<'arena>) -> ExprPtr<'arena, 'arena> {
        self.arena.alloc(expr)
    }

    // ---- parsing ----

    pub fn parse(&mut self) -> Vec<Supercombinator<'arena>> {
        let mut scs = Vec::new();
        while !self.peek().kind.is_eof() {
            if let Some(sc) = self.parse_sc() {
                scs.push(sc);
            }
        }
        scs
    }

    fn parse_sc(&mut self) -> Option<Supercombinator<'arena>> {
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
            self.report(ParseDiagnosticKind::NotABinding,  Default::default());
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
                        body: self.alloc_expr(Expr::err()),
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
            if matches!(cur.kind, TokenKind::SemiColon) {
                break cur;
            }
            is_semicolon = false;
            self.eat();
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

    fn parse_expr(&mut self) -> ExprPtr<'arena, 'arena> {
        self.parse_expr_bp(BindingPower::lowest())
    }

    /// Core Pratt parser — parse an expression with the given minimum binding power.
    ///
    /// Returns an [`ExprPtr`] handle into the arena.
    /// The handle is valid as long as the parser is alive.
    fn parse_expr_bp(&mut self, min_bp: BindingPower) -> ExprPtr<'arena, 'arena> {
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

    fn parse_app(&mut self, left: ExprPtr<'arena, 'arena>) -> ExprPtr<'arena, 'arena> {
        let (_, rbp) = Precedence::Call.to_infix_bp();
        let right = self.parse_expr_bp(rbp);
        let span = left.span.merge(right.span);
        self.alloc_expr(Expr::new(ExprKind::App(left, right), span))
    }

    /// Parse a token as the **start** of an expression.
    fn parse_prefix(&mut self, token: Token) -> ExprPtr<'arena, 'arena> {
        match token.kind {
            TokenKind::Number(n) => self.alloc_expr(Expr::new(ExprKind::Number(n), token.span)),

            TokenKind::String(s) => self.alloc_expr(Expr::new(ExprKind::String(s), token.span)),

            TokenKind::Ident(name) => self.alloc_expr(Expr::new(ExprKind::Var(name), token.span)),

            TokenKind::Plus => {
                let right = self.parse_expr_bp(BindingPower::prefix());
                let span = token.span.merge(right.span);
                self.alloc_expr(Expr::new(ExprKind::Prefix(PrefixOp::Pos, right), span))
            }

            TokenKind::Minus => {
                let right = self.parse_expr_bp(BindingPower::prefix());
                let span = token.span.merge(right.span);
                self.alloc_expr(Expr::new(ExprKind::Prefix(PrefixOp::Neg, right), span))
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
                self.alloc_expr(Expr::err())
            }

            _ => {
                self.report(
                    ParseDiagnosticKind::NotAnExpression { found: token.kind },
                    token.span,
                );
                self.alloc_expr(Expr::err())
            }
        }
    }

    /// Parse an infix (left denotation) operator and its right-hand side.
    fn parse_infix(&mut self, left: ExprPtr<'arena, 'arena>) -> ExprPtr<'arena, 'arena> {
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
                self.alloc_expr(Expr::new(ExprKind::Infix(op, left, right), span))
            }
            TokenKind::Dot => {
                // TODO: Maybe need a new ExprKind
                unimplemented!()
            }
            _ => unreachable!("Not an infix operator but got in `parse_infix`: {:?}", left),
        }
    }
}
