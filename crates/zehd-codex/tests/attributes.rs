mod helpers;

use helpers::*;
use zehd_codex::ast::*;

#[test]
fn type_with_field_attributes() {
    let result = parse_ok(
        "type CreateUser {
            #[validate.min(1)]
            #[validate.max(100)]
            name: string;
            #[validate.range(18, 150)]
            age: int;
        }",
    );
    let item = &result.program.items[0];
    match &item.kind {
        ItemKind::TypeDef(td) => {
            assert_eq!(td.fields.len(), 2);

            // name field has 2 attributes
            assert_eq!(td.fields[0].attributes.len(), 2);
            assert_eq!(td.fields[0].attributes[0].path[0].name, "validate");
            assert_eq!(td.fields[0].attributes[0].path[1].name, "min");
            assert_eq!(td.fields[0].attributes[0].args.len(), 1);
            assert!(matches!(
                td.fields[0].attributes[0].args[0].kind,
                ExprKind::IntLiteral(1)
            ));

            assert_eq!(td.fields[0].attributes[1].path[1].name, "max");
            assert!(matches!(
                td.fields[0].attributes[1].args[0].kind,
                ExprKind::IntLiteral(100)
            ));

            // age field has 1 attribute with 2 args
            assert_eq!(td.fields[1].attributes.len(), 1);
            assert_eq!(td.fields[1].attributes[0].path[1].name, "range");
            assert_eq!(td.fields[1].attributes[0].args.len(), 2);
        }
        other => panic!("expected TypeDef, got {:?}", other),
    }
}

#[test]
fn attribute_no_args() {
    let result = parse_ok(
        "type User {
            #[validate.email]
            email: string;
        }",
    );
    let item = &result.program.items[0];
    match &item.kind {
        ItemKind::TypeDef(td) => {
            assert_eq!(td.fields[0].attributes.len(), 1);
            assert_eq!(td.fields[0].attributes[0].path[0].name, "validate");
            assert_eq!(td.fields[0].attributes[0].path[1].name, "email");
            assert!(td.fields[0].attributes[0].args.is_empty());
        }
        other => panic!("expected TypeDef, got {:?}", other),
    }
}

#[test]
fn attribute_with_named_arg() {
    // message="error" is parsed as Binary(Eq, Ident, String) in the arg list
    let result = parse_ok(
        "type User {
            #[validate.fail(message=\"error\")]
            name: string;
        }",
    );
    let item = &result.program.items[0];
    match &item.kind {
        ItemKind::TypeDef(td) => {
            let attr = &td.fields[0].attributes[0];
            assert_eq!(attr.path[1].name, "fail");
            assert_eq!(attr.args.len(), 1);
            // message="error" → Binary(Eq, Ident("message"), StringLiteral("error"))
            match &attr.args[0].kind {
                ExprKind::Binary { op, left, right } => {
                    assert_eq!(*op, BinaryOp::Eq);
                    assert!(matches!(&left.kind, ExprKind::Ident(id) if id.name == "message"));
                    assert!(matches!(&right.kind, ExprKind::StringLiteral(s) if s == "error"));
                }
                other => panic!("expected Binary Eq, got {:?}", other),
            }
        }
        other => panic!("expected TypeDef, got {:?}", other),
    }
}

#[test]
fn multiple_attributes_on_field() {
    let result = parse_ok(
        "type User {
            #[validate.optional]
            #[validate.max(500)]
            bio: string;
        }",
    );
    let item = &result.program.items[0];
    match &item.kind {
        ItemKind::TypeDef(td) => {
            assert_eq!(td.fields[0].attributes.len(), 2);
        }
        other => panic!("expected TypeDef, got {:?}", other),
    }
}
