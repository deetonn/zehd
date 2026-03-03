mod helpers;

use helpers::*;
use zehd_codex::ast::*;

#[test]
fn simple_match() {
    let expr = parse_single_expr(
        "match x {
            1 => true,
            2 => false,
        };",
    );
    match &expr.kind {
        ExprKind::Match { scrutinee, arms } => {
            assert!(matches!(&scrutinee.kind, ExprKind::Ident(id) if id.name == "x"));
            assert_eq!(arms.len(), 2);

            match &arms[0].pattern.kind {
                PatternKind::Literal(LiteralPattern::Int(1)) => {}
                other => panic!("expected Int(1), got {:?}", other),
            }
            assert!(matches!(arms[0].body.kind, ExprKind::BoolLiteral(true)));

            match &arms[1].pattern.kind {
                PatternKind::Literal(LiteralPattern::Int(2)) => {}
                other => panic!("expected Int(2), got {:?}", other),
            }
            assert!(matches!(arms[1].body.kind, ExprKind::BoolLiteral(false)));
        }
        other => panic!("expected Match, got {:?}", other),
    }
}

#[test]
fn match_with_wildcard() {
    let expr = parse_single_expr(
        "match x {
            1 => true,
            _ => false,
        };",
    );
    match &expr.kind {
        ExprKind::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);
            assert!(matches!(arms[1].pattern.kind, PatternKind::Wildcard));
        }
        other => panic!("expected Match, got {:?}", other),
    }
}

#[test]
fn match_with_binding() {
    let expr = parse_single_expr(
        "match x {
            value => value,
        };",
    );
    match &expr.kind {
        ExprKind::Match { arms, .. } => {
            assert_eq!(arms.len(), 1);
            match &arms[0].pattern.kind {
                PatternKind::Binding(id) => assert_eq!(id.name, "value"),
                other => panic!("expected Binding, got {:?}", other),
            }
        }
        other => panic!("expected Match, got {:?}", other),
    }
}

#[test]
fn match_enum_variant_with_binding() {
    let expr = parse_single_expr(
        "match result {
            Ok(value) => value,
            Err(e) => e,
        };",
    );
    match &expr.kind {
        ExprKind::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);

            match &arms[0].pattern.kind {
                PatternKind::EnumVariant { path, binding } => {
                    assert_eq!(path.len(), 1);
                    assert_eq!(path[0].name, "Ok");
                    match &binding.as_ref().unwrap().kind {
                        PatternKind::Binding(id) => assert_eq!(id.name, "value"),
                        other => panic!("expected Binding, got {:?}", other),
                    }
                }
                other => panic!("expected EnumVariant, got {:?}", other),
            }

            match &arms[1].pattern.kind {
                PatternKind::EnumVariant { path, binding } => {
                    assert_eq!(path[0].name, "Err");
                    assert!(binding.is_some());
                }
                other => panic!("expected EnumVariant, got {:?}", other),
            }
        }
        other => panic!("expected Match, got {:?}", other),
    }
}

#[test]
fn match_dotted_enum_variant() {
    let expr = parse_single_expr(
        "match err {
            UserError.NotFound(msg) => msg,
            UserError.Unauthorized => 403,
        };",
    );
    match &expr.kind {
        ExprKind::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);

            match &arms[0].pattern.kind {
                PatternKind::EnumVariant { path, binding } => {
                    assert_eq!(path.len(), 2);
                    assert_eq!(path[0].name, "UserError");
                    assert_eq!(path[1].name, "NotFound");
                    assert!(binding.is_some());
                }
                other => panic!("expected EnumVariant, got {:?}", other),
            }

            match &arms[1].pattern.kind {
                PatternKind::EnumVariant { path, binding } => {
                    assert_eq!(path.len(), 2);
                    assert_eq!(path[0].name, "UserError");
                    assert_eq!(path[1].name, "Unauthorized");
                    assert!(binding.is_none());
                }
                other => panic!("expected EnumVariant, got {:?}", other),
            }
        }
        other => panic!("expected Match, got {:?}", other),
    }
}

#[test]
fn match_with_block_body() {
    let expr = parse_single_expr(
        "match x {
            1 => {
                let y = 2;
                y
            }
        };",
    );
    match &expr.kind {
        ExprKind::Match { arms, .. } => {
            assert_eq!(arms.len(), 1);
            assert!(matches!(arms[0].body.kind, ExprKind::Block(_)));
        }
        other => panic!("expected Match, got {:?}", other),
    }
}

#[test]
fn match_string_patterns() {
    let expr = parse_single_expr(
        "match method {
            \"GET\" => 1,
            \"POST\" => 2,
            _ => 0,
        };",
    );
    match &expr.kind {
        ExprKind::Match { arms, .. } => {
            assert_eq!(arms.len(), 3);
            match &arms[0].pattern.kind {
                PatternKind::Literal(LiteralPattern::String(s)) => assert_eq!(s, "GET"),
                other => panic!("expected String literal pattern, got {:?}", other),
            }
        }
        other => panic!("expected Match, got {:?}", other),
    }
}

#[test]
fn match_none_variant() {
    let expr = parse_single_expr(
        "match opt {
            Some(x) => x,
            None => 0,
        };",
    );
    match &expr.kind {
        ExprKind::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);
            match &arms[0].pattern.kind {
                PatternKind::EnumVariant { path, binding } => {
                    assert_eq!(path[0].name, "Some");
                    assert!(binding.is_some());
                }
                other => panic!("expected EnumVariant, got {:?}", other),
            }
            match &arms[1].pattern.kind {
                PatternKind::EnumVariant { path, binding } => {
                    assert_eq!(path[0].name, "None");
                    assert!(binding.is_none());
                }
                other => panic!("expected EnumVariant, got {:?}", other),
            }
        }
        other => panic!("expected Match, got {:?}", other),
    }
}
