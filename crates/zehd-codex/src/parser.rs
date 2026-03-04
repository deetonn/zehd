use zehd_tome::{Span, Token, TokenKind};

use crate::ast::*;
use crate::error::*;

pub(crate) struct Parser<'a> {
    tokens: &'a [Token],
    source: &'a str,
    pos: usize,
    pub(crate) errors: Vec<ParseError>,
    id_counter: u32,
}

// ── Infrastructure ───────────────────────────────────────────────

impl<'a> Parser<'a> {
    pub(crate) fn new(tokens: &'a [Token], source: &'a str) -> Self {
        Self {
            tokens,
            source,
            pos: 0,
            errors: Vec::new(),
            id_counter: 0,
        }
    }

    /// Allocate a fresh unique NodeId.
    pub(crate) fn next_id(&mut self) -> NodeId {
        let id = NodeId(self.id_counter);
        self.id_counter += 1;
        id
    }

    /// Look at the current token without consuming it.
    pub(crate) fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or_else(|| {
            self.tokens
                .last()
                .expect("token stream must contain at least EOF")
        })
    }

    /// Get the kind of the current token.
    pub(crate) fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    /// Look ahead n tokens from the current position.
    pub(crate) fn peek_nth(&self, n: usize) -> &Token {
        self.tokens.get(self.pos + n).unwrap_or_else(|| {
            self.tokens
                .last()
                .expect("token stream must contain at least EOF")
        })
    }

    /// Consume the current token and advance.
    pub(crate) fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if tok.kind != TokenKind::Eof {
            self.pos += 1;
        }
        tok
    }

    /// If the current token matches `kind`, consume it and return true.
    pub(crate) fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.peek_kind() == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Expect the current token to be `kind`. Consume and return its span, or emit an error.
    pub(crate) fn expect(&mut self, kind: &TokenKind) -> Option<Span> {
        if self.peek_kind() == kind {
            Some(self.advance().span)
        } else {
            let span = self.current_span();
            let msg = format!(
                "expected `{}`, found `{}`",
                token_kind_name(kind),
                token_kind_name(self.peek_kind())
            );
            self.errors.push(
                ParseError::error(ErrorCode::E002, msg, span)
                    .label(span, format!("expected `{}`", token_kind_name(kind)))
                    .build(),
            );
            None
        }
    }

    /// Check if the current token matches `kind` without consuming.
    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        self.peek_kind() == kind
    }

    /// Span of the current token.
    pub(crate) fn current_span(&self) -> Span {
        self.peek().span
    }

    /// Span of the previously consumed token.
    pub(crate) fn previous_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::new(0, 0)
        }
    }

    /// Create a span from `start` to the end of the previous token.
    pub(crate) fn span_from(&self, start: Span) -> Span {
        let end = self.previous_span();
        Span::new(start.start, end.end)
    }

    /// True if we've hit EOF.
    pub(crate) fn is_at_end(&self) -> bool {
        self.peek_kind() == &TokenKind::Eof
    }

    /// Extract the lexeme from source for a span.
    pub(crate) fn lexeme(&self, span: &Span) -> &'a str {
        span.lexeme(self.source)
    }

    /// Save the current position for speculative parsing.
    pub(crate) fn save(&self) -> usize {
        self.pos
    }

    /// Restore a previously saved position.
    pub(crate) fn restore(&mut self, pos: usize) {
        self.pos = pos;
    }

    // ── Error Recovery ───────────────────────────────────────────

    /// Skip tokens until we reach a synchronization point.
    pub(crate) fn synchronize(&mut self) {
        while !self.is_at_end() {
            // If we just passed a semicolon, we're synchronized.
            if self.previous_span().end > 0 {
                let prev_kind = &self.tokens[self.pos.saturating_sub(1)].kind;
                if *prev_kind == TokenKind::Semicolon {
                    return;
                }
            }

            match self.peek_kind() {
                // Anchor tokens: end of block or start of new item
                TokenKind::RightBrace
                | TokenKind::Semicolon
                | TokenKind::Let
                | TokenKind::Const
                | TokenKind::Fn
                | TokenKind::Type
                | TokenKind::Enum
                | TokenKind::Import
                | TokenKind::Get
                | TokenKind::Post
                | TokenKind::Put
                | TokenKind::Patch
                | TokenKind::Delete
                | TokenKind::Init
                | TokenKind::Error => {
                    // Consume semicolons so we start fresh
                    if self.peek_kind() == &TokenKind::Semicolon {
                        self.advance();
                    }
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
}

// ── Top-level Parsing ────────────────────────────────────────────

impl<'a> Parser<'a> {
    pub(crate) fn parse_program(&mut self) -> Program {
        let start = self.current_span();
        let mut items = Vec::new();

        while !self.is_at_end() {
            let before = self.pos;
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else {
                // Error recovery — skip to next synchronization point
                self.synchronize();
                // If we made no progress, skip one token to avoid infinite loop
                if self.pos == before {
                    self.advance();
                }
            }
        }

        let span = if items.is_empty() {
            start
        } else {
            Span::new(start.start, self.previous_span().end)
        };

        Program { items, span }
    }

    fn parse_item(&mut self) -> Option<Item> {
        // Collect attributes first
        let attrs = self.parse_attributes();

        let start = self.current_span();
        let kind = match self.peek_kind() {
            TokenKind::Import => self.parse_import_item()?,
            TokenKind::Type => self.parse_type_def_item(attrs)?,
            TokenKind::Enum => self.parse_enum_def_item()?,
            TokenKind::Fn => self.parse_function_item()?,
            TokenKind::Let | TokenKind::Const => self.parse_var_decl_item()?,
            TokenKind::Get => self.parse_http_block_item(HttpMethod::Get)?,
            TokenKind::Post => self.parse_http_block_item(HttpMethod::Post)?,
            TokenKind::Put => self.parse_http_block_item(HttpMethod::Put)?,
            TokenKind::Patch => self.parse_http_block_item(HttpMethod::Patch)?,
            TokenKind::Delete => self.parse_http_block_item(HttpMethod::Delete)?,
            TokenKind::Init => self.parse_init_block_item()?,
            TokenKind::Error => self.parse_error_handler_item()?,
            _ => {
                // If we collected attributes but the next thing isn't a type/enum, that's an error
                if !attrs.is_empty() {
                    let span = self.current_span();
                    self.errors.push(
                        ParseError::error(
                            ErrorCode::E016,
                            "attributes must be followed by a type or field definition",
                            span,
                        )
                        .build(),
                    );
                    return None;
                }
                // Try as expression statement
                self.parse_expr_stmt_item()?
            }
        };

        let span = self.span_from(start);
        Some(Item { id: self.next_id(), kind, span })
    }

    fn parse_import_item(&mut self) -> Option<ItemKind> {
        let import = self.parse_import()?;
        Some(ItemKind::Import(import))
    }

    fn parse_type_def_item(&mut self, attrs: Vec<Attribute>) -> Option<ItemKind> {
        let td = self.parse_type_def(attrs)?;
        Some(ItemKind::TypeDef(td))
    }

    fn parse_enum_def_item(&mut self) -> Option<ItemKind> {
        let ed = self.parse_enum_def()?;
        Some(ItemKind::EnumDef(ed))
    }

    fn parse_function_item(&mut self) -> Option<ItemKind> {
        let f = self.parse_function()?;
        Some(ItemKind::Function(f))
    }

    fn parse_var_decl_item(&mut self) -> Option<ItemKind> {
        let v = self.parse_var_decl()?;
        self.expect(&TokenKind::Semicolon);
        Some(ItemKind::VarDecl(v))
    }

    fn parse_http_block_item(&mut self, method: HttpMethod) -> Option<ItemKind> {
        let start = self.current_span();
        self.advance(); // consume get/post/put/patch/delete
        let body = self.parse_block()?;
        let span = self.span_from(start);
        Some(ItemKind::HttpBlock(HttpBlock { method, body, span }))
    }

    fn parse_init_block_item(&mut self) -> Option<ItemKind> {
        let start = self.current_span();
        self.advance(); // consume `init`
        let body = self.parse_block()?;
        let span = self.span_from(start);
        Some(ItemKind::InitBlock(InitBlock { body, span }))
    }

    fn parse_error_handler_item(&mut self) -> Option<ItemKind> {
        let start = self.current_span();
        self.advance(); // consume `error`
        self.expect(&TokenKind::LeftParen)?;
        let param = self.parse_ident()?;
        self.expect(&TokenKind::RightParen)?;
        let body = self.parse_block()?;
        let span = self.span_from(start);
        Some(ItemKind::ErrorHandler(ErrorHandler {
            param,
            body,
            span,
        }))
    }

    fn parse_expr_stmt_item(&mut self) -> Option<ItemKind> {
        let start = self.current_span();
        let expr = self.parse_expr()?;
        self.expect(&TokenKind::Semicolon);
        let span = self.span_from(start);
        Some(ItemKind::ExprStmt(ExprStmt { expr, span }))
    }
}

// ── Statement Parsing ────────────────────────────────────────────

impl<'a> Parser<'a> {
    pub(crate) fn parse_stmt(&mut self) -> Option<Stmt> {
        let start = self.current_span();
        let kind = match self.peek_kind() {
            TokenKind::Let | TokenKind::Const => {
                let v = self.parse_var_decl()?;
                self.expect(&TokenKind::Semicolon);
                StmtKind::VarDecl(v)
            }
            TokenKind::Return => {
                let ret = self.parse_return_stmt()?;
                self.expect(&TokenKind::Semicolon);
                StmtKind::Return(ret)
            }
            TokenKind::Break => {
                self.advance();
                self.expect(&TokenKind::Semicolon);
                StmtKind::Break
            }
            TokenKind::Continue => {
                self.advance();
                self.expect(&TokenKind::Semicolon);
                StmtKind::Continue
            }
            TokenKind::For => {
                let f = self.parse_for_stmt()?;
                StmtKind::For(f)
            }
            TokenKind::While => {
                let w = self.parse_while_stmt()?;
                StmtKind::While(w)
            }
            _ => {
                // Expression, possibly followed by `=` for assignment
                let expr = self.parse_expr()?;

                if self.eat(&TokenKind::Eq) {
                    let value = self.parse_expr()?;
                    self.expect(&TokenKind::Semicolon);
                    let span = self.span_from(start);
                    return Some(Stmt {
                        id: self.next_id(),
                        kind: StmtKind::Assignment(Assignment {
                            target: expr,
                            value,
                            span,
                        }),
                        span,
                    });
                }

                self.expect(&TokenKind::Semicolon);
                let span = self.span_from(start);
                StmtKind::ExprStmt(ExprStmt { expr, span })
            }
        };

        let span = self.span_from(start);
        Some(Stmt { id: self.next_id(), kind, span })
    }

    fn parse_return_stmt(&mut self) -> Option<ReturnStmt> {
        let start = self.current_span();
        self.advance(); // consume `return`

        let value = if !self.check(&TokenKind::Semicolon) && !self.check(&TokenKind::RightBrace) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        let span = self.span_from(start);
        Some(ReturnStmt { value, span })
    }

    fn parse_for_stmt(&mut self) -> Option<ForStmt> {
        let start = self.current_span();
        self.advance(); // consume `for`
        let binding = self.parse_ident()?;
        self.expect(&TokenKind::In)?;
        let iterable = self.parse_expr()?;
        let body = self.parse_block()?;
        let span = self.span_from(start);
        Some(ForStmt {
            binding,
            iterable,
            body,
            span,
        })
    }

    fn parse_while_stmt(&mut self) -> Option<WhileStmt> {
        let start = self.current_span();
        self.advance(); // consume `while`
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        let span = self.span_from(start);
        Some(WhileStmt {
            condition,
            body,
            span,
        })
    }
}

// ── Variable Declarations ────────────────────────────────────────

impl<'a> Parser<'a> {
    pub(crate) fn parse_var_decl(&mut self) -> Option<VarDecl> {
        let start = self.current_span();
        let mutable = match self.peek_kind() {
            TokenKind::Let => true,
            TokenKind::Const => false,
            _ => {
                let span = self.current_span();
                self.errors.push(
                    ParseError::error(ErrorCode::E004, "expected `let` or `const`", span).build(),
                );
                return None;
            }
        };
        self.advance();

        let name = self.parse_ident()?;

        let ty = if self.eat(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let initializer = if self.eat(&TokenKind::Eq) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        let span = self.span_from(start);
        Some(VarDecl {
            mutable,
            name,
            ty,
            initializer,
            span,
        })
    }
}

// ── Block Parsing ────────────────────────────────────────────────

impl<'a> Parser<'a> {
    pub(crate) fn parse_block(&mut self) -> Option<Block> {
        let start = self.current_span();
        self.expect(&TokenKind::LeftBrace)?;

        let mut stmts = Vec::new();
        let mut tail_expr = None;

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let before = self.pos;

            // Check for keyword-initiated statements first
            match self.peek_kind() {
                TokenKind::Let
                | TokenKind::Const
                | TokenKind::Return
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::For
                | TokenKind::While => {
                    if let Some(stmt) = self.parse_stmt() {
                        stmts.push(stmt);
                    } else {
                        self.synchronize();
                        if self.pos == before {
                            self.advance();
                        }
                    }
                    continue;
                }
                _ => {}
            }

            // Parse expression, then decide: expression statement or tail expression
            let expr_start = self.current_span();
            if let Some(expr) = self.parse_expr() {
                // Check if this is an assignment
                if self.eat(&TokenKind::Eq) {
                    if let Some(value) = self.parse_expr() {
                        self.expect(&TokenKind::Semicolon);
                        let span = self.span_from(expr_start);
                        stmts.push(Stmt {
                            id: self.next_id(),
                            kind: StmtKind::Assignment(Assignment {
                                target: expr,
                                value,
                                span,
                            }),
                            span,
                        });
                    } else {
                        self.synchronize();
                    }
                    continue;
                }

                if self.eat(&TokenKind::Semicolon) {
                    // Expression statement
                    let span = self.span_from(expr_start);
                    stmts.push(Stmt {
                        id: self.next_id(),
                        kind: StmtKind::ExprStmt(ExprStmt {
                            expr,
                            span,
                        }),
                        span,
                    });
                } else if self.check(&TokenKind::RightBrace) {
                    // Tail expression — this is the block's value
                    tail_expr = Some(Box::new(expr));
                } else if expr_ends_with_block(&expr.kind) {
                    // Block-terminated expressions (if, match) don't need `;`
                    let span = self.span_from(expr_start);
                    stmts.push(Stmt {
                        id: self.next_id(),
                        kind: StmtKind::ExprStmt(ExprStmt {
                            expr,
                            span,
                        }),
                        span,
                    });
                } else {
                    // Missing semicolon
                    let span = self.current_span();
                    self.errors.push(
                        ParseError::error(ErrorCode::E002, "expected `;` after expression", span)
                            .label(span, "expected `;` here")
                            .suggestion(
                                "add a semicolon",
                                span,
                                ";",
                                Applicability::MachineApplicable,
                            )
                            .note("zehd requires semicolons after all statements")
                            .build(),
                    );
                    // Treat as expression statement and continue
                    let span = self.span_from(expr_start);
                    stmts.push(Stmt {
                        id: self.next_id(),
                        kind: StmtKind::ExprStmt(ExprStmt {
                            expr,
                            span,
                        }),
                        span,
                    });
                }
            } else {
                self.synchronize();
                if self.pos == before {
                    self.advance();
                }
            }
        }

        self.expect(&TokenKind::RightBrace);
        let span = self.span_from(start);
        Some(Block {
            stmts,
            tail_expr,
            span,
        })
    }
}

// ── Function Parsing ─────────────────────────────────────────────

impl<'a> Parser<'a> {
    pub(crate) fn parse_function(&mut self) -> Option<Function> {
        let start = self.current_span();
        self.advance(); // consume `fn`

        let name = self.parse_ident()?;

        // Parse optional type params: <T, U>
        let type_params = self.parse_optional_type_params()?;

        self.expect(&TokenKind::LeftParen)?;
        let params = self.parse_param_list()?;
        self.expect(&TokenKind::RightParen)?;

        let return_type = if self.eat(&TokenKind::Colon) {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        let span = self.span_from(start);

        Some(Function {
            name,
            type_params,
            params,
            return_type,
            body,
            span,
        })
    }

    pub(crate) fn parse_param_list(&mut self) -> Option<Vec<Param>> {
        let mut params = Vec::new();

        if self.check(&TokenKind::RightParen) {
            return Some(params);
        }

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
            // Allow trailing comma
            if self.check(&TokenKind::RightParen) {
                break;
            }
        }

        Some(params)
    }
}

// ── Import Parsing ───────────────────────────────────────────────

impl<'a> Parser<'a> {
    pub(crate) fn parse_import(&mut self) -> Option<ImportItem> {
        let start = self.current_span();
        self.advance(); // consume `import`

        // Check for destructured import: import { a, b } from path;
        if self.check(&TokenKind::LeftBrace) {
            return self.parse_destructured_import(start);
        }

        // Non-destructured: import std::types::Response;
        // Parse as a path, and the last segment is the imported name
        let path = self.parse_import_path()?;
        self.expect(&TokenKind::Semicolon);

        // The last segment becomes the import name
        let last = path.segments.last()?.clone();
        let names = vec![ImportName {
            span: last.span,
            name: last,
        }];

        let span = self.span_from(start);
        Some(ImportItem { names, path, span })
    }

    fn parse_destructured_import(&mut self, start: Span) -> Option<ImportItem> {
        self.advance(); // consume `{`
        let mut names = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let name_start = self.current_span();
            let name = self.parse_ident()?;
            let span = self.span_from(name_start);
            names.push(ImportName { name, span });

            if !self.eat(&TokenKind::Comma) {
                break;
            }
            // Allow trailing comma
            if self.check(&TokenKind::RightBrace) {
                break;
            }
        }

        self.expect(&TokenKind::RightBrace)?;
        self.expect(&TokenKind::From)?;
        let path = self.parse_import_path()?;
        self.expect(&TokenKind::Semicolon);

        let span = self.span_from(start);
        Some(ImportItem { names, path, span })
    }

    fn parse_import_path(&mut self) -> Option<ImportPath> {
        let start = self.current_span();
        let mut segments = Vec::new();

        let first = self.parse_ident()?;
        segments.push(first);

        while self.eat(&TokenKind::ColonColon) {
            let seg = self.parse_ident()?;
            segments.push(seg);
        }

        let span = self.span_from(start);
        Some(ImportPath { segments, span })
    }
}

// ── Type & Enum Definitions ──────────────────────────────────────

impl<'a> Parser<'a> {
    pub(crate) fn parse_type_def(&mut self, leading_attrs: Vec<Attribute>) -> Option<TypeDef> {
        let start = self.current_span();
        self.advance(); // consume `type`

        let name = self.parse_ident()?;
        let type_params = self.parse_optional_type_params()?;

        self.expect(&TokenKind::LeftBrace)?;
        let mut fields = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let field_attrs = if self.check(&TokenKind::Hash) {
                self.parse_attributes()
            } else {
                Vec::new()
            };
            let field = self.parse_type_field(field_attrs)?;
            fields.push(field);
        }

        self.expect(&TokenKind::RightBrace)?;
        let span = self.span_from(start);

        let _ = leading_attrs; // Type-level attrs could be used later

        Some(TypeDef {
            name,
            type_params,
            fields,
            span,
        })
    }

    fn parse_type_field(&mut self, attributes: Vec<Attribute>) -> Option<TypeField> {
        let start = self.current_span();
        let name = self.parse_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type_annotation()?;
        self.expect(&TokenKind::Semicolon);
        let span = self.span_from(start);
        Some(TypeField {
            attributes,
            name,
            ty,
            span,
        })
    }

    pub(crate) fn parse_enum_def(&mut self) -> Option<EnumDef> {
        let start = self.current_span();
        self.advance(); // consume `enum`

        let name = self.parse_ident()?;
        let type_params = self.parse_optional_type_params()?;

        self.expect(&TokenKind::LeftBrace)?;
        let mut variants = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let v = self.parse_enum_variant()?;
            variants.push(v);
            // Eat optional comma between variants
            self.eat(&TokenKind::Comma);
        }

        self.expect(&TokenKind::RightBrace)?;
        let span = self.span_from(start);

        Some(EnumDef {
            name,
            type_params,
            variants,
            span,
        })
    }

    fn parse_enum_variant(&mut self) -> Option<EnumVariant> {
        let start = self.current_span();
        let name = self.parse_ident()?;

        let payload = if self.eat(&TokenKind::LeftParen) {
            let ty = self.parse_type_annotation()?;
            self.expect(&TokenKind::RightParen)?;
            Some(ty)
        } else {
            None
        };

        let span = self.span_from(start);
        Some(EnumVariant {
            name,
            payload,
            span,
        })
    }

    fn parse_optional_type_params(&mut self) -> Option<Vec<Ident>> {
        if !self.eat(&TokenKind::Lt) {
            return Some(Vec::new());
        }

        let mut params = Vec::new();
        loop {
            let name = self.parse_ident()?;
            params.push(name);

            if !self.eat(&TokenKind::Comma) {
                break;
            }
            if self.check(&TokenKind::Gt) {
                break;
            }
        }

        self.expect(&TokenKind::Gt)?;
        Some(params)
    }
}

