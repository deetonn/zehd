mod helpers;

use helpers::*;
use zehd_codex::ast::*;

#[test]
fn named_type_annotation() {
    let item = parse_single_item("let x: string;");
    match &item.kind {
        ItemKind::VarDecl(v) => {
            match &v.ty.as_ref().unwrap().kind {
                TypeKind::Named(id) => assert_eq!(id.name, "string"),
                other => panic!("expected Named, got {:?}", other),
            }
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn generic_type_single_param() {
    let item = parse_single_item("let x: Option<int>;");
    match &item.kind {
        ItemKind::VarDecl(v) => {
            match &v.ty.as_ref().unwrap().kind {
                TypeKind::Generic { name, args } => {
                    assert_eq!(name.name, "Option");
                    assert_eq!(args.len(), 1);
                    assert!(matches!(&args[0].kind, TypeKind::Named(id) if id.name == "int"));
                }
                other => panic!("expected Generic, got {:?}", other),
            }
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn generic_type_multiple_params() {
    let item = parse_single_item("let x: Result<User, string>;");
    match &item.kind {
        ItemKind::VarDecl(v) => {
            match &v.ty.as_ref().unwrap().kind {
                TypeKind::Generic { name, args } => {
                    assert_eq!(name.name, "Result");
                    assert_eq!(args.len(), 2);
                    assert!(matches!(&args[0].kind, TypeKind::Named(id) if id.name == "User"));
                    assert!(matches!(&args[1].kind, TypeKind::Named(id) if id.name == "string"));
                }
                other => panic!("expected Generic, got {:?}", other),
            }
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn function_type_annotation() {
    let item = parse_single_item("let f: (int) => bool;");
    match &item.kind {
        ItemKind::VarDecl(v) => {
            match &v.ty.as_ref().unwrap().kind {
                TypeKind::Function {
                    params,
                    return_type,
                } => {
                    assert_eq!(params.len(), 1);
                    assert!(matches!(&params[0].kind, TypeKind::Named(id) if id.name == "int"));
                    assert!(
                        matches!(&return_type.kind, TypeKind::Named(id) if id.name == "bool")
                    );
                }
                other => panic!("expected Function type, got {:?}", other),
            }
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn function_type_multiple_params() {
    let item = parse_single_item("let f: (int, string) => bool;");
    match &item.kind {
        ItemKind::VarDecl(v) => {
            match &v.ty.as_ref().unwrap().kind {
                TypeKind::Function {
                    params,
                    return_type,
                } => {
                    assert_eq!(params.len(), 2);
                    assert!(matches!(&params[0].kind, TypeKind::Named(id) if id.name == "int"));
                    assert!(
                        matches!(&params[1].kind, TypeKind::Named(id) if id.name == "string")
                    );
                    assert!(
                        matches!(&return_type.kind, TypeKind::Named(id) if id.name == "bool")
                    );
                }
                other => panic!("expected Function type, got {:?}", other),
            }
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}

#[test]
fn type_definition_simple() {
    let item = parse_single_item(
        "type User {
            name: string;
            age: int;
        }",
    );
    match &item.kind {
        ItemKind::TypeDef(td) => {
            assert_eq!(td.name.name, "User");
            assert!(td.type_params.is_empty());
            assert_eq!(td.fields.len(), 2);
            assert_eq!(td.fields[0].name.name, "name");
            assert!(
                matches!(&td.fields[0].ty.kind, TypeKind::Named(id) if id.name == "string")
            );
            assert_eq!(td.fields[1].name.name, "age");
            assert!(
                matches!(&td.fields[1].ty.kind, TypeKind::Named(id) if id.name == "int")
            );
        }
        other => panic!("expected TypeDef, got {:?}", other),
    }
}

#[test]
fn type_definition_with_generics() {
    let item = parse_single_item(
        "type ApiResponse<T> {
            data: T;
            status: int;
        }",
    );
    match &item.kind {
        ItemKind::TypeDef(td) => {
            assert_eq!(td.name.name, "ApiResponse");
            assert_eq!(td.type_params.len(), 1);
            assert_eq!(td.type_params[0].name, "T");
            assert_eq!(td.fields.len(), 2);
        }
        other => panic!("expected TypeDef, got {:?}", other),
    }
}

#[test]
fn enum_definition() {
    let item = parse_single_item(
        "enum UserError {
            NotFound(string),
            Unauthorized,
            ValidationFailed(string),
        }",
    );
    match &item.kind {
        ItemKind::EnumDef(ed) => {
            assert_eq!(ed.name.name, "UserError");
            assert!(ed.type_params.is_empty());
            assert_eq!(ed.variants.len(), 3);
            assert_eq!(ed.variants[0].name.name, "NotFound");
            assert!(ed.variants[0].payload.is_some());
            assert_eq!(ed.variants[1].name.name, "Unauthorized");
            assert!(ed.variants[1].payload.is_none());
            assert_eq!(ed.variants[2].name.name, "ValidationFailed");
            assert!(ed.variants[2].payload.is_some());
        }
        other => panic!("expected EnumDef, got {:?}", other),
    }
}

#[test]
fn enum_with_generics() {
    let item = parse_single_item(
        "enum Result<T, E> {
            Ok(T),
            Err(E),
        }",
    );
    match &item.kind {
        ItemKind::EnumDef(ed) => {
            assert_eq!(ed.name.name, "Result");
            assert_eq!(ed.type_params.len(), 2);
            assert_eq!(ed.type_params[0].name, "T");
            assert_eq!(ed.type_params[1].name, "E");
            assert_eq!(ed.variants.len(), 2);
        }
        other => panic!("expected EnumDef, got {:?}", other),
    }
}

#[test]
fn nested_generic_type() {
    let item = parse_single_item("let x: Option<Result<int, string>>;");
    match &item.kind {
        ItemKind::VarDecl(v) => {
            match &v.ty.as_ref().unwrap().kind {
                TypeKind::Generic { name, args } => {
                    assert_eq!(name.name, "Option");
                    assert_eq!(args.len(), 1);
                    match &args[0].kind {
                        TypeKind::Generic { name, args } => {
                            assert_eq!(name.name, "Result");
                            assert_eq!(args.len(), 2);
                        }
                        other => panic!("expected inner Generic, got {:?}", other),
                    }
                }
                other => panic!("expected Generic, got {:?}", other),
            }
        }
        other => panic!("expected VarDecl, got {:?}", other),
    }
}
