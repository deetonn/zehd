mod helpers;
use helpers::*;
use zehd_codex::ast::*;

// ── Constant Folding ────────────────────────────────────────────

#[test]
fn fold_int_addition() {
    let result = check_ok("const x = 1 + 2;");
    let program = result.optimized_program.as_ref().unwrap();
    // Find the var decl initializer.
    if let ItemKind::VarDecl(v) = &program.items[0].kind {
        if let Some(init) = &v.initializer {
            assert!(
                matches!(init.kind, ExprKind::IntLiteral(3)),
                "expected folded to 3, got {:?}",
                init.kind
            );
        }
    }
}

#[test]
fn fold_int_multiplication() {
    let result = check_ok("const x = 3 * 4;");
    let program = result.optimized_program.as_ref().unwrap();
    if let ItemKind::VarDecl(v) = &program.items[0].kind {
        if let Some(init) = &v.initializer {
            assert!(
                matches!(init.kind, ExprKind::IntLiteral(12)),
                "expected 12, got {:?}",
                init.kind
            );
        }
    }
}

#[test]
fn fold_nested_arithmetic() {
    let result = check_ok("const x = 1 + 2 * 3;");
    let program = result.optimized_program.as_ref().unwrap();
    if let ItemKind::VarDecl(v) = &program.items[0].kind {
        if let Some(init) = &v.initializer {
            assert!(
                matches!(init.kind, ExprKind::IntLiteral(7)),
                "expected 7, got {:?}",
                init.kind
            );
        }
    }
}

#[test]
fn fold_boolean_not() {
    let result = check_ok("const x = !true;");
    let program = result.optimized_program.as_ref().unwrap();
    if let ItemKind::VarDecl(v) = &program.items[0].kind {
        if let Some(init) = &v.initializer {
            assert!(
                matches!(init.kind, ExprKind::BoolLiteral(false)),
                "expected false, got {:?}",
                init.kind
            );
        }
    }
}

#[test]
fn fold_boolean_and() {
    let result = check_ok("const x = true && false;");
    let program = result.optimized_program.as_ref().unwrap();
    if let ItemKind::VarDecl(v) = &program.items[0].kind {
        if let Some(init) = &v.initializer {
            assert!(
                matches!(init.kind, ExprKind::BoolLiteral(false)),
                "expected false, got {:?}",
                init.kind
            );
        }
    }
}

#[test]
fn fold_int_comparison() {
    let result = check_ok("const x = 1 < 2;");
    let program = result.optimized_program.as_ref().unwrap();
    if let ItemKind::VarDecl(v) = &program.items[0].kind {
        if let Some(init) = &v.initializer {
            assert!(
                matches!(init.kind, ExprKind::BoolLiteral(true)),
                "expected true, got {:?}",
                init.kind
            );
        }
    }
}

#[test]
fn fold_string_concat() {
    let result = check_ok("const x = \"hello\" + \" world\";");
    let program = result.optimized_program.as_ref().unwrap();
    if let ItemKind::VarDecl(v) = &program.items[0].kind {
        if let Some(init) = &v.initializer {
            match &init.kind {
                ExprKind::StringLiteral(s) => assert_eq!(s, "hello world"),
                other => panic!("expected string literal, got {:?}", other),
            }
        }
    }
}

#[test]
fn fold_float_addition() {
    let result = check_ok("const x = 1.5 + 2.5;");
    let program = result.optimized_program.as_ref().unwrap();
    if let ItemKind::VarDecl(v) = &program.items[0].kind {
        if let Some(init) = &v.initializer {
            assert!(
                matches!(init.kind, ExprKind::FloatLiteral(f) if f == 4.0),
                "expected 4.0, got {:?}",
                init.kind
            );
        }
    }
}

// ── Const Inlining ──────────────────────────────────────────────

#[test]
fn inline_const_reference() {
    let source = r#"
        const PORT = 8080;
        PORT;
    "#;
    let result = check_ok(source);
    let program = result.optimized_program.as_ref().unwrap();
    // The expression statement (PORT;) should be inlined to 8080.
    if let ItemKind::ExprStmt(es) = &program.items[1].kind {
        assert!(
            matches!(es.expr.kind, ExprKind::IntLiteral(8080)),
            "expected inlined 8080, got {:?}",
            es.expr.kind
        );
    }
}

// ── If Simplification ───────────────────────────────────────────

#[test]
fn simplify_if_true() {
    let source = r#"
        let x = if true { 1 } else { 2 };
    "#;
    let result = check_ok(source);
    let program = result.optimized_program.as_ref().unwrap();
    if let ItemKind::VarDecl(v) = &program.items[0].kind {
        if let Some(init) = &v.initializer {
            // Should be simplified to a block containing 1.
            assert!(
                matches!(init.kind, ExprKind::Block(_)),
                "expected block, got {:?}",
                init.kind
            );
        }
    }
}

#[test]
fn simplify_if_false() {
    let source = r#"
        let x = if false { 1 } else { 2 };
    "#;
    let result = check_ok(source);
    let program = result.optimized_program.as_ref().unwrap();
    if let ItemKind::VarDecl(v) = &program.items[0].kind {
        if let Some(init) = &v.initializer {
            // Should be simplified to the else block.
            assert!(
                matches!(init.kind, ExprKind::Block(_)),
                "expected block, got {:?}",
                init.kind
            );
        }
    }
}