// ── Type Annotations ─────────────────────────────────────────────

impl<'a> Parser<'a> {
    pub(crate) fn parse_type_annotation(&mut self) -> Option<TypeAnnotation> {
        let start = self.current_span();

        // Function type: (int, string) => bool
        if self.check(&TokenKind::LeftParen) {
            // Try to parse as function type
            let saved = self.save();
            if let Some(func_ty) = self.try_parse_function_type() {
                return Some(func_ty);
            }
            self.restore(saved);
        }

        // Named or generic type
        let name = self.parse_ident()?;

        if self.eat(&TokenKind::Lt) {
            // Generic type: Option<string>, Result<T, E>
            let mut args = Vec::new();
            loop {
                let arg = self.parse_type_annotation()?;
                args.push(arg);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
                if self.check(&TokenKind::Gt) {
                    break;
                }
            }
            self.expect(&TokenKind::Gt)?;
            let span = self.span_from(start);
            Some(TypeAnnotation {
                kind: TypeKind::Generic { name, args },
                span,
            })
        } else {
            let span = self.span_from(start);
            Some(TypeAnnotation {
                kind: TypeKind::Named(name),
                span,
            })
        }
    }

    fn try_parse_function_type(&mut self) -> Option<TypeAnnotation> {
        let start = self.current_span();
        self.advance(); // consume `(`

        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                let param = self.parse_type_annotation()?;
                params.push(param);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
                if self.check(&TokenKind::RightParen) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RightParen)?;
        self.expect(&TokenKind::FatArrow)?;
        let return_type = Box::new(self.parse_type_annotation()?);
        let span = self.span_from(start);
        Some(TypeAnnotation {
            kind: TypeKind::Function {
                params,
                return_type,
            },
            span,
        })
    }
}

