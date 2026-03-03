use std::collections::HashMap;

use zehd_codex::ast::*;

use crate::error::*;
use crate::scope::*;
use crate::types::*;

// ── Resolve Output ──────────────────────────────────────────────

/// Output of the resolve pass.
pub struct ResolveResult {
    pub scopes: ScopeArena,
    /// Maps each resolved identifier's NodeId to the ScopeId where the symbol is defined.
    pub resolutions: HashMap<NodeId, ScopeId>,
    pub errors: Vec<TypeError>,
}

// ── Resolver ────────────────────────────────────────────────────

pub struct Resolver {
    scopes: ScopeArena,
    resolutions: HashMap<NodeId, ScopeId>,
    errors: Vec<TypeError>,
    current_scope: ScopeId,
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver {
    pub fn new() -> Self {
        let mut scopes = ScopeArena::new();
        let module_scope = scopes.create(ScopeKind::Module, None);
        Self {
            scopes,
            resolutions: HashMap::new(),
            errors: Vec::new(),
            current_scope: module_scope,
        }
    }

    /// Run the resolve pass on a program.
    pub fn resolve(mut self, program: &Program) -> ResolveResult {
        // Pass 1a: Collect top-level declarations (forward references).
        self.collect_top_level(program);

        // Pass 1b: Walk all bodies to resolve references.
        self.resolve_items(&program.items);

        ResolveResult {
            scopes: self.scopes,
            resolutions: self.resolutions,
            errors: self.errors,
        }
    }

    // ── Pass 1a: Collect Declarations ───────────────────────────

    fn collect_top_level(&mut self, program: &Program) {
        for item in &program.items {
            match &item.kind {
                ItemKind::Function(f) => {
                    self.define_symbol(
                        &f.name.name,
                        Symbol {
                            kind: SymbolKind::Function,
                            ty: Type::Var(0), // placeholder — checker fills in real type
                            mutable: false,
                            defined_at: f.name.span,
                            used: false,
                        },
                    );
                }
                ItemKind::TypeDef(td) => {
                    self.define_symbol(
                        &td.name.name,
                        Symbol {
                            kind: SymbolKind::TypeDef,
                            ty: Type::Unit, // type defs don't have a value type here
                            mutable: false,
                            defined_at: td.name.span,
                            used: false,
                        },
                    );
                }
                ItemKind::EnumDef(ed) => {
                    self.define_symbol(
                        &ed.name.name,
                        Symbol {
                            kind: SymbolKind::EnumDef,
                            ty: Type::Unit,
                            mutable: false,
                            defined_at: ed.name.span,
                            used: false,
                        },
                    );
                    // Register each variant as a symbol too.
                    for variant in &ed.variants {
                        self.define_symbol(
                            &variant.name.name,
                            Symbol {
                                kind: SymbolKind::EnumVariant,
                                ty: Type::Unit,
                                mutable: false,
                                defined_at: variant.name.span,
                                used: false,
                            },
                        );
                    }
                }
                ItemKind::VarDecl(v) => {
                    self.define_symbol(
                        &v.name.name,
                        Symbol {
                            kind: SymbolKind::Variable,
                            ty: Type::Var(0),
                            mutable: v.mutable,
                            defined_at: v.name.span,
                            used: false,
                        },
                    );
                }
                ItemKind::Import(imp) => {
                    for name in &imp.names {
                        self.define_symbol(
                            &name.name.name,
                            Symbol {
                                kind: SymbolKind::Import,
                                ty: Type::Var(0),
                                mutable: false,
                                defined_at: name.name.span,
                                used: false,
                            },
                        );
                    }
                }
                _ => {}
            }
        }
    }

    // ── Pass 1b: Walk Bodies ────────────────────────────────────

    fn resolve_items(&mut self, items: &[Item]) {
        for item in items {
            self.resolve_item(item);
        }
    }

    fn resolve_item(&mut self, item: &Item) {
        match &item.kind {
            ItemKind::Import(_) => {
                // Imports are already registered in collect_top_level.
            }
            ItemKind::TypeDef(_) => {
                // Type field resolution is handled by the checker.
            }
            ItemKind::EnumDef(_) => {
                // Enum variant resolution is handled by the checker.
            }
            ItemKind::Function(f) => {
                self.resolve_function(f);
            }
            ItemKind::VarDecl(v) => {
                if let Some(init) = &v.initializer {
                    self.resolve_expr(init);
                }
            }
            ItemKind::HttpBlock(h) => {
                let parent = self.current_scope;
                self.current_scope = self.scopes.create(ScopeKind::HttpHandler, Some(parent));
                self.resolve_block(&h.body);
                self.current_scope = parent;
            }
            ItemKind::InitBlock(i) => {
                let parent = self.current_scope;
                self.current_scope = self.scopes.create(ScopeKind::Init, Some(parent));
                self.resolve_block(&i.body);
                self.current_scope = parent;
            }
            ItemKind::ErrorHandler(e) => {
                let parent = self.current_scope;
                self.current_scope = self.scopes.create(ScopeKind::ErrorHandler, Some(parent));
                // The error parameter is a binding in the handler scope.
                self.define_symbol(
                    &e.param.name,
                    Symbol {
                        kind: SymbolKind::Parameter,
                        ty: Type::Var(0),
                        mutable: false,
                        defined_at: e.param.span,
                        used: false,
                    },
                );
                self.resolve_block(&e.body);
                self.current_scope = parent;
            }
            ItemKind::ExprStmt(es) => {
                self.resolve_expr(&es.expr);
            }
        }
    }

    fn resolve_function(&mut self, f: &Function) {
        let parent = self.current_scope;
        self.current_scope = self.scopes.create(ScopeKind::Function, Some(parent));

        // Register parameters.
        for param in &f.params {
            self.define_symbol(
                &param.name.name,
                Symbol {
                    kind: SymbolKind::Parameter,
                    ty: Type::Var(0),
                    mutable: false,
                    defined_at: param.name.span,
                    used: false,
                },
            );
        }

        self.resolve_block(&f.body);
        self.current_scope = parent;
    }

    fn resolve_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.resolve_stmt(stmt);
        }
        if let Some(tail) = &block.tail_expr {
            self.resolve_expr(tail);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::VarDecl(v) => {
                if let Some(init) = &v.initializer {
                    self.resolve_expr(init);
                }
                self.define_symbol(
                    &v.name.name,
                    Symbol {
                        kind: SymbolKind::Variable,
                        ty: Type::Var(0),
                        mutable: v.mutable,
                        defined_at: v.name.span,
                        used: false,
                    },
                );
            }
            StmtKind::ExprStmt(es) => {
                self.resolve_expr(&es.expr);
            }
            StmtKind::Return(r) => {
                if let Some(val) = &r.value {
                    self.resolve_expr(val);
                }
            }
            StmtKind::Break => {
                if !self.scopes.is_in_loop(self.current_scope) {
                    self.errors.push(
                        TypeError::error(
                            TypeErrorCode::T131,
                            "`break` outside of loop",
                            stmt.span,
                        )
                        .build(),
                    );
                }
            }
            StmtKind::Continue => {
                if !self.scopes.is_in_loop(self.current_scope) {
                    self.errors.push(
                        TypeError::error(
                            TypeErrorCode::T132,
                            "`continue` outside of loop",
                            stmt.span,
                        )
                        .build(),
                    );
                }
            }
            StmtKind::For(f) => {
                self.resolve_expr(&f.iterable);
                let parent = self.current_scope;
                self.current_scope = self.scopes.create(ScopeKind::Loop, Some(parent));
                // The loop binding.
                self.define_symbol(
                    &f.binding.name,
                    Symbol {
                        kind: SymbolKind::Variable,
                        ty: Type::Var(0),
                        mutable: false,
                        defined_at: f.binding.span,
                        used: false,
                    },
                );
                self.resolve_block(&f.body);
                self.current_scope = parent;
            }
            StmtKind::While(w) => {
                self.resolve_expr(&w.condition);
                let parent = self.current_scope;
                self.current_scope = self.scopes.create(ScopeKind::Loop, Some(parent));
                self.resolve_block(&w.body);
                self.current_scope = parent;
            }
            StmtKind::Assignment(a) => {
                self.resolve_expr(&a.target);
                self.resolve_expr(&a.value);
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Ident(ident) => {
                if let Some((scope_id, _)) = self.scopes.lookup(self.current_scope, &ident.name) {
                    self.resolutions.insert(expr.id, scope_id);
                    self.scopes.mark_used(scope_id, &ident.name);
                } else {
                    self.errors.push(
                        TypeError::error(
                            TypeErrorCode::T100,
                            format!("undefined variable `{}`", ident.name),
                            ident.span,
                        )
                        .label(ident.span, "not found in this scope")
                        .build(),
                    );
                }
            }
            ExprKind::SelfExpr => {
                if !self.scopes.is_in_handler(self.current_scope) {
                    self.errors.push(
                        TypeError::error(
                            TypeErrorCode::T133,
                            "`self` is only available inside HTTP handlers, init, and error blocks",
                            expr.span,
                        )
                        .build(),
                    );
                }
            }
            ExprKind::Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            ExprKind::Unary { operand, .. } => {
                self.resolve_expr(operand);
            }
            ExprKind::Call { callee, args, .. } => {
                self.resolve_expr(callee);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            ExprKind::FieldAccess { object, .. } => {
                self.resolve_expr(object);
            }
            ExprKind::Index { object, index } => {
                self.resolve_expr(object);
                self.resolve_expr(index);
            }
            ExprKind::Try(inner) => {
                self.resolve_expr(inner);
            }
            ExprKind::If {
                condition,
                then_block,
                else_block,
            } => {
                self.resolve_expr(condition);
                let parent = self.current_scope;
                self.current_scope = self.scopes.create(ScopeKind::Block, Some(parent));
                self.resolve_block(then_block);
                self.current_scope = parent;
                if let Some(eb) = else_block {
                    match eb {
                        ElseBranch::ElseBlock(block) => {
                            self.current_scope =
                                self.scopes.create(ScopeKind::Block, Some(parent));
                            self.resolve_block(block);
                            self.current_scope = parent;
                        }
                        ElseBranch::ElseIf(elif) => {
                            self.resolve_expr(elif);
                        }
                    }
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                self.resolve_expr(scrutinee);
                for arm in arms {
                    let parent = self.current_scope;
                    self.current_scope = self.scopes.create(ScopeKind::Block, Some(parent));
                    self.resolve_pattern(&arm.pattern);
                    self.resolve_expr(&arm.body);
                    self.current_scope = parent;
                }
            }
            ExprKind::ArrowFunction { params, body, .. } => {
                let parent = self.current_scope;
                self.current_scope = self.scopes.create(ScopeKind::Function, Some(parent));
                for param in params {
                    self.define_symbol(
                        &param.name.name,
                        Symbol {
                            kind: SymbolKind::Parameter,
                            ty: Type::Var(0),
                            mutable: false,
                            defined_at: param.name.span,
                            used: false,
                        },
                    );
                }
                match body {
                    ArrowBody::Expr(e) => self.resolve_expr(e),
                    ArrowBody::Block(b) => self.resolve_block(b),
                }
                self.current_scope = parent;
            }
            ExprKind::ObjectLiteral { fields } => {
                for field in fields {
                    if let Some(value) = &field.value {
                        self.resolve_expr(value);
                    } else {
                        // Shorthand { name } — resolve `name` as an identifier.
                        if let Some((scope_id, _)) =
                            self.scopes.lookup(self.current_scope, &field.key.name)
                        {
                            self.resolutions.insert(expr.id, scope_id);
                            self.scopes.mark_used(scope_id, &field.key.name);
                        } else {
                            self.errors.push(
                                TypeError::error(
                                    TypeErrorCode::T100,
                                    format!("undefined variable `{}`", field.key.name),
                                    field.key.span,
                                )
                                .label(field.key.span, "not found in this scope")
                                .build(),
                            );
                        }
                    }
                }
            }
            ExprKind::ListLiteral { elements } => {
                for elem in elements {
                    self.resolve_expr(elem);
                }
            }
            ExprKind::InterpolatedString { parts } => {
                for part in parts {
                    if let InterpolatedPart::Expr(e) = part {
                        self.resolve_expr(e);
                    }
                }
            }
            ExprKind::Block(block) => {
                let parent = self.current_scope;
                self.current_scope = self.scopes.create(ScopeKind::Block, Some(parent));
                self.resolve_block(block);
                self.current_scope = parent;
            }
            ExprKind::Grouped(inner) => {
                self.resolve_expr(inner);
            }
            ExprKind::EnumConstructor { arg, .. } => {
                self.resolve_expr(arg);
            }
            // Literals need no resolution.
            ExprKind::IntLiteral(_)
            | ExprKind::FloatLiteral(_)
            | ExprKind::StringLiteral(_)
            | ExprKind::TimeLiteral(_)
            | ExprKind::BoolLiteral(_)
            | ExprKind::NoneLiteral => {}
        }
    }

    fn resolve_pattern(&mut self, pattern: &Pattern) {
        match &pattern.kind {
            PatternKind::Binding(ident) => {
                self.define_symbol(
                    &ident.name,
                    Symbol {
                        kind: SymbolKind::Variable,
                        ty: Type::Var(0),
                        mutable: false,
                        defined_at: ident.span,
                        used: false,
                    },
                );
            }
            PatternKind::EnumVariant { binding, .. } => {
                if let Some(inner) = binding {
                    self.resolve_pattern(inner);
                }
            }
            PatternKind::Wildcard | PatternKind::Literal(_) => {}
        }
    }

    // ── Helpers ─────────────────────────────────────────────────

    fn define_symbol(&mut self, name: &str, symbol: Symbol) {
        let span = symbol.defined_at;
        if !self.scopes.define(self.current_scope, name.to_string(), symbol) {
            self.errors.push(
                TypeError::error(
                    TypeErrorCode::T102,
                    format!("duplicate definition of `{name}`"),
                    span,
                )
                .label(span, "already defined in this scope")
                .build(),
            );
        }
    }
}
