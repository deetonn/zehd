mod helpers;

use helpers::*;
use zehd_codex::ast::*;

#[test]
fn let_without_type() {
    let item = parse_single_item("let x = 42;");
    match &item.kind {
        ItemKind::VarDecl(v) => {
            assert!(v.mutable);
            assert_eq!(v.name.name, "x");
            assert!(v.ty.is_none());
            assert!(matches!(
                v.initializer.as_ref().unwrap().kind,
                ExprKind::IntLiteral(42)
            ));
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn const_without_type() {
    let item = parse_single_item("const name = \"zehd\";");
    match &item.kind {
        ItemKind::VarDecl(v) => {
            assert!(!v.mutable);
            assert_eq!(v.name.name, "name");
            assert!(v.ty.is_none());
            assert!(matches!(
                &v.initializer.as_ref().unwrap().kind,
                ExprKind::StringLiteral(s) if s == "zehd"
            ));
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn let_with_type() {
    let item = parse_single_item("let count: int = 0;");
    match &item.kind {
        ItemKind::VarDecl(v) => {
            assert!(v.mutable);
            assert_eq!(v.name.name, "count");
            match &v.ty.as_ref().unwrap().kind {
                TypeKind::Named(id) => assert_eq!(id.name, "int"),
                other => panic!("expected Named type, got {:?}", other),
            }
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn const_with_generic_type() {
    let item = parse_single_item("const result: Option<string> = None;");
    match &item.kind {
        ItemKind::VarDecl(v) => {
            assert!(!v.mutable);
            assert_eq!(v.name.name, "result");
            match &v.ty.as_ref().unwrap().kind {
                TypeKind::Generic { name, args } => {
                    assert_eq!(name.name, "Option");
                    assert_eq!(args.len(), 1);
                    assert!(
                        matches!(&args[0].kind, TypeKind::Named(id) if id.name == "string")
                    );
                }
                other => panic!("expected Generic type, got {:?}", other),
            }
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn let_without_initializer() {
    let item = parse_single_item("let x: int;");
    match &item.kind {
        ItemKind::VarDecl(v) => {
            assert!(v.mutable);
            assert_eq!(v.name.name, "x");
            assert!(v.ty.is_some());
            assert!(v.initializer.is_none());
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn const_with_expression_initializer() {
    let item = parse_single_item("const timeout = 30s;");
    match &item.kind {
        ItemKind::VarDecl(v) => {
            assert!(!v.mutable);
            assert_eq!(v.name.name, "timeout");
            assert!(matches!(
                v.initializer.as_ref().unwrap().kind,
                ExprKind::TimeLiteral(30000)
            ));
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}
