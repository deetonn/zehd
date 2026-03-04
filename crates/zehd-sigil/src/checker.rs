use std::collections::HashMap;

use zehd_codex::ast::*;
use zehd_tome::Span;

use crate::error::*;
use crate::infer::InferCtx;
use crate::resolve::ResolveResult;
use crate::scope::*;
use crate::types::*;
use crate::ModuleTypes;

// ── Type Table ──────────────────────────────────────────────────

/// Maps AST NodeIds to their resolved types.
pub type TypeTable = HashMap<NodeId, Type>;

// ── Checker ─────────────────────────────────────────────────────

pub struct Checker {
    pub types: TypeTable,
    pub infer: InferCtx,
    pub scopes: ScopeArena,
    pub resolutions: HashMap<NodeId, ScopeId>,
    pub errors: Vec<TypeError>,
    current_scope: ScopeId,
    /// Expected return type for the current function/handler.
    return_type: Option<Type>,
    /// Resolved user-defined type definitions (name → StructType with fields).
    type_defs: HashMap<String, StructType>,
    /// Module type registry — maps module path → exported name → Type.
    module_types: ModuleTypes,
}

impl Checker {
    pub fn new(resolve_result: ResolveResult, module_types: ModuleTypes) -> Self {
        Self {
            types: HashMap::new(),
            infer: InferCtx::new(),
            scopes: resolve_result.scopes,
            resolutions: resolve_result.resolutions,
            errors: resolve_result.errors,
            current_scope: 0, // module scope
            return_type: None,
            type_defs: HashMap::new(),
            module_types,
        }
    }

    /// Run the type checking pass on a program.
    pub fn check(mut self, program: &Program) -> CheckerResult {
        // Pre-pass: collect user-defined type definitions so they're available
        // when resolving type annotations in functions and variable declarations.
        self.collect_type_defs(&program.items);

        // Pre-pass: collect function signatures so forward references work.
        self.collect_function_signatures(&program.items);

        self.check_items(&program.items);

        // Zonk all types — finalize inference.
        let mut final_types = HashMap::new();
        for (id, ty) in &self.types {
            final_types.insert(*id, self.infer.zonk(ty));
        }

        CheckerResult {
            types: final_types,
            scopes: self.scopes,
            errors: self.errors,
        }
    }

    // ── Type Definition Collection ──────────────────────────────

    fn collect_type_defs(&mut self, items: &[Item]) {
        for item in items {
            if let ItemKind::TypeDef(td) = &item.kind {
                let fields: Vec<(String, Type)> = td
                    .fields
                    .iter()
                    .map(|f| {
                        let ty = self.resolve_type_annotation(&f.ty);
                        (f.name.name.clone(), ty)
                    })
                    .collect();
                self.type_defs.insert(
                    td.name.name.clone(),
                    StructType {
                        name: Some(td.name.name.clone()),
                        fields,
                        type_params: vec![],
                    },
                );
            }
        }
    }

    fn collect_function_signatures(&mut self, items: &[Item]) {
        for item in items {
            if let ItemKind::Function(f) = &item.kind {
                let param_types: Vec<Type> = f
                    .params
                    .iter()
                    .map(|p| {
                        if let Some(ann) = &p.ty {
                            self.resolve_type_annotation(ann)
                        } else {
                            self.infer.fresh()
                        }
                    })
                    .collect();
                let return_ty = if let Some(ann) = &f.return_type {
                    self.resolve_type_annotation(ann)
                } else {
                    self.infer.fresh()
                };
                let func_ty = Type::Function(FunctionType {
                    params: param_types,
                    return_type: Box::new(return_ty),
                });
                self.update_symbol_type(&f.name.name, func_ty);
            }
        }
    }

    // ── Items ───────────────────────────────────────────────────

    fn check_items(&mut self, items: &[Item]) {
        for item in items {
            self.check_item(item);
        }
    }

    fn check_item(&mut self, item: &Item) {
        match &item.kind {
            ItemKind::Import(imp) => {
                self.check_import(imp);
            }
            ItemKind::TypeDef(_) => {} // already handled in collect_type_defs
            ItemKind::EnumDef(_) => {}
            ItemKind::Function(f) => self.check_function(f),
            ItemKind::VarDecl(v) => self.check_var_decl(v),
            ItemKind::HttpBlock(h) => self.check_http_block(h),
            ItemKind::InitBlock(i) => self.check_init_block(i),
            ItemKind::ErrorHandler(e) => self.check_error_handler(e),
            ItemKind::ExprStmt(es) => {
                self.check_expr(&es.expr);
            }
        }
    }

