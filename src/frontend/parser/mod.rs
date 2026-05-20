use crate::{
    common::{arena::Arena, span::Span},
    frontend::{
        lexer::{Lexer, Token, TokenKind},
        parser::{
            diagnostic::{ParseDiagnostic, ParseDiagnosticKind},
            expr::{Expr, ExprKind, ExprPtr, InfixOp, PrefixOp},
            precedence::{Precedence, infix_left_bp, infix_right_bp},
        },
    },
};

mod diagnostic;
mod expr;
mod precedence;

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
        let name: String;

        loop {
            // Clone / copy what we need from peek() so the borrow
            // does not conflict with the mutable calls below.
            let kind = self.peek().kind.clone();
            let span = self.peek().span;
            match &kind {
                TokenKind::Eof => {
                    self.report(ParseDiagnosticKind::NotABinding, span);
                    self.eat();
                    return None;
                }
                TokenKind::SemiColon => {
                    self.report(ParseDiagnosticKind::NotABinding, span);
                    self.eat();
                    return None;
                }
                TokenKind::Ident(n) => {
                    name = n.clone();
                    self.eat();
                    // only if meet an ident we can normally parse an SC
                    break;
                }
                _ => {
                    is_ident_first = false;
                    self.eat();
                }
            }
        }

        if !is_ident_first {
            self.report(ParseDiagnosticKind::NotABinding, Span::default());
        }

        let mut args: Vec<String> = Vec::new();
        loop {
            let kind = self.peek().kind.clone();
            let span = self.peek().span;
            match &kind {
                TokenKind::Eof | TokenKind::SemiColon => {
                    self.report(
                        ParseDiagnosticKind::UnexpectedToken {
                            expected: &["ident", "colon"],
                            found: kind.clone(),
                        },
                        span,
                    );
                    self.eat();
                    return Some(Supercombinator {
                        name,
                        args,
                        body: self.alloc_expr(Expr::err()),
                    });
                }
                TokenKind::Equal => {
                    self.eat();
                    break;
                }
                TokenKind::Ident(n) => {
                    args.push(n.clone());
                    self.eat();
                }
                _ => {
                    self.report(
                        ParseDiagnosticKind::UnexpectedToken {
                            expected: &["ident", "colon"],
                            found: kind.clone(),
                        },
                        span,
                    );
                    self.eat();
                }
            }
        }

        let body = self.parse_expr();

        // finally expect a semicolon
        let cur = self.eat();
        if cur.kind != TokenKind::SemiColon {
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
        self.parse_expr_bp(Precedence::Lowest)
    }

    /// Core Pratt parser — parse an expression with the given minimum binding power.
    ///
    /// Returns an [`ExprPtr`] handle into the arena.
    /// The handle is valid as long as the parser is alive.
    fn parse_expr_bp(&mut self, min_bp: Precedence) -> ExprPtr<'arena, 'arena> {
        let cur = self.eat();

        let mut left = self.parse_prefix(&cur);

        loop {
            let bp = match infix_left_bp(&self.peek().kind) {
                Some(bp) => bp,
                None => break,
            };
            if min_bp >= bp {
                break
            }
            left = self.parse_infix(left);
        }

        left
    }

    /// Parse a token as the **start** of an expression (null denotation).
    fn parse_prefix(&mut self, token: &Token) -> ExprPtr<'arena, 'arena> {
        match &token.kind {
            TokenKind::Number(n) => self.alloc_expr(Expr::new(ExprKind::Number(*n), token.span)),

            TokenKind::String(s) => {
                self.alloc_expr(Expr::new(ExprKind::String(s.clone()), token.span))
            }

            TokenKind::Ident(name) => {
                self.alloc_expr(Expr::new(ExprKind::Var(name.clone()), token.span))
            }

            TokenKind::Plus => {
                let right = self.parse_expr_bp(Precedence::Prefix);
                let span = token.span.merge(right.span);
                self.alloc_expr(Expr::new(ExprKind::Prefix(PrefixOp::Pos, right), span))
            }

            TokenKind::Minus => {
                let right = self.parse_expr_bp(Precedence::Prefix);
                let span = token.span.merge(right.span);
                self.alloc_expr(Expr::new(ExprKind::Prefix(PrefixOp::Neg, right), span))
            }

            TokenKind::LParen => {
                let inner = self.parse_expr_bp(Precedence::Lowest);
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
                    ParseDiagnosticKind::NotAnExpression {
                        found: token.kind.clone(),
                    },
                    token.span,
                );
                self.alloc_expr(Expr::err())
            }
        }
    }

    /// Parse an infix (left denotation) operator and its right-hand side.
    fn parse_infix(&mut self, left: ExprPtr<'arena, 'arena>) -> ExprPtr<'arena, 'arena> {
        // Copy everything we need from peek() before any mutable call.
        let op_kind = self.peek().kind.clone();
        let op_span = self.peek().span;

        match &op_kind {
            // ── binary arithmetic operators ──────────────────────────
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash => {
                let op = match &op_kind {
                    TokenKind::Plus => InfixOp::Add,
                    TokenKind::Minus => InfixOp::Sub,
                    TokenKind::Star => InfixOp::Mul,
                    TokenKind::Slash => InfixOp::Div,
                    _ => unreachable!(),
                };
                self.eat(); // consume the operator
                let right = self.parse_expr_bp(infix_right_bp(&op_kind));
                let span = op_span.merge(left.span).merge(right.span);
                self.alloc_expr(Expr::new(ExprKind::Infix(op, left, right), span))
            }

            // ── function application (juxtaposition) ─────────────────
            //
            // `f x y`  →  App(App(f, x), y)
            //
            // We do NOT eat the operator — the recursive call's
            // initial `eat()` will consume it as the start of the
            // argument expression.
            TokenKind::Number(_)
            | TokenKind::String(_)
            | TokenKind::Ident(_)
            | TokenKind::LParen => {
                let right = self.parse_expr_bp(infix_right_bp(&op_kind));
                let span = left.span.merge(right.span);
                self.alloc_expr(Expr::new(ExprKind::App(left, right), span))
            }

            _ => unreachable!("infix_left_bp returned Some but parse_infix has no handler"),
        }
    }
}
