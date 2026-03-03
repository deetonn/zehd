mod helpers;

use helpers::*;
use zehd_codex::ast::*;

#[test]
fn simple_interpolated_string() {
    let expr = parse_single_expr("$\"Hello, {name}!\";");
    match &expr.kind {
        ExprKind::InterpolatedString { parts } => {
            assert_eq!(parts.len(), 3);
            assert!(matches!(&parts[0], InterpolatedPart::Literal(s, _) if s == "Hello, "));
            match &parts[1] {
                InterpolatedPart::Expr(e) => {
                    assert!(matches!(&e.kind, ExprKind::Ident(id) if id.name == "name"));
                }
                other => panic!("expected Expr part, got {:?}", other),
            }
            assert!(matches!(&parts[2], InterpolatedPart::Literal(s, _) if s == "!"));
        }
        other => panic!("expected InterpolatedString, got {:?}", other),
    }
}

#[test]
fn interpolated_string_with_field_access() {
    let expr = parse_single_expr("$\"User {user.name}\";");
    match &expr.kind {
        ExprKind::InterpolatedString { parts } => {
            assert_eq!(parts.len(), 2);
            assert!(matches!(&parts[0], InterpolatedPart::Literal(s, _) if s == "User "));
            match &parts[1] {
                InterpolatedPart::Expr(e) => {
                    assert!(matches!(&e.kind, ExprKind::FieldAccess { .. }));
                }
                other => panic!("expected Expr part, got {:?}", other),
            }
        }
        other => panic!("expected InterpolatedString, got {:?}", other),
    }
}

#[test]
fn interpolated_string_no_expressions() {
    let expr = parse_single_expr("$\"plain text\";");
    match &expr.kind {
        ExprKind::InterpolatedString { parts } => {
            assert_eq!(parts.len(), 1);
            assert!(matches!(&parts[0], InterpolatedPart::Literal(s, _) if s == "plain text"));
        }
        other => panic!("expected InterpolatedString, got {:?}", other),
    }
}

#[test]
fn interpolated_string_multiple_exprs() {
    let expr = parse_single_expr("$\"{a} and {b}\";");
    match &expr.kind {
        ExprKind::InterpolatedString { parts } => {
            // Should be: Expr(a), Literal(" and "), Expr(b)
            assert_eq!(parts.len(), 3);
            assert!(matches!(&parts[0], InterpolatedPart::Expr(_)));
            assert!(matches!(&parts[1], InterpolatedPart::Literal(s, _) if s == " and "));
            assert!(matches!(&parts[2], InterpolatedPart::Expr(_)));
        }
        other => panic!("expected InterpolatedString, got {:?}", other),
    }
}