    fn check_import(&mut self, imp: &ImportItem) {
        // Build module path from segments: ["std", "log"] → "std::log"
        let module_path = imp
            .path
            .segments
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
            .join("::");

        // Look up the module in the registry.
        let module_exports = self.module_types.get(&module_path);

        for name in &imp.names {
            let export_name = &name.name.name;
            match module_exports {
                Some(exports) => {
                    if let Some(ty) = exports.get(export_name) {
                        // Update the symbol's type in the current scope.
                        if let Some(sym) = self.scopes.lookup_mut(self.current_scope, export_name) {
                            sym.ty = ty.clone();
                        }
                        // If it's a named struct type, register it as a type definition
                        // so it can be used as a type annotation (e.g. `const req: Request = ...`).
                        if let Type::Struct(st) = ty {
                            if st.name.is_some() {
                                self.type_defs.insert(export_name.clone(), st.clone());
                            }
                        }
                    } else {
                        self.errors.push(
                            TypeError::error(
                                TypeErrorCode::T103,
                                format!("'{export_name}' is not exported by module '{module_path}'"),
                                name.span,
                            )
                            .build(),
                        );
                        // Set to Error to prevent cascading type errors.
                        if let Some(sym) = self.scopes.lookup_mut(self.current_scope, export_name) {
                            sym.ty = Type::Error;
                        }
                    }
                }
                None => {
                    // Set all imported names to Error to prevent cascading type errors.
                    for n in &imp.names {
                        if let Some(sym) = self.scopes.lookup_mut(self.current_scope, &n.name.name) {
                            sym.ty = Type::Error;
                        }
                    }
                    self.errors.push(
                        TypeError::error(
                            TypeErrorCode::T103,
                            format!("unknown module '{module_path}'"),
                            imp.path.span,
                        )
                        .build(),
                    );
                    // Only report "unknown module" once per import statement.
                    return;
                }
            }
        }
    }

    fn check_function(&mut self, f: &Function) {
        self.enter_scope(ScopeKind::Function);

        // Register parameters with their annotated types (or fresh vars).
        let mut param_types = Vec::new();
        for param in &f.params {
            let ty = if let Some(ann) = &param.ty {
                self.resolve_type_annotation(ann)
            } else {
                self.infer.fresh()
            };
            self.define_in_current(&param.name.name, ty.clone(), false, param.name.span);
            param_types.push(ty);
        }

        // Set up return type.
        let return_ty = if let Some(ann) = &f.return_type {
            self.resolve_type_annotation(ann)
        } else {
            self.infer.fresh()
        };
        let prev_return = self.return_type.take();
        self.return_type = Some(return_ty.clone());

        // Check body.
        let body_ty = self.check_block(&f.body);

        // Unify body type with return type (if the body has a tail expression).
        if f.body.tail_expr.is_some() {
            if let Err(e) = self.infer.unify(&body_ty, &return_ty, f.span) {
                self.errors.push(e);
            }
        }

        // Build the function type and update the symbol.
        let func_ty = Type::Function(FunctionType {
            params: param_types,
            return_type: Box::new(return_ty),
        });
        self.update_symbol_type(&f.name.name, func_ty);

        self.return_type = prev_return;
        self.exit_scope();
    }

    fn check_http_block(&mut self, h: &HttpBlock) {
        self.enter_scope(ScopeKind::HttpHandler);
        let prev_return = self.return_type.take();
        self.return_type = Some(self.infer.fresh());
        self.check_block(&h.body);
        self.return_type = prev_return;
        self.exit_scope();
    }

    fn check_init_block(&mut self, i: &InitBlock) {
        self.enter_scope(ScopeKind::Init);
        self.check_block(&i.body);
        self.exit_scope();
    }

    fn check_error_handler(&mut self, e: &ErrorHandler) {
        self.enter_scope(ScopeKind::ErrorHandler);
        // Error param gets a generic error type.
        self.define_in_current(&e.param.name, Type::Error, false, e.param.span);
        self.check_block(&e.body);
        self.exit_scope();
    }

