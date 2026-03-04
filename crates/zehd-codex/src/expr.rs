use zehd_tome::{Span, TokenKind};

use crate::ast::*;
use crate::error::*;
use crate::parser::Parser;
use crate::precedence::*;

// ── Expression Entry Point ───────────────────────────────────────

impl<'a> Parser<'a> {
    /// Parse an expression (public entry point).
    pub(crate) fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_expr_bp(0)
    }

    /// Pratt parser core: parse expression with minimum binding power.
    fn parse_expr_bp(&mut self, min_bp: u8) -> Option<Expr> {
        let mut lhs = self.parse_prefix()?;

        loop {
            if self.is_at_end() {
                break;
            }

            // Try postfix (highest precedence)
            if let Some(left_bp) = postfix_binding_power(self.peek_kind()) {
                if left_bp < min_bp {
                    break;
                }
                lhs = self.parse_postfix(lhs)?;
                continue;
            }

            // Try infix
            if let Some((left_bp, right_bp)) = infix_binding_power(self.peek_kind()) {
                if left_bp < min_bp {
                    break;
                }
                lhs = self.parse_infix(lhs, right_bp)?;
                continue;
            }

            break;
        }

        Some(lhs)
    }
}

// ── Prefix Parsing ───────────────────────────────────────────────

impl<'a> Parser<'a> {
    fn parse_prefix(&mut self) -> Option<Expr> {
        let tok = self.peek().clone();

        match &tok.kind {
            // Literals
            TokenKind::Integer(v) => {
                let v = *v;
                self.advance();
                Some(Expr {
                    id: self.next_id(),
                    kind: ExprKind::IntLiteral(v),
                    span: tok.span,
                })
            }
            TokenKind::Float(v) => {
                let v = *v;
                self.advance();
                Some(Expr {
                    id: self.next_id(),
                    kind: ExprKind::FloatLiteral(v),
                    span: tok.span,
                })
            }
            TokenKind::String => {
                self.advance();
                // Strip quotes from lexeme
                let raw = self.lexeme(&tok.span);
                let content = raw[1..raw.len() - 1].to_string();
                Some(Expr {
                    id: self.next_id(),
                    kind: ExprKind::StringLiteral(content),
                    span: tok.span,
                })
            }
            TokenKind::TimeLiteral(v) => {
                let v = *v;
                self.advance();
                Some(Expr {
                    id: self.next_id(),
                    kind: ExprKind::TimeLiteral(v),
                    span: tok.span,
                })
            }
            TokenKind::True => {
                self.advance();
                Some(Expr {
                    id: self.next_id(),
                    kind: ExprKind::BoolLiteral(true),
                    span: tok.span,
                })
            }
            TokenKind::False => {
                self.advance();
                Some(Expr {
                    id: self.next_id(),
                    kind: ExprKind::BoolLiteral(false),
                    span: tok.span,
                })
            }
            TokenKind::None => {
                self.advance();
                Some(Expr {
                    id: self.next_id(),
                    kind: ExprKind::NoneLiteral,
                    span: tok.span,
                })
            }

            // Enum constructors: Some(x), Ok(x), Err(x)
            TokenKind::Some | TokenKind::Ok | TokenKind::Err => {
                self.advance();
                let name = Ident {
                    name: self.lexeme(&tok.span).to_string(),
                    span: tok.span,
                };
                // Check if followed by `(` for constructor
                if self.check(&TokenKind::LeftParen) {
                    self.advance(); // consume `(`
                    let arg = self.parse_expr()?;
                    self.expect(&TokenKind::RightParen)?;
                    let span = Span::new(tok.span.start, self.previous_span().end);
                    Some(Expr {
                        id: self.next_id(),
                        kind: ExprKind::EnumConstructor {
                            name,
                            arg: Box::new(arg),
                        },
                        span,
                    })
                } else {
                    // Just the identifier (e.g., used in pattern-like contexts)
                    Some(Expr {
                        id: self.next_id(),
                        kind: ExprKind::Ident(name),
                        span: tok.span,
                    })
                }
            }

            // Identifier
            TokenKind::Identifier => {
                self.advance();
                let name = self.lexeme(&tok.span).to_string();
                let ident_expr = Expr {
                    id: self.next_id(),
                    kind: ExprKind::Ident(Ident {
                        name,
                        span: tok.span,
                    }),
                    span: tok.span,
                };

                // Speculatively try `ident<Type>(args)` — generic call syntax.
                if self.check(&TokenKind::Lt) {
                    let saved = self.save();
                    let saved_errors = self.errors.len();
                    if let Some(call) = self.try_parse_generic_call(ident_expr.clone()) {
                        return Some(call);
                    }
                    self.restore(saved);
                    self.errors.truncate(saved_errors);
                }

                Some(ident_expr)
            }

            // Self
            TokenKind::SelfKw => {
                self.advance();
                Some(Expr {
                    id: self.next_id(),
                    kind: ExprKind::SelfExpr,
                    span: tok.span,
                })
            }

            // Unary operators
            TokenKind::Bang | TokenKind::Minus => {
                let right_bp = prefix_binding_power(&tok.kind).unwrap();
                self.advance();
                let operand = self.parse_expr_bp(right_bp)?;
                let span = Span::new(tok.span.start, operand.span.end);
                let op = match tok.kind {
                    TokenKind::Bang => UnaryOp::Not,
                    TokenKind::Minus => UnaryOp::Neg,
                    _ => unreachable!(),
                };
                Some(Expr {
                    id: self.next_id(),
                    kind: ExprKind::Unary {
                        op,
                        operand: Box::new(operand),
                    },
                    span,
                })
            }

            // Parenthesized expression or arrow function
            TokenKind::LeftParen => self.parse_paren_or_arrow(),

            // If expression
            TokenKind::If => self.parse_if_expr(),

            // Match expression
            TokenKind::Match => self.parse_match_expr(),

            // Block expression or object literal
            TokenKind::LeftBrace => self.parse_brace_expr(),

            // List literal
            TokenKind::LeftBracket => self.parse_list_literal(),

            // Interpolated string
            TokenKind::InterpolatedStringStart => self.parse_interpolated_string(),

            _ => {
                let span = self.current_span();
                self.errors.push(
                    ParseError::error(
                        ErrorCode::E003,
                        format!(
                            "expected expression, found `{}`",
                            crate::parser::token_kind_name(&tok.kind)
                        ),
                        span,
                    )
                    .build(),
                );
                None
            }
        }
    }
}

