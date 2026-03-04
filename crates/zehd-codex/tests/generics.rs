mod helpers;

use helpers::*;
use zehd_codex::ast::*;

#[test]
fn generic_function_single_type_param() {
    let item = parse_single_item("fn identity<T>(x: T): T { x }");
    match &item.kind {
        ItemKind::Function(f) => {
            assert_eq!(f.name.name, "identity");
            assert_eq!(f.type_params.len(), 1);
            assert_eq!(f.type_params[0].name, "T");
            assert_eq!(f.params.len(), 1);
            assert_eq!(f.params[0].name.name, "x");
            assert!(matches!(
                &f.params[0].ty.as_ref().unwrap().kind,
                TypeKind::Named(id) if id.name == "T"
            ));
            assert!(matches!(
                &f.return_type.as_ref().unwrap().kind,
                TypeKind::Named(id) if id.name == "T"
            ));
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn generic_function_multiple_type_params() {
    let item = parse_single_item("fn swap<A, B>(a: A, b: B): B { b }");
    match &item.kind {
        ItemKind::Function(f) => {
            assert_eq!(f.name.name, "swap");
            assert_eq!(f.type_params.len(), 2);
            assert_eq!(f.type_params[0].name, "A");
            assert_eq!(f.type_params[1].name, "B");
        }
        other => panic!("expected Function, got {:?}", other),
    }
}

#[test]
fn non_generic_function_backward_compat() {
    let item = parse_single_item("fn plain(): int { 1 }");
    match &item.kind {
        ItemKind::Function(f) => {
            assert_eq!(f.name.name, "plain");
            assert!(f.type_params.is_empty());
        }
        other => panic!("expected Function, got {:?}", other),
    }
}