    // ── Variable Declarations ───────────────────────────────────

    fn check_var_decl(&mut self, v: &VarDecl) {
        let annotated_ty = v.ty.as_ref().map(|ann| self.resolve_type_annotation(ann));

        let init_ty = v
            .initializer
            .as_ref()
            .map(|init| self.check_expr(init));

        let final_ty = match (annotated_ty, init_ty) {
            (Some(ann), Some(init)) => {
                if let Err(e) = self.infer.unify(&init, &ann, v.span) {
                    self.errors.push(e);
                }
                ann
            }
            (Some(ann), None) => ann,
            (None, Some(init)) => init,
            (None, None) => self.infer.fresh(),
        };

        // Define (or update) in the checker's current scope so subsequent
        // lookups find the variable with its resolved type.
        self.define_or_update(&v.name.name, final_ty, v.mutable, v.name.span);
    }

    // ── Blocks ──────────────────────────────────────────────────

    fn check_block(&mut self, block: &Block) -> Type {
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
        if let Some(tail) = &block.tail_expr {
            self.check_expr(tail)
        } else {
            Type::Unit
        }
    }

    // ── Statements ──────────────────────────────────────────────

    fn check_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::VarDecl(v) => self.check_var_decl(v),
            StmtKind::ExprStmt(es) => {
                self.check_expr(&es.expr);
            }
            StmtKind::Return(r) => {
                let val_ty = if let Some(val) = &r.value {
                    self.check_expr(val)
                } else {
                    Type::Unit
                };
                if let Some(ret) = &self.return_type.clone() {
                    if let Err(e) = self.infer.unify(&val_ty, ret, r.span) {
                        self.errors.push(e);
                    }
                }
            }
            StmtKind::Break | StmtKind::Continue => {
                // Validated in resolve pass.
            }
            StmtKind::For(f) => {
                let iter_ty = self.check_expr(&f.iterable);
                let elem_ty = match self.infer.resolve(&iter_ty) {
                    Type::List(elem) => *elem,
                    Type::Error => Type::Error,
                    other => {
                        self.errors.push(
                            TypeError::error(
                                TypeErrorCode::T116,
                                format!("cannot iterate over `{other}`"),
                                f.iterable.span,
                            )
                            .label(f.iterable.span, "not iterable")
                            .note("for loops require a List type")
                            .build(),
                        );
                        Type::Error
                    }
                };
                self.enter_scope(ScopeKind::Loop);
                self.define_in_current(&f.binding.name, elem_ty, false, f.binding.span);
                self.check_block(&f.body);
                self.exit_scope();
            }
            StmtKind::While(w) => {
                let cond_ty = self.check_expr(&w.condition);
                self.expect_bool(&cond_ty, w.condition.span);
                self.enter_scope(ScopeKind::Loop);
                self.check_block(&w.body);
                self.exit_scope();
            }
            StmtKind::Assignment(a) => {
                let target_ty = self.check_expr(&a.target);
                let value_ty = self.check_expr(&a.value);

                // Check mutability.
                if let ExprKind::Ident(ident) = &a.target.kind {
                    if let Some((scope_id, sym)) =
                        self.scopes.lookup(self.current_scope, &ident.name)
                    {
                        if !sym.mutable {
                            self.errors.push(
                                TypeError::error(
                                    TypeErrorCode::T130,
                                    format!(
                                        "cannot assign to `{}` — declared with `const`",
                                        ident.name
                                    ),
                                    a.target.span,
                                )
                                .label(sym.defined_at, "defined as immutable here")
                                .label(a.target.span, "cannot assign")
                                .build(),
                            );
                        }
                        let _ = scope_id; // used for lookup only
                    }
                }

                if let Err(e) = self.infer.unify(&value_ty, &target_ty, a.span) {
                    self.errors.push(e);
                }
            }
        }
    }

    // ── Expressions ─────────────────────────────────────────────

    pub fn check_expr(&mut self, expr: &Expr) -> Type {
        let ty = self.synthesize_expr(expr);
        self.types.insert(expr.id, ty.clone());
        ty
    }

    fn synthesize_expr(&mut self, expr: &Expr) -> Type {
        match &expr.kind {
            // ── Literals ────────────────────────────────────────
            ExprKind::IntLiteral(_) => Type::Int,
            ExprKind::FloatLiteral(_) => Type::Float,
            ExprKind::StringLiteral(_) => Type::String,
            ExprKind::TimeLiteral(_) => Type::Time,
            ExprKind::BoolLiteral(_) => Type::Bool,
            ExprKind::NoneLiteral => {
                let inner = self.infer.fresh();
                Type::Option(Box::new(inner))
            }

            // ── Enum constructors ───────────────────────────────
            ExprKind::EnumConstructor { name, arg } => {
                let arg_ty = self.check_expr(arg);
                match name.name.as_str() {
                    "Some" => Type::Option(Box::new(arg_ty)),
                    "Ok" => {
                        let err_ty = self.infer.fresh();
                        Type::Result(Box::new(arg_ty), Box::new(err_ty))
                    }
                    "Err" => {
                        let ok_ty = self.infer.fresh();
                        Type::Result(Box::new(ok_ty), Box::new(arg_ty))
                    }
                    _ => {
                        // User-defined enum constructor — resolve via scope.
                        Type::Error
                    }
                }
            }

            // ── Identifiers ─────────────────────────────────────
            ExprKind::Ident(ident) => {
                if let Some((scope_id, sym)) =
                    self.scopes.lookup(self.current_scope, &ident.name)
                {
                    let _ = scope_id;
                    sym.ty.clone()
                } else {
                    // Error already reported by resolve pass.
                    Type::Error
                }
            }

            ExprKind::SelfExpr => {
                // Build self type from std::http module types.
                let http_module = self.module_types.get("std::http");
                let request_ty = http_module
                    .and_then(|m| m.get("Request"))
                    .cloned()
                    .unwrap_or(Type::Error);
                let response_ty = http_module
                    .and_then(|m| m.get("Response"))
                    .cloned()
                    .unwrap_or(Type::Error);

                Type::Struct(StructType {
                    name: Some("RouteContext".to_string()),
                    fields: vec![
                        ("request".to_string(), request_ty),
                        ("response".to_string(), response_ty),
                        ("params".to_string(), Type::Map(Box::new(Type::String), Box::new(Type::String))),
                    ],
                    type_params: vec![],
                })
            }

            // ── Binary operators ────────────────────────────────
            ExprKind::Binary { op, left, right } => {
                let left_ty = self.check_expr(left);
                let right_ty = self.check_expr(right);
                self.check_binary_op(*op, &left_ty, &right_ty, expr.span)
            }

            // ── Unary operators ─────────────────────────────────
            ExprKind::Unary { op, operand } => {
                let operand_ty = self.check_expr(operand);
                match op {
                    UnaryOp::Neg => {
                        let resolved = self.infer.resolve(&operand_ty);
                        match resolved {
                            Type::Int | Type::Float | Type::Time | Type::Error => operand_ty,
                            Type::Var(_) => {
                                // Try to unify with Int.
                                if let Err(e) = self.infer.unify(&operand_ty, &Type::Int, expr.span) {
                                    self.errors.push(e);
                                    Type::Error
                                } else {
                                    Type::Int
                                }
                            }
                            _ => {
                                self.errors.push(
                                    TypeError::error(
                                        TypeErrorCode::T111,
                                        format!("cannot negate `{operand_ty}`"),
                                        expr.span,
                                    )
                                    .label(operand.span, format!("has type `{operand_ty}`"))
                                    .build(),
                                );
                                Type::Error
                            }
                        }
                    }
                    UnaryOp::Not => {
                        self.expect_bool(&operand_ty, operand.span);
                        Type::Bool
                    }
                }
            }

            // ── Try operator (?) ────────────────────────────────
            ExprKind::Try(inner) => {
                let inner_ty = self.check_expr(inner);
                let resolved = self.infer.resolve(&inner_ty);
                match resolved {
                    Type::Result(ok, _err) => *ok,
                    Type::Error => Type::Error,
                    _ => {
                        self.errors.push(
                            TypeError::error(
                                TypeErrorCode::T117,
                                format!("the `?` operator can only be used on `Result` types, found `{resolved}`"),
                                expr.span,
                            )
                            .label(inner.span, format!("has type `{resolved}`"))
                            .build(),
                        );
                        Type::Error
                    }
                }
            }

            // ── Field access ────────────────────────────────────
            ExprKind::FieldAccess { object, field } => {
                let obj_ty = self.check_expr(object);
                let resolved = self.infer.resolve(&obj_ty);
                match resolved {
                    Type::Struct(ref s) => {
                        if let Some((_, ty)) = s.fields.iter().find(|(n, _)| n == &field.name) {
                            ty.clone()
                        } else {
                            self.errors.push(
                                TypeError::error(
                                    TypeErrorCode::T104,
                                    format!("no field `{}` on type `{resolved}`", field.name),
                                    field.span,
                                )
                                .label(field.span, "unknown field")
                                .build(),
                            );
                            Type::Error
                        }
                    }
                    Type::Error => Type::Error,
                    _ => {
                        self.errors.push(
                            TypeError::error(
                                TypeErrorCode::T104,
                                format!("type `{resolved}` has no fields"),
                                field.span,
                            )
                            .label(object.span, format!("has type `{resolved}`"))
                            .build(),
                        );
                        Type::Error
                    }
                }
            }

            // ── Index access ────────────────────────────────────
            ExprKind::Index { object, index } => {
                let obj_ty = self.check_expr(object);
                let idx_ty = self.check_expr(index);
                let resolved = self.infer.resolve(&obj_ty);
                match resolved {
                    Type::List(elem) => {
                        if let Err(e) = self.infer.unify(&idx_ty, &Type::Int, index.span) {
                            self.errors.push(e);
                        }
                        *elem
                    }
                    Type::Map(key, val) => {
                        if let Err(e) = self.infer.unify(&idx_ty, &key, index.span) {
                            self.errors.push(e);
                        }
                        *val
                    }
                    Type::Error => Type::Error,
                    _ => {
                        self.errors.push(
                            TypeError::error(
                                TypeErrorCode::T116,
                                format!("type `{resolved}` is not indexable"),
                                expr.span,
                            )
                            .label(object.span, format!("has type `{resolved}`"))
                            .build(),
                        );
                        Type::Error
                    }
                }
            }

            // ── Function call ───────────────────────────────────
            ExprKind::Call { callee, args, .. } => {
                let callee_ty = self.check_expr(callee);
                let resolved = self.infer.resolve(&callee_ty);
                match resolved {
                    Type::Function(ft) => {
                        if ft.params.len() != args.len() {
                            self.errors.push(
                                TypeError::error(
                                    TypeErrorCode::T114,
                                    format!(
                                        "expected {} argument{}, found {}",
                                        ft.params.len(),
                                        if ft.params.len() == 1 { "" } else { "s" },
                                        args.len()
                                    ),
                                    expr.span,
                                )
                                .build(),
                            );
                            return *ft.return_type;
                        }
                        for (arg, param_ty) in args.iter().zip(&ft.params) {
                            let arg_ty = self.check_expr(arg);
                            if let Err(e) = self.infer.unify(&arg_ty, param_ty, arg.span) {
                                self.errors.push(e);
                            }
                        }
                        *ft.return_type
                    }
                    Type::Error => {
                        for arg in args {
                            self.check_expr(arg);
                        }
                        Type::Error
                    }
                    Type::Var(_) => {
                        // Unknown callable — check args and produce a fresh return type.
                        let mut arg_types = Vec::new();
                        for arg in args {
                            arg_types.push(self.check_expr(arg));
                        }
                        let ret = self.infer.fresh();
                        let func_ty = Type::Function(FunctionType {
                            params: arg_types,
                            return_type: Box::new(ret.clone()),
                        });
                        if let Err(e) = self.infer.unify(&callee_ty, &func_ty, callee.span) {
                            self.errors.push(e);
                        }
                        ret
                    }
                    _ => {
                        self.errors.push(
                            TypeError::error(
                                TypeErrorCode::T115,
                                format!("`{resolved}` is not callable"),
                                callee.span,
                            )
                            .label(callee.span, format!("has type `{resolved}`"))
                            .build(),
                        );
                        for arg in args {
                            self.check_expr(arg);
                        }
                        Type::Error
                    }
                }
            }

            // ── If expression ───────────────────────────────────
            ExprKind::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond_ty = self.check_expr(condition);
                self.expect_bool(&cond_ty, condition.span);

                self.enter_scope(ScopeKind::Block);
                let then_ty = self.check_block(then_block);
                self.exit_scope();

                if let Some(eb) = else_block {
                    let else_ty = match eb {
                        ElseBranch::ElseBlock(block) => {
                            self.enter_scope(ScopeKind::Block);
                            let ty = self.check_block(block);
                            self.exit_scope();
                            ty
                        }
                        ElseBranch::ElseIf(elif) => self.check_expr(elif),
                    };
                    match self.infer.unify(&then_ty, &else_ty, expr.span) {
                        Ok(unified) => unified,
                        Err(_) => {
                            self.errors.push(
                                TypeError::error(
                                    TypeErrorCode::T112,
                                    format!(
                                        "incompatible types in if branches: `{then_ty}` vs `{else_ty}`"
                                    ),
                                    expr.span,
                                )
                                .build(),
                            );
                            Type::Error
                        }
                    }
                } else {
                    // No else — if expression evaluates to Unit.
                    Type::Unit
                }
            }

            // ── Match expression ────────────────────────────────
            ExprKind::Match { scrutinee, arms } => {
                let scrut_ty = self.check_expr(scrutinee);
                let _ = scrut_ty; // used for pattern checking

                if arms.is_empty() {
                    return Type::Unit;
                }

                let mut result_ty: Option<Type> = None;
                for arm in arms {
                    self.enter_scope(ScopeKind::Block);
                    // Check pattern introduces bindings but we skip deep pattern type checking for now.
                    let arm_ty = self.check_expr(&arm.body);
                    self.exit_scope();

                    match &result_ty {
                        None => result_ty = Some(arm_ty),
                        Some(prev) => {
                            match self.infer.unify(prev, &arm_ty, arm.span) {
                                Ok(unified) => result_ty = Some(unified),
                                Err(_) => {
                                    self.errors.push(
                                        TypeError::error(
                                            TypeErrorCode::T119,
                                            format!(
                                                "incompatible match arm types: `{prev}` vs `{arm_ty}`"
                                            ),
                                            arm.span,
                                        )
                                        .build(),
                                    );
                                }
                            }
                        }
                    }
                }

                result_ty.unwrap_or(Type::Unit)
            }

            // ── Arrow function ──────────────────────────────────
            ExprKind::ArrowFunction {
                params,
                return_type,
                body,
            } => {
                self.enter_scope(ScopeKind::Function);

                let mut param_types = Vec::new();
                for param in params {
                    let ty = if let Some(ann) = &param.ty {
                        self.resolve_type_annotation(ann)
                    } else {
                        self.infer.fresh()
                    };
                    self.define_in_current(&param.name.name, ty.clone(), false, param.name.span);
                    param_types.push(ty);
                }

                let ret_ty = if let Some(ann) = return_type {
                    self.resolve_type_annotation(ann)
                } else {
                    self.infer.fresh()
                };

                let prev_return = self.return_type.take();
                self.return_type = Some(ret_ty.clone());

                let body_ty = match body {
                    ArrowBody::Expr(e) => self.check_expr(e),
                    ArrowBody::Block(b) => self.check_block(b),
                };

                if let Err(e) = self.infer.unify(&body_ty, &ret_ty, expr.span) {
                    self.errors.push(e);
                }

                self.return_type = prev_return;
                self.exit_scope();

                Type::Function(FunctionType {
                    params: param_types,
                    return_type: Box::new(ret_ty),
                })
            }

            // ── Object literal ──────────────────────────────────
            ExprKind::ObjectLiteral { fields } => {
                let mut type_fields = Vec::new();
                for field in fields {
                    let ty = if let Some(value) = &field.value {
                        self.check_expr(value)
                    } else {
                        // Shorthand — look up the variable.
                        if let Some((_, sym)) =
                            self.scopes.lookup(self.current_scope, &field.key.name)
                        {
                            sym.ty.clone()
                        } else {
                            Type::Error
                        }
                    };
                    type_fields.push((field.key.name.clone(), ty));
                }
                Type::Struct(StructType {
                    name: None,
                    fields: type_fields,
                    type_params: vec![],
                })
            }

            // ── List literal ────────────────────────────────────
            ExprKind::ListLiteral { elements } => {
                if elements.is_empty() {
                    let elem = self.infer.fresh();
                    return Type::List(Box::new(elem));
                }
                let first_ty = self.check_expr(&elements[0]);
                for elem in &elements[1..] {
                    let elem_ty = self.check_expr(elem);
                    if let Err(e) = self.infer.unify(&elem_ty, &first_ty, elem.span) {
                        self.errors.push(e);
                    }
                }
                Type::List(Box::new(first_ty))
            }

            // ── Interpolated string ─────────────────────────────
            ExprKind::InterpolatedString { parts } => {
                for part in parts {
                    if let InterpolatedPart::Expr(e) = part {
                        self.check_expr(e);
                    }
                }
                Type::String
            }

            // ── Block expression ────────────────────────────────
            ExprKind::Block(block) => {
                self.enter_scope(ScopeKind::Block);
                let ty = self.check_block(block);
                self.exit_scope();
                ty
            }

            // ── Grouped expression ──────────────────────────────
            ExprKind::Grouped(inner) => self.check_expr(inner),
        }
    }

    // ── Binary Operator Type Rules ──────────────────────────────

    fn check_binary_op(
        &mut self,
        op: BinaryOp,
        left: &Type,
        right: &Type,
        span: Span,
    ) -> Type {
        let left = self.infer.resolve(left);
        let right = self.infer.resolve(right);

        match op {
            // Arithmetic: both numeric, same type.
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                // String concatenation for Add.
                if op == BinaryOp::Add
                    && matches!((&left, &right), (Type::String, Type::String))
                {
                    return Type::String;
                }
                match (&left, &right) {
                    (Type::Int, Type::Int) | (Type::Time, Type::Int) | (Type::Int, Type::Time) | (Type::Time, Type::Time) => Type::Int,
                    (Type::Float, Type::Float) => Type::Float,
                    (Type::Error, _) | (_, Type::Error) => Type::Error,
                    _ => {
                        self.errors.push(
                            TypeError::error(
                                TypeErrorCode::T111,
                                format!(
                                    "cannot apply `{op}` to `{left}` and `{right}`",
                                    op = binary_op_symbol(op)
                                ),
                                span,
                            )
                            .build(),
                        );
                        Type::Error
                    }
                }
            }

            // Comparison: same type, returns bool.
            BinaryOp::Eq | BinaryOp::NotEq | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => {
                match (&left, &right) {
                    (Type::Error, _) | (_, Type::Error) => Type::Bool,
                    _ => {
                        if self.infer.unify(&left, &right, span).is_err() {
                            self.errors.push(
                                TypeError::error(
                                    TypeErrorCode::T111,
                                    format!(
                                        "cannot compare `{left}` and `{right}` with `{}`",
                                        binary_op_symbol(op)
                                    ),
                                    span,
                                )
                                .build(),
                            );
                        }
                        Type::Bool
                    }
                }
            }

            // Logical: both bool.
            BinaryOp::And | BinaryOp::Or => {
                self.expect_bool(&left, span);
                self.expect_bool(&right, span);
                Type::Bool
            }
        }
    }

    // ── Type Annotation Resolution ──────────────────────────────

    fn resolve_type_annotation(&mut self, ann: &TypeAnnotation) -> Type {
        match &ann.kind {
            TypeKind::Named(ident) => self.resolve_named_type(&ident.name, ann.span),
            TypeKind::Generic { name, args } => {
                let resolved_args: Vec<Type> = args
                    .iter()
                    .map(|a| self.resolve_type_annotation(a))
                    .collect();
                self.resolve_generic_type(&name.name, resolved_args, ann.span)
            }
            TypeKind::Function { params, return_type } => {
                let param_types: Vec<Type> = params
                    .iter()
                    .map(|p| self.resolve_type_annotation(p))
                    .collect();
                let ret = self.resolve_type_annotation(return_type);
                Type::Function(FunctionType {
                    params: param_types,
                    return_type: Box::new(ret),
                })
            }
        }
    }

    fn resolve_named_type(&self, name: &str, span: Span) -> Type {
        match name {
            "int" => Type::Int,
            "float" => Type::Float,
            "string" => Type::String,
            "bool" => Type::Bool,
            "time" => Type::Time,
            _ => {
                // Check if it's a resolved user-defined type.
                if let Some(st) = self.type_defs.get(name) {
                    Type::Struct(st.clone())
                } else if self.scopes.lookup(self.current_scope, name).is_some() {
                    // Known symbol but not a resolved typedef — use placeholder.
                    Type::Struct(StructType {
                        name: Some(name.to_string()),
                        fields: vec![],
                        type_params: vec![],
                    })
                } else {
                    let _ = span;
                    Type::Error
                }
            }
        }
    }

    fn resolve_generic_type(&mut self, name: &str, args: Vec<Type>, _span: Span) -> Type {
        match name {
            "Option" => {
                if args.len() == 1 {
                    Type::Option(Box::new(args.into_iter().next().unwrap()))
                } else {
                    Type::Error
                }
            }
            "Result" => {
                if args.len() == 2 {
                    let mut iter = args.into_iter();
                    Type::Result(
                        Box::new(iter.next().unwrap()),
                        Box::new(iter.next().unwrap()),
                    )
                } else {
                    Type::Error
                }
            }
            "List" => {
                if args.len() == 1 {
                    Type::List(Box::new(args.into_iter().next().unwrap()))
                } else {
                    Type::Error
                }
            }
            "Map" => {
                if args.len() == 2 {
                    let mut iter = args.into_iter();
                    Type::Map(
                        Box::new(iter.next().unwrap()),
                        Box::new(iter.next().unwrap()),
                    )
                } else {
                    Type::Error
                }
            }
            _ => {
                // User-defined generic type.
                Type::Struct(StructType {
                    name: Some(name.to_string()),
                    fields: vec![],
                    type_params: args,
                })
            }
        }
    }

    // ── Scope Helpers ───────────────────────────────────────────

    fn enter_scope(&mut self, kind: ScopeKind) {
        let parent = self.current_scope;
        self.current_scope = self.scopes.create(kind, Some(parent));
    }

    fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes.get(self.current_scope).parent {
            self.current_scope = parent;
        }
    }

    fn define_in_current(&mut self, name: &str, ty: Type, mutable: bool, span: Span) {
        self.scopes.define(
            self.current_scope,
            name.to_string(),
            Symbol {
                kind: SymbolKind::Variable,
                ty,
                mutable,
                defined_at: span,
                used: false,
            },
        );
    }

    /// Define or update a symbol in the current scope.
    /// If the symbol already exists in the current scope, updates its type.
    /// Otherwise, inserts it. This handles the resolver/checker scope mismatch:
    /// - Top-level vars are already in the module scope (update their type).
    /// - Local vars inside functions are NOT in the checker's scope (define them).
    fn define_or_update(&mut self, name: &str, ty: Type, mutable: bool, span: Span) {
        self.scopes.upsert(
            self.current_scope,
            name.to_string(),
            Symbol {
                kind: SymbolKind::Variable,
                ty,
                mutable,
                defined_at: span,
                used: false,
            },
        );
    }

    fn update_symbol_type(&mut self, name: &str, ty: Type) {
        // Walk up from current scope to find the symbol and update its type.
        // Also check the current scope first (checker may have defined it there).
        let mut scope_id = self.current_scope;
        loop {
            let scope = self.scopes.get_mut(scope_id);
            if let Some(sym) = scope.symbols.get_mut(name) {
                sym.ty = ty;
                return;
            }
            if let Some(parent) = scope.parent {
                scope_id = parent;
            } else {
                // Not found anywhere — define in current scope as fallback.
                self.scopes.upsert(
                    self.current_scope,
                    name.to_string(),
                    Symbol {
                        kind: SymbolKind::Variable,
                        ty,
                        mutable: false,
                        defined_at: zehd_tome::Span::new(0, 0),
                        used: false,
                    },
                );
                return;
            }
        }
    }

    fn expect_bool(&mut self, ty: &Type, span: Span) {
        let resolved = self.infer.resolve(ty);
        match resolved {
            Type::Bool | Type::Error => {}
            Type::Var(_) => {
                if let Err(e) = self.infer.unify(ty, &Type::Bool, span) {
                    self.errors.push(e);
                }
            }
            _ => {
                self.errors.push(
                    TypeError::error(
                        TypeErrorCode::T113,
                        format!("expected `bool`, found `{resolved}`"),
                        span,
                    )
                    .build(),
                );
            }
        }
    }
}

/// Result of the checker pass.
pub struct CheckerResult {
    pub types: TypeTable,
    pub scopes: ScopeArena,
    pub errors: Vec<TypeError>,
}

fn binary_op_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::Eq => "==",
        BinaryOp::NotEq => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Gt => ">",
        BinaryOp::LtEq => "<=",
        BinaryOp::GtEq => ">=",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
    }
}