// ── Postfix Parsing ──────────────────────────────────────────────

impl<'a> Parser<'a> {
    fn parse_postfix(&mut self, lhs: Expr) -> Option<Expr> {
        match self.peek_kind() {
            TokenKind::Question => {
                self.advance();
                let span = Span::new(lhs.span.start, self.previous_span().end);
                Some(Expr {
                    id: self.next_id(),
                    kind: ExprKind::Try(Box::new(lhs)),
                    span,
                })
            }
            TokenKind::Dot => {
                self.advance();
                let field = self.parse_ident()?;
                let span = Span::new(lhs.span.start, self.previous_span().end);
                Some(Expr {
                    id: self.next_id(),
                    kind: ExprKind::FieldAccess {
                        object: Box::new(lhs),
                        field,
                    },
                    span,
                })
            }
            TokenKind::LeftBracket => {
                self.advance(); // consume `[`
                let index = self.parse_expr()?;
                self.expect(&TokenKind::RightBracket)?;
                let span = Span::new(lhs.span.start, self.previous_span().end);
                Some(Expr {
                    id: self.next_id(),
                    kind: ExprKind::Index {
                        object: Box::new(lhs),
                        index: Box::new(index),
                    },
                    span,
                })
            }
            TokenKind::LeftParen => {
                // Function call, possibly with generic type args
                // Generic call syntax: callee<Type>(args)
                // We don't get here for `<` because it's handled as infix Lt.
                // Instead, generic calls are handled by the `<` ambiguity logic.
                self.parse_call(lhs, Vec::new())
            }
            _ => Some(lhs),
        }
    }

    /// Speculatively parse `<Type, ...>(args)` after an identifier.
    /// Returns `None` to signal backtrack if this isn't a generic call.
    fn try_parse_generic_call(&mut self, callee: Expr) -> Option<Expr> {
        self.advance(); // consume `<`
        let mut type_args = Vec::new();
        loop {
            let arg = self.parse_type_annotation()?;
            type_args.push(arg);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
            if self.check(&TokenKind::Gt) {
                break;
            }
        }
        if !self.eat(&TokenKind::Gt) {
            return None;
        }
        if !self.check(&TokenKind::LeftParen) {
            return None;
        }
        self.parse_call(callee, type_args)
    }

