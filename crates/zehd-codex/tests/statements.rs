mod helpers;

use helpers::*;
use zehd_codex::ast::*;

#[test]
fn return_with_value() {
    let result = parse_ok("fn test() { return 42; }");
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    assert_eq!(func.body.stmts.len(), 1);
    match &func.body.stmts[0].kind {
        StmtKind::Return(r) => {
            assert!(matches!(
                r.value.as_ref().unwrap().kind,
                ExprKind::IntLiteral(42)
            ));
        }
        other => panic!("expected Return, got {:?}", other),
    }
}

#[test]
fn return_without_value() {
    let result = parse_ok("fn test() { return; }");
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    match &func.body.stmts[0].kind {
        StmtKind::Return(r) => {
            assert!(r.value.is_none());
        }
        other => panic!("expected Return, got {:?}", other),
    }
}

#[test]
fn break_statement() {
    let result = parse_ok("fn test() { break; }");
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    assert!(matches!(func.body.stmts[0].kind, StmtKind::Break));
}

#[test]
fn continue_statement() {
    let result = parse_ok("fn test() { continue; }");
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    assert!(matches!(func.body.stmts[0].kind, StmtKind::Continue));
}

#[test]
fn for_loop() {
    let result = parse_ok("fn test() { for item in items { x; } }");
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    match &func.body.stmts[0].kind {
        StmtKind::For(f) => {
            assert_eq!(f.binding.name, "item");
            assert!(matches!(&f.iterable.kind, ExprKind::Ident(id) if id.name == "items"));
        }
        other => panic!("expected For, got {:?}", other),
    }
}

#[test]
fn while_loop() {
    let result = parse_ok("fn test() { while x < 10 { x; } }");
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    match &func.body.stmts[0].kind {
        StmtKind::While(w) => {
            assert!(matches!(&w.condition.kind, ExprKind::Binary { op: BinaryOp::Lt, .. }));
        }
        other => panic!("expected While, got {:?}", other),
    }
}

#[test]
fn assignment() {
    let result = parse_ok("fn test() { x = 42; }");
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    match &func.body.stmts[0].kind {
        StmtKind::Assignment(a) => {
            assert!(matches!(&a.target.kind, ExprKind::Ident(id) if id.name == "x"));
            assert!(matches!(a.value.kind, ExprKind::IntLiteral(42)));
        }
        other => panic!("expected Assignment, got {:?}", other),
    }
}

#[test]
fn field_assignment() {
    let result = parse_ok("fn test() { self.response.status = 200; }");
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    match &func.body.stmts[0].kind {
        StmtKind::Assignment(a) => {
            assert!(matches!(&a.target.kind, ExprKind::FieldAccess { .. }));
            assert!(matches!(a.value.kind, ExprKind::IntLiteral(200)));
        }
        other => panic!("expected Assignment, got {:?}", other),
    }
}

#[test]
fn block_tail_expression() {
    let result = parse_ok("fn test() { 42 }");
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    assert!(func.body.stmts.is_empty());
    assert!(matches!(
        func.body.tail_expr.as_ref().unwrap().kind,
        ExprKind::IntLiteral(42)
    ));
}

#[test]
fn block_statements_then_tail() {
    let result = parse_ok("fn test() { let x = 1; x }");
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    assert_eq!(func.body.stmts.len(), 1);
    assert!(matches!(func.body.stmts[0].kind, StmtKind::VarDecl(_)));
    assert!(matches!(
        &func.body.tail_expr.as_ref().unwrap().kind,
        ExprKind::Ident(id) if id.name == "x"
    ));
}

#[test]
fn var_decl_in_block() {
    let result = parse_ok("fn test() { let x = 42; const y = x; }");
    let func = match &result.program.items[0].kind {
        ItemKind::Function(f) => f,
        other => panic!("expected Function, got {:?}", other),
    };
    assert_eq!(func.body.stmts.len(), 2);
    assert!(matches!(func.body.stmts[0].kind, StmtKind::VarDecl(_)));
    assert!(matches!(func.body.stmts[1].kind, StmtKind::VarDecl(_)));
}
