use std::collections::HashMap;

use zehd_codex::ast::*;
use zehd_tome::Span;

use crate::error::*;
use crate::scope::ScopeArena;

// ── Optimizer ───────────────────────────────────────────────────

/// Performs optimization passes on a type-checked program.
///
/// Optimizations:
/// - Constant folding: evaluate compile-time-known expressions
/// - Dead code elimination: remove unreachable code
/// - Const inlining: replace const references with their values
pub struct Optimizer {
    /// const name → literal value (for inlining).
    const_values: HashMap<String, ExprKind>,
    pub warnings: Vec<TypeError>,
}

impl Optimizer {
    pub fn new() -> Self {
        Self {
            const_values: HashMap::new(),
            warnings: Vec::new(),
        }
    }

    /// Optimize a program in place. Returns warnings generated during optimization.
    pub fn optimize(mut self, program: &mut Program, _scopes: &ScopeArena) -> Vec<TypeError> {
        // Collect const values for inlining.
        self.collect_consts(&program.items);

        // Optimize all items.
        for item in &mut program.items {
            self.optimize_item(item);
        }

        self.warnings
    }

    // ── Const Collection ────────────────────────────────────────

    fn collect_consts(&mut self, items: &[Item]) {
        for item in items {
            if let ItemKind::VarDecl(v) = &item.kind {
                if !v.mutable {
                    if let Some(init) = &v.initializer {
                        if is_simple_literal(&init.kind) {
                            self.const_values.insert(v.name.name.clone(), init.kind.clone());
                        }
                    }
                }
            }
        }
    }

    // ── Item Optimization ───────────────────────────────────────

    fn optimize_item(&mut self, item: &mut Item) {
        match &mut item.kind {
            ItemKind::Function(f) => self.optimize_block(&mut f.body),
            ItemKind::VarDecl(v) => {
                if let Some(init) = &mut v.initializer {
                    self.optimize_expr(init);
                }
            }
            ItemKind::HttpBlock(h) => self.optimize_block(&mut h.body),
            ItemKind::InitBlock(i) => self.optimize_block(&mut i.body),
            ItemKind::ErrorHandler(e) => self.optimize_block(&mut e.body),
            ItemKind::ExprStmt(es) => self.optimize_expr(&mut es.expr),
            ItemKind::Import(_) | ItemKind::TypeDef(_) | ItemKind::EnumDef(_) => {}
        }
    }

    // ── Block Optimization ──────────────────────────────────────

    fn optimize_block(&mut self, block: &mut Block) {
        // Dead code elimination: remove statements after unconditional return.
        let mut truncate_at = None;
        for (i, stmt) in block.stmts.iter().enumerate() {
            if matches!(&stmt.kind, StmtKind::Return(_))
                && i + 1 < block.stmts.len()
            {
                self.warnings.push(
                    TypeError::warning(
                        TypeErrorCode::T150,
                        "unreachable code after return",
                        block.stmts[i + 1].span,
                    )
                    .label(stmt.span, "any code after this return is unreachable")
                    .build(),
                );
                truncate_at = Some(i + 1);
                break;
            }
        }
        if let Some(idx) = truncate_at {
            block.stmts.truncate(idx);
            block.tail_expr = None;
        }

        // Optimize remaining statements.
        for stmt in &mut block.stmts {
            self.optimize_stmt(stmt);
        }

        if let Some(tail) = &mut block.tail_expr {
            self.optimize_expr(tail);
        }
    }

    // ── Statement Optimization ──────────────────────────────────

    fn optimize_stmt(&mut self, stmt: &mut Stmt) {
        match &mut stmt.kind {
            StmtKind::VarDecl(v) => {
                if let Some(init) = &mut v.initializer {
                    self.optimize_expr(init);
                }
            }
            StmtKind::ExprStmt(es) => self.optimize_expr(&mut es.expr),
            StmtKind::Return(r) => {
                if let Some(val) = &mut r.value {
                    self.optimize_expr(val);
                }
            }
            StmtKind::For(f) => {
                self.optimize_expr(&mut f.iterable);
                self.optimize_block(&mut f.body);
            }
            StmtKind::While(w) => {
                self.optimize_expr(&mut w.condition);
                self.optimize_block(&mut w.body);
            }
            StmtKind::Assignment(a) => {
                self.optimize_expr(&mut a.value);
            }
            StmtKind::Break | StmtKind::Continue => {}
        }
    }

    // ── Expression Optimization ─────────────────────────────────