    fn parse_call(&mut self, callee: Expr, type_args: Vec<TypeAnnotation>) -> Option<Expr> {
        self.advance(); // consume `(`
        let mut args = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                let arg = self.parse_expr()?;
                args.push(arg);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
                if self.check(&TokenKind::RightParen) {
                    break;
                }
            }
        }

        self.expect(&TokenKind::RightParen)?;
        let span = Span::new(callee.span.start, self.previous_span().end);
        Some(Expr {
            id: self.next_id(),
            kind: ExprKind::Call {
                callee: Box::new(callee),
                type_args,
                args,
            },
            span,
        })
    }
}

// ── Infix Parsing ────────────────────────────────────────────────

impl<'a> Parser<'a> {
    fn parse_infix(&mut self, lhs: Expr, right_bp: u8) -> Option<Expr> {
        let op_tok = self.advance().clone();
        let op = token_to_binary_op(&op_tok.kind)?;
        let rhs = self.parse_expr_bp(right_bp)?;
        let span = Span::new(lhs.span.start, rhs.span.end);
        Some(Expr {
            id: self.next_id(),
            kind: ExprKind::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            },
            span,
        })
    }
}

// ── Compound Expression Parsing ──────────────────────────────────

impl<'a> Parser<'a> {
    /// Parse `(` — could be grouped expr, arrow function, or function call type args.
    fn parse_paren_or_arrow(&mut self) -> Option<Expr> {
        // Speculative parse: try arrow function first
        let saved = self.save();
        let saved_errors = self.errors.len();
        if let Some(arrow) = self.try_parse_arrow_function() {
            return Some(arrow);
        }
        // Backtrack and truncate any errors from the speculative parse
        self.restore(saved);
        self.errors.truncate(saved_errors);

        // Grouped expression
        let start = self.current_span();
        self.advance(); // consume `(`
        let expr = self.parse_expr()?;
        self.expect(&TokenKind::RightParen)?;
        let span = Span::new(start.start, self.previous_span().end);
        Some(Expr {
            id: self.next_id(),
            kind: ExprKind::Grouped(Box::new(expr)),
            span,
        })
    }