// ── Attributes ───────────────────────────────────────────────────

impl<'a> Parser<'a> {
    pub(crate) fn parse_attributes(&mut self) -> Vec<Attribute> {
        let mut attrs = Vec::new();
        while self.check(&TokenKind::Hash) {
            if let Some(attr) = self.parse_attribute() {
                attrs.push(attr);
            }
        }
        attrs
    }

    fn parse_attribute(&mut self) -> Option<Attribute> {
        let start = self.current_span();
        self.advance(); // consume `#`
        self.expect(&TokenKind::LeftBracket)?;

        // Parse path: validate.min or just validate
        let mut path = Vec::new();
        let first = self.parse_ident()?;
        path.push(first);
        while self.eat(&TokenKind::Dot) {
            let seg = self.parse_ident()?;
            path.push(seg);
        }

        // Parse optional args: (...)
        // Inside attributes, `name=value` is parsed as Binary(Eq, Ident, value)
        let args = if self.eat(&TokenKind::LeftParen) {
            let mut args = Vec::new();
            if !self.check(&TokenKind::RightParen) {
                loop {
                    let arg = self.parse_attr_arg()?;
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
            args
        } else {
            Vec::new()
        };

        self.expect(&TokenKind::RightBracket)?;
        let span = self.span_from(start);
        Some(Attribute { path, args, span })
    }

    /// Parse an attribute argument. Handles `name=value` syntax by treating `=`
    /// as a binary operator (BinaryOp::Eq) in this context only.
    fn parse_attr_arg(&mut self) -> Option<Expr> {
        let expr = self.parse_expr()?;
        if self.eat(&TokenKind::Eq) {
            let value = self.parse_expr()?;
            let span = Span::new(expr.span.start, value.span.end);
            Some(Expr {
                id: self.next_id(),
                kind: ExprKind::Binary {
                    op: BinaryOp::Eq,
                    left: Box::new(expr),
                    right: Box::new(value),
                },
                span,
            })
        } else {
            Some(expr)
        }
    }
}

// ── Identifier Helper ────────────────────────────────────────────

impl<'a> Parser<'a> {
    pub(crate) fn parse_ident(&mut self) -> Option<Ident> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Identifier => {
                self.advance();
                Some(Ident {
                    name: self.lexeme(&tok.span).to_string(),
                    span: tok.span,
                })
            }
            // Allow certain keywords to be used as identifiers in specific contexts
            // like import paths (std, lib, etc.) and field names
            kind if is_contextual_ident(kind) => {
                self.advance();
                Some(Ident {
                    name: self.lexeme(&tok.span).to_string(),
                    span: tok.span,
                })
            }
            _ => {
                let span = self.current_span();
                self.errors.push(
                    ParseError::error(
                        ErrorCode::E005,
                        format!(
                            "expected identifier, found `{}`",
                            token_kind_name(self.peek_kind())
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

/// Returns true for expressions that end with a `}` block, which don't need
/// a trailing `;` when used as statements.
fn expr_ends_with_block(kind: &ExprKind) -> bool {
    matches!(
        kind,
        ExprKind::If { .. } | ExprKind::Match { .. } | ExprKind::Block(_)
    )
}

/// Returns true for keywords that can be used as identifiers in certain contexts.
fn is_contextual_ident(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Get
            | TokenKind::Post
            | TokenKind::Put
            | TokenKind::Patch
            | TokenKind::Delete
            | TokenKind::Init
            | TokenKind::Error
            | TokenKind::From
            | TokenKind::Ok
            | TokenKind::Err
            | TokenKind::Some
            | TokenKind::None
            | TokenKind::Type
    )
}

/// Human-readable name for a token kind.
pub(crate) fn token_kind_name(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Let => "let",
        TokenKind::Const => "const",
        TokenKind::Fn => "fn",
        TokenKind::If => "if",
        TokenKind::Else => "else",
        TokenKind::Match => "match",
        TokenKind::For => "for",
        TokenKind::In => "in",
        TokenKind::While => "while",
        TokenKind::Get => "get",
        TokenKind::Post => "post",
        TokenKind::Put => "put",
        TokenKind::Patch => "patch",
        TokenKind::Delete => "delete",
        TokenKind::Init => "init",
        TokenKind::Error => "error",
        TokenKind::Type => "type",
        TokenKind::Enum => "enum",
        TokenKind::Import => "import",
        TokenKind::From => "from",
        TokenKind::Return => "return",
        TokenKind::Break => "break",
        TokenKind::Continue => "continue",
        TokenKind::SelfKw => "self",
        TokenKind::True => "true",
        TokenKind::False => "false",
        TokenKind::None => "None",
        TokenKind::Some => "Some",
        TokenKind::Ok => "Ok",
        TokenKind::Err => "Err",
        TokenKind::Integer(_) => "integer",
        TokenKind::Float(_) => "float",
        TokenKind::String => "string",
        TokenKind::TimeLiteral(_) => "time literal",
        TokenKind::InterpolatedStringStart => "$\"",
        TokenKind::StringFragment => "string fragment",
        TokenKind::InterpolatedExprStart => "{",
        TokenKind::InterpolatedExprEnd => "}",
        TokenKind::InterpolatedStringEnd => "\"",
        TokenKind::Plus => "+",
        TokenKind::Minus => "-",
        TokenKind::Star => "*",
        TokenKind::Slash => "/",
        TokenKind::Percent => "%",
        TokenKind::EqEq => "==",
        TokenKind::BangEq => "!=",
        TokenKind::Lt => "<",
        TokenKind::Gt => ">",
        TokenKind::LtEq => "<=",
        TokenKind::GtEq => ">=",
        TokenKind::AmpAmp => "&&",
        TokenKind::PipePipe => "||",
        TokenKind::Bang => "!",
        TokenKind::Eq => "=",
        TokenKind::Question => "?",
        TokenKind::FatArrow => "=>",
        TokenKind::Dot => ".",
        TokenKind::ColonColon => "::",
        TokenKind::LeftBrace => "{",
        TokenKind::RightBrace => "}",
        TokenKind::LeftParen => "(",
        TokenKind::RightParen => ")",
        TokenKind::LeftBracket => "[",
        TokenKind::RightBracket => "]",
        TokenKind::Semicolon => ";",
        TokenKind::Comma => ",",
        TokenKind::Colon => ":",
        TokenKind::Hash => "#",
        TokenKind::Underscore => "_",
        TokenKind::DotDotDot => "...",
        TokenKind::Identifier => "identifier",
        TokenKind::Eof => "end of file",
    }
}
