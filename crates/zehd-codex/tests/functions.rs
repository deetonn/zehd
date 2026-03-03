mod helpers;

use helpers::*;
use zehd_codex::ast::*;

#[test]
fn simple_function() {
    let item = parse_single_item(
        "fn greet(name: string): string {
            return name;
        }",
    );
    match &item.kind {
        ItemKind::Function(f) => {
            assert_eq!(f.name.name, "greet");
            assert_eq!(f.params.len(), 1);
            assert_eq!(f.params[0].name.name, "name");
            assert!(
                matches!(&f.params[0].ty.as_ref().unwrap().kind, TypeKind::Named(id) if id.name == "string")
            );
            assert!(matches!(
                &f.return_type.as_ref().unwrap().kind,
                TypeKind::Named(id) if id.name == "string"
            ));
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn function_no_params() {
    let item = parse_single_item("fn hello() { return 42; }");
    match &item.kind {
        ItemKind::Function(f) => {
            assert_eq!(f.name.name, "hello");
            assert!(f.params.is_empty());
            assert!(f.return_type.is_none());
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn function_multiple_params() {
    let item = parse_single_item("fn add(a: int, b: int): int { return a; }");
    match &item.kind {
        ItemKind::Function(f) => {
            assert_eq!(f.params.len(), 2);
            assert_eq!(f.params[0].name.name, "a");
            assert_eq!(f.params[1].name.name, "b");
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn function_inferred_return() {
    let item = parse_single_item(
        "fn double(x: int) {
            return x * 2;
        }",
    );
    match &item.kind {
        ItemKind::Function(f) => {
            assert_eq!(f.name.name, "double");
            assert!(f.return_type.is_none());
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn function_with_trailing_comma() {
    let item = parse_single_item("fn add(a: int, b: int,): int { return a; }");
    match &item.kind {
        ItemKind::Function(f) => {
            assert_eq!(f.params.len(), 2);
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn function_untyped_params() {
    let item = parse_single_item("fn transform(req) { return req; }");
    match &item.kind {
        ItemKind::Function(f) => {
            assert_eq!(f.params.len(), 1);
            assert_eq!(f.params[0].name.name, "req");
            assert!(f.params[0].ty.is_none());
        }
        other => panic!("expected Function, got {:?}", other),
    }
}