    fn try_parse_arrow_function(&mut self) -> Option<Expr> {
        let start = self.current_span();
        self.advance(); // consume `(`

        // Parse params
        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                let param_start = self.current_span();
                let name = self.parse_ident()?;
                let ty = if self.eat(&TokenKind::Colon) {
                    Some(self.parse_type_annotation()?)
                } else {
                    None
                };
                let span = self.span_from(param_start);
                params.push(Param { name, ty, span });

                if !self.eat(&TokenKind::Comma) {
                    break;
                }
                if self.check(&TokenKind::RightParen) {
                    break;
                }
            }
        }

        // Must have `)` then `=>` or `)` `:` type `=>`
        if !self.eat(&TokenKind::RightParen) {
            return None;
        }

        let return_type = if self.eat(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        if !self.eat(&TokenKind::FatArrow) {
            return None;
        }

        // Arrow body: block or expression
        let body = if self.check(&TokenKind::LeftBrace) {
            ArrowBody::Block(self.parse_block()?)
        } else {
            ArrowBody::Expr(Box::new(self.parse_expr()?))
        };

        let span = self.span_from(start);
        Some(Expr {
            id: self.next_id(),
            kind: ExprKind::ArrowFunction {
                params,
                return_type,
                body,
            },
            span,
        })
    }

    fn parse_if_expr(&mut self) -> Option<Expr> {
        let start = self.current_span();
        self.advance(); // consume `if`

        let condition = Box::new(self.parse_expr()?);
        let then_block = self.parse_block()?;

        let else_block = if self.eat(&TokenKind::Else) {
            if self.check(&TokenKind::If) {
                Some(ElseBranch::ElseIf(Box::new(self.parse_if_expr()?)))
            } else {
                Some(ElseBranch::ElseBlock(self.parse_block()?))
            }
        } else {
            None
        };

        let span = self.span_from(start);
        Some(Expr {
            id: self.next_id(),
            kind: ExprKind::If {
                condition,
                then_block,
                else_block,
            },
            span,
        })
    }

    fn parse_match_expr(&mut self) -> Option<Expr> {
        let start = self.current_span();
        self.advance(); // consume `match`

        let scrutinee = Box::new(self.parse_expr()?);
        self.expect(&TokenKind::LeftBrace)?;

        let mut arms = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let arm = self.parse_match_arm()?;
            arms.push(arm);
            // Allow optional comma/semicolon between arms
            self.eat(&TokenKind::Comma);
            self.eat(&TokenKind::Semicolon);
        }

        self.expect(&TokenKind::RightBrace)?;
        let span = self.span_from(start);
        Some(Expr {
            id: self.next_id(),
            kind: ExprKind::Match { scrutinee, arms },
            span,
        })
    }

    fn parse_match_arm(&mut self) -> Option<MatchArm> {
        let start = self.current_span();
        let pattern = self.parse_pattern()?;
        self.expect(&TokenKind::FatArrow)?;

        let body = if self.check(&TokenKind::LeftBrace) {
            // Block body — parse as block expression
            let block = self.parse_block()?;
            let span = block.span;
            Expr {
                id: self.next_id(),
                kind: ExprKind::Block(block),
                span,
            }
        } else {
            self.parse_expr()?
        };

        let span = self.span_from(start);
        Some(MatchArm {
            pattern,
            body,
            span,
        })
    }

    pub(crate) fn parse_pattern(&mut self) -> Option<Pattern> {
        let start = self.current_span();
        let tok = self.peek().clone();

        match &tok.kind {
            TokenKind::Underscore => {
                self.advance();
                Some(Pattern {
                    kind: PatternKind::Wildcard,
                    span: tok.span,
                })
            }
            TokenKind::Integer(v) => {
                let v = *v;
                self.advance();
                Some(Pattern {
                    kind: PatternKind::Literal(LiteralPattern::Int(v)),
                    span: tok.span,
                })
            }
            TokenKind::Float(v) => {
                let v = *v;
                self.advance();
                Some(Pattern {
                    kind: PatternKind::Literal(LiteralPattern::Float(v)),
                    span: tok.span,
                })
            }
            TokenKind::String => {
                self.advance();
                let raw = self.lexeme(&tok.span);
                let content = raw[1..raw.len() - 1].to_string();
                Some(Pattern {
                    kind: PatternKind::Literal(LiteralPattern::String(content)),
                    span: tok.span,
                })
            }
            TokenKind::True => {
                self.advance();
                Some(Pattern {
                    kind: PatternKind::Literal(LiteralPattern::Bool(true)),
                    span: tok.span,
                })
            }
            TokenKind::False => {
                self.advance();
                Some(Pattern {
                    kind: PatternKind::Literal(LiteralPattern::Bool(false)),
                    span: tok.span,
                })
            }
            // Enum variant pattern: SomeEnum.Variant(binding) or just Variant
            TokenKind::Identifier | TokenKind::Some | TokenKind::Ok | TokenKind::Err
            | TokenKind::None => {
                // Could be: binding, Variant, Type.Variant, Type.Variant(pat)
                let first = Ident {
                    name: self.lexeme(&tok.span).to_string(),
                    span: tok.span,
                };
                self.advance();

                let mut path = vec![first];

                // Dot-separated path: Type.Variant
                while self.eat(&TokenKind::Dot) {
                    let seg = self.parse_ident()?;
                    path.push(seg);
                }

                // Check for payload: Variant(binding)
                let binding = if self.eat(&TokenKind::LeftParen) {
                    let inner = self.parse_pattern()?;
                    self.expect(&TokenKind::RightParen)?;
                    Some(Box::new(inner))
                } else {
                    None
                };

                let span = self.span_from(start);

                // If it's a single lowercase identifier with no payload, it's a binding
                if path.len() == 1 && binding.is_none() {
                    let name = &path[0].name;
                    let first_char = name.chars().next().unwrap_or('a');
                    if first_char.is_lowercase() && tok.kind == TokenKind::Identifier {
                        return Some(Pattern {
                            kind: PatternKind::Binding(path.into_iter().next().unwrap()),
                            span,
                        });
                    }
                }

                Some(Pattern {
                    kind: PatternKind::EnumVariant { path, binding },
                    span,
                })
            }
            _ => {
                let span = self.current_span();
                self.errors.push(
                    ParseError::error(
                        ErrorCode::E023,
                        format!(
                            "expected pattern, found `{}`",
                            crate::parser::token_kind_name(&tok.kind)
                        ),
                        span,
                    )
                    .build(),
                );
                None
            }
        }
    }

    /// Parse `{` — could be object literal or block expression.
    /// Heuristic: `{ ident :` → object literal; `{ ident ,` or `{ ident }` → shorthand object; else → block.
    fn parse_brace_expr(&mut self) -> Option<Expr> {
        // Lookahead to distinguish object literal from block
        let next = self.peek_nth(1);
        let next_next = self.peek_nth(2);

        let is_object = match (&next.kind, &next_next.kind) {
            // { ident: ... } — object literal
            (TokenKind::Identifier, TokenKind::Colon) => true,
            // { ident, ... } — shorthand object
            (TokenKind::Identifier, TokenKind::Comma) => true,
            // { ident } — shorthand object (single field)
            (TokenKind::Identifier, TokenKind::RightBrace) => true,
            // Empty object: { }
            (TokenKind::RightBrace, _) => true,
            // Keyword fields that can be identifiers
            (kind, TokenKind::Colon) if is_field_like(kind) => true,
            _ => false,
        };

        if is_object {
            self.parse_object_literal()
        } else {
            let start = self.current_span();
            let block = self.parse_block()?;
            let span = self.span_from(start);
            Some(Expr {
                id: self.next_id(),
                kind: ExprKind::Block(block),
                span,
            })
        }
    }

    fn parse_object_literal(&mut self) -> Option<Expr> {
        let start = self.current_span();
        self.advance(); // consume `{`
        let mut fields = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let field_start = self.current_span();
            let key = self.parse_ident()?;

            let value = if self.eat(&TokenKind::Colon) {
                Some(self.parse_expr()?)
            } else {
                None // shorthand
            };

            let span = self.span_from(field_start);
            fields.push(ObjectField { key, value, span });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
            // Allow trailing comma
            if self.check(&TokenKind::RightBrace) {
                break;
            }
        }

        self.expect(&TokenKind::RightBrace)?;
        let span = self.span_from(start);
        Some(Expr {
            id: self.next_id(),
            kind: ExprKind::ObjectLiteral { fields },
            span,
        })
    }

    fn parse_list_literal(&mut self) -> Option<Expr> {
        let start = self.current_span();
        self.advance(); // consume `[`
        let mut elements = Vec::new();

        if !self.check(&TokenKind::RightBracket) {
            loop {
                let elem = self.parse_expr()?;
                elements.push(elem);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
                if self.check(&TokenKind::RightBracket) {
                    break;
                }
            }
        }

        self.expect(&TokenKind::RightBracket)?;
        let span = self.span_from(start);
        Some(Expr {
            id: self.next_id(),
            kind: ExprKind::ListLiteral { elements },
            span,
        })
    }

    fn parse_interpolated_string(&mut self) -> Option<Expr> {
        let start = self.current_span();
        self.advance(); // consume InterpolatedStringStart ($")
        let mut parts = Vec::new();

        loop {
            match self.peek_kind() {
                TokenKind::StringFragment => {
                    let tok = self.advance().clone();
                    let text = self.lexeme(&tok.span).to_string();
                    parts.push(InterpolatedPart::Literal(text, tok.span));
                }
                TokenKind::InterpolatedExprStart => {
                    self.advance(); // consume `{`
                    let expr = self.parse_expr()?;
                    self.expect(&TokenKind::InterpolatedExprEnd)?;
                    parts.push(InterpolatedPart::Expr(expr));
                }
                TokenKind::InterpolatedStringEnd => {
                    self.advance(); // consume closing `"`
                    break;
                }
                TokenKind::Eof => {
                    let span = self.current_span();
                    self.errors.push(
                        ParseError::error(
                            ErrorCode::E026,
                            "unterminated interpolated string",
                            span,
                        )
                        .build(),
                    );
                    break;
                }
                _ => {
                    let span = self.current_span();
                    self.errors.push(
                        ParseError::error(
                            ErrorCode::E026,
                            "unexpected token in interpolated string",
                            span,
                        )
                        .build(),
                    );
                    self.advance();
                }
            }
        }

        let span = self.span_from(start);
        Some(Expr {
            id: self.next_id(),
            kind: ExprKind::InterpolatedString { parts },
            span,
        })
    }
}

/// Check if a token kind could be a field name in an object literal.
fn is_field_like(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Identifier
            | TokenKind::Get
            | TokenKind::Post
            | TokenKind::Put
            | TokenKind::Patch
            | TokenKind::Delete
            | TokenKind::Init
            | TokenKind::Error
            | TokenKind::From
            | TokenKind::Type
    )
}