    fn optimize_expr(&mut self, expr: &mut Expr) {
        // First, recurse into children.
        match &mut expr.kind {
            ExprKind::Binary { left, right, .. } => {
                self.optimize_expr(left);
                self.optimize_expr(right);
            }
            ExprKind::Unary { operand, .. } => {
                self.optimize_expr(operand);
            }
            ExprKind::Call { callee, args, .. } => {
                self.optimize_expr(callee);
                for arg in args {
                    self.optimize_expr(arg);
                }
            }
            ExprKind::FieldAccess { object, .. } => self.optimize_expr(object),
            ExprKind::Index { object, index } => {
                self.optimize_expr(object);
                self.optimize_expr(index);
            }
            ExprKind::Try(inner) => self.optimize_expr(inner),
            ExprKind::If {
                condition,
                then_block,
                else_block,
            } => {
                self.optimize_expr(condition);
                self.optimize_block(then_block);
                if let Some(eb) = else_block {
                    match eb {
                        ElseBranch::ElseBlock(b) => self.optimize_block(b),
                        ElseBranch::ElseIf(e) => self.optimize_expr(e),
                    }
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                self.optimize_expr(scrutinee);
                for arm in arms {
                    self.optimize_expr(&mut arm.body);
                }
            }
            ExprKind::ArrowFunction { body, .. } => match body {
                ArrowBody::Expr(e) => self.optimize_expr(e),
                ArrowBody::Block(b) => self.optimize_block(b),
            },
            ExprKind::ObjectLiteral { fields } => {
                for field in fields {
                    if let Some(value) = &mut field.value {
                        self.optimize_expr(value);
                    }
                }
            }
            ExprKind::ListLiteral { elements } => {
                for elem in elements {
                    self.optimize_expr(elem);
                }
            }
            ExprKind::InterpolatedString { parts } => {
                for part in parts {
                    if let InterpolatedPart::Expr(e) = part {
                        self.optimize_expr(e);
                    }
                }
            }
            ExprKind::Block(block) => self.optimize_block(block),
            ExprKind::Grouped(inner) => self.optimize_expr(inner),
            ExprKind::EnumConstructor { arg, .. } => self.optimize_expr(arg),
            _ => {}
        }

        // Then, try to fold/inline this expression.
        self.try_fold(expr);
        self.try_inline(expr);
        self.try_simplify_if(expr);
    }

    // ── Constant Folding ────────────────────────────────────────

    fn try_fold(&mut self, expr: &mut Expr) {
        let span = expr.span;

        if let ExprKind::Binary { op, left, right } = &expr.kind {
            let op = *op;
            if let Some(folded) = fold_binary(op, &left.kind, &right.kind, span, expr.id) {
                *expr = folded;
            }
        }

        if let ExprKind::Unary { op, operand } = &expr.kind {
            let op = *op;
            if let Some(folded) = fold_unary(op, &operand.kind, span, expr.id) {
                *expr = folded;
            }
        }
    }

    // ── Const Inlining ──────────────────────────────────────────

    fn try_inline(&mut self, expr: &mut Expr) {
        if let ExprKind::Ident(ident) = &expr.kind {
            if let Some(value) = self.const_values.get(&ident.name) {
                expr.kind = value.clone();
            }
        }
    }

    // ── If Simplification ───────────────────────────────────────

    fn try_simplify_if(&mut self, expr: &mut Expr) {
        if let ExprKind::If {
            condition,
            then_block,
            else_block,
        } = &mut expr.kind
        {
            match &condition.kind {
                ExprKind::BoolLiteral(true) => {
                    // if true { a } else { b } → a
                    let block = std::mem::replace(
                        then_block,
                        Block {
                            stmts: vec![],
                            tail_expr: None,
                            span: expr.span,
                        },
                    );
                    expr.kind = ExprKind::Block(block);
                }
                ExprKind::BoolLiteral(false) => {
                    // if false { a } else { b } → b
                    if let Some(eb) = else_block.take() {
                        match eb {
                            ElseBranch::ElseBlock(block) => {
                                expr.kind = ExprKind::Block(block);
                            }
                            ElseBranch::ElseIf(elif) => {
                                *expr = *elif;
                            }
                        }
                    } else {
                        // if false { a } with no else → unit block
                        expr.kind = ExprKind::Block(Block {
                            stmts: vec![],
                            tail_expr: None,
                            span: expr.span,
                        });
                    }
                }
                _ => {}
            }
        }
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Folding Helpers ─────────────────────────────────────────────

fn is_simple_literal(kind: &ExprKind) -> bool {
    matches!(
        kind,
        ExprKind::IntLiteral(_)
            | ExprKind::FloatLiteral(_)
            | ExprKind::StringLiteral(_)
            | ExprKind::TimeLiteral(_)
            | ExprKind::BoolLiteral(_)
    )
}

fn fold_binary(op: BinaryOp, left: &ExprKind, right: &ExprKind, span: Span, id: NodeId) -> Option<Expr> {
    match (left, right) {
        // Int operations.
        (ExprKind::IntLiteral(a), ExprKind::IntLiteral(b)) => {
            let result = match op {
                BinaryOp::Add => Some(ExprKind::IntLiteral(a.wrapping_add(*b))),
                BinaryOp::Sub => Some(ExprKind::IntLiteral(a.wrapping_sub(*b))),
                BinaryOp::Mul => Some(ExprKind::IntLiteral(a.wrapping_mul(*b))),
                BinaryOp::Div => {
                    if *b != 0 {
                        Some(ExprKind::IntLiteral(a / b))
                    } else {
                        None
                    }
                }
                BinaryOp::Mod => {
                    if *b != 0 {
                        Some(ExprKind::IntLiteral(a % b))
                    } else {
                        None
                    }
                }
                BinaryOp::Eq => Some(ExprKind::BoolLiteral(a == b)),
                BinaryOp::NotEq => Some(ExprKind::BoolLiteral(a != b)),
                BinaryOp::Lt => Some(ExprKind::BoolLiteral(a < b)),
                BinaryOp::Gt => Some(ExprKind::BoolLiteral(a > b)),
                BinaryOp::LtEq => Some(ExprKind::BoolLiteral(a <= b)),
                BinaryOp::GtEq => Some(ExprKind::BoolLiteral(a >= b)),
                _ => None,
            };
            result.map(|kind| Expr { id, kind, span })
        }

        // Float operations.
        (ExprKind::FloatLiteral(a), ExprKind::FloatLiteral(b)) => {
            let result = match op {
                BinaryOp::Add => Some(ExprKind::FloatLiteral(a + b)),
                BinaryOp::Sub => Some(ExprKind::FloatLiteral(a - b)),
                BinaryOp::Mul => Some(ExprKind::FloatLiteral(a * b)),
                BinaryOp::Div => {
                    if *b != 0.0 {
                        Some(ExprKind::FloatLiteral(a / b))
                    } else {
                        None
                    }
                }
                BinaryOp::Eq => Some(ExprKind::BoolLiteral(a == b)),
                BinaryOp::NotEq => Some(ExprKind::BoolLiteral(a != b)),
                BinaryOp::Lt => Some(ExprKind::BoolLiteral(a < b)),
                BinaryOp::Gt => Some(ExprKind::BoolLiteral(a > b)),
                BinaryOp::LtEq => Some(ExprKind::BoolLiteral(a <= b)),
                BinaryOp::GtEq => Some(ExprKind::BoolLiteral(a >= b)),
                _ => None,
            };
            result.map(|kind| Expr { id, kind, span })
        }

        // Bool operations.
        (ExprKind::BoolLiteral(a), ExprKind::BoolLiteral(b)) => {
            let result = match op {
                BinaryOp::And => Some(ExprKind::BoolLiteral(*a && *b)),
                BinaryOp::Or => Some(ExprKind::BoolLiteral(*a || *b)),
                BinaryOp::Eq => Some(ExprKind::BoolLiteral(a == b)),
                BinaryOp::NotEq => Some(ExprKind::BoolLiteral(a != b)),
                _ => None,
            };
            result.map(|kind| Expr { id, kind, span })
        }

        // String concatenation.
        (ExprKind::StringLiteral(a), ExprKind::StringLiteral(b)) => {
            if op == BinaryOp::Add {
                let mut result = a.clone();
                result.push_str(b);
                Some(Expr {
                    id,
                    kind: ExprKind::StringLiteral(result),
                    span,
                })
            } else {
                None
            }
        }

        // Time literal folding (ms values).
        (ExprKind::TimeLiteral(a), ExprKind::TimeLiteral(b)) => {
            let result = match op {
                BinaryOp::Add => Some(ExprKind::IntLiteral((a + b) as i64)),
                BinaryOp::Sub => Some(ExprKind::IntLiteral((a.wrapping_sub(*b)) as i64)),
                BinaryOp::Mul => Some(ExprKind::IntLiteral((a * b) as i64)),
                _ => None,
            };
            result.map(|kind| Expr { id, kind, span })
        }

        _ => None,
    }
}

fn fold_unary(op: UnaryOp, operand: &ExprKind, span: Span, id: NodeId) -> Option<Expr> {
    match (op, operand) {
        (UnaryOp::Neg, ExprKind::IntLiteral(v)) => Some(Expr {
            id,
            kind: ExprKind::IntLiteral(-v),
            span,
        }),
        (UnaryOp::Neg, ExprKind::FloatLiteral(v)) => Some(Expr {
            id,
            kind: ExprKind::FloatLiteral(-v),
            span,
        }),
        (UnaryOp::Not, ExprKind::BoolLiteral(v)) => Some(Expr {
            id,
            kind: ExprKind::BoolLiteral(!v),
            span,
        }),
        _ => None,
    }
}
