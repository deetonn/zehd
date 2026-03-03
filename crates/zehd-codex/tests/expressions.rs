mod helpers;

use helpers::*;
use zehd_codex::ast::*;

// ── Literals ─────────────────────────────────────────────────────

#[test]
fn integer_literal() {
    let expr = parse_single_expr("42;");
    assert!(matches!(expr.kind, ExprKind::IntLiteral(42)));
}

#[test]
fn negative_integer() {
    let expr = parse_single_expr("-42;");
    match &expr.kind {
        ExprKind::Unary { op, operand } => {
            assert_eq!(*op, UnaryOp::Neg);
            assert!(matches!(operand.kind, ExprKind::IntLiteral(42)));
        }
        other => panic!("expected Unary, got {:?}", other),
    }
}

#[test]
fn float_literal() {
    let expr = parse_single_expr("3.14;");
    match &expr.kind {
        ExprKind::FloatLiteral(v) => assert!((*v - 3.14).abs() < f64::EPSILON),
        other => panic!("expected FloatLiteral, got {:?}", other),
    }
}

#[test]
fn string_literal() {
    let expr = parse_single_expr("\"hello\";");
    assert!(matches!(&expr.kind, ExprKind::StringLiteral(s) if s == "hello"));
}

#[test]
fn time_literal() {
    let cases = vec![
        ("500ms;", 500u64),
        ("60s;", 60000),
        ("5m;", 300000),
        ("1h;", 3600000),
    ];
    for (source, expected) in cases {
        let expr = parse_single_expr(source);
        match &expr.kind {
            ExprKind::TimeLiteral(v) => assert_eq!(*v, expected, "for source: {}", source),
            other => panic!("expected TimeLiteral, got {:?} for source: {}", other, source),
        }
    }
}

#[test]
fn bool_literals() {
    let expr_true = parse_single_expr("true;");
    assert!(matches!(expr_true.kind, ExprKind::BoolLiteral(true)));

    let expr_false = parse_single_expr("false;");
    assert!(matches!(expr_false.kind, ExprKind::BoolLiteral(false)));
}

#[test]
fn none_literal() {
    let expr = parse_single_expr("None;");
    assert!(matches!(expr.kind, ExprKind::NoneLiteral));
}

#[test]
fn identifier() {
    let expr = parse_single_expr("foo;");
    match &expr.kind {
        ExprKind::Ident(id) => assert_eq!(id.name, "foo"),
        other => panic!("expected Ident, got {:?}", other),
    }
}

#[test]
fn self_expr() {
    let expr = parse_single_expr("self;");
    assert!(matches!(expr.kind, ExprKind::SelfExpr));
}

// ── Enum Constructors ────────────────────────────────────────────

#[test]
fn some_constructor() {
    let expr = parse_single_expr("Some(42);");
    match &expr.kind {
        ExprKind::EnumConstructor { name, arg } => {
            assert_eq!(name.name, "Some");
            assert!(matches!(arg.kind, ExprKind::IntLiteral(42)));
        }
        other => panic!("expected EnumConstructor, got {:?}", other),
    }
}

#[test]
fn ok_constructor() {
    let expr = parse_single_expr("Ok(value);");
    match &expr.kind {
        ExprKind::EnumConstructor { name, arg } => {
            assert_eq!(name.name, "Ok");
            assert!(matches!(&arg.kind, ExprKind::Ident(id) if id.name == "value"));
        }
        other => panic!("expected EnumConstructor, got {:?}", other),
    }
}

#[test]
fn err_constructor() {
    let expr = parse_single_expr("Err(\"not found\");");
    match &expr.kind {
        ExprKind::EnumConstructor { name, arg } => {
            assert_eq!(name.name, "Err");
            assert!(matches!(&arg.kind, ExprKind::StringLiteral(s) if s == "not found"));
        }
        other => panic!("expected EnumConstructor, got {:?}", other),
    }
}

// ── Binary Operators ─────────────────────────────────────────────

#[test]
fn binary_add() {
    let expr = parse_single_expr("1 + 2;");
    match &expr.kind {
        ExprKind::Binary { op, left, right } => {
            assert_eq!(*op, BinaryOp::Add);
            assert!(matches!(left.kind, ExprKind::IntLiteral(1)));
            assert!(matches!(right.kind, ExprKind::IntLiteral(2)));
        }
        other => panic!("expected Binary, got {:?}", other),
    }
}

#[test]
fn binary_operators() {
    let cases = vec![
        ("1 + 2;", BinaryOp::Add),
        ("1 - 2;", BinaryOp::Sub),
        ("1 * 2;", BinaryOp::Mul),
        ("1 / 2;", BinaryOp::Div),
        ("1 % 2;", BinaryOp::Mod),
        ("1 == 2;", BinaryOp::Eq),
        ("1 != 2;", BinaryOp::NotEq),
        ("1 < 2;", BinaryOp::Lt),
        ("1 > 2;", BinaryOp::Gt),
        ("1 <= 2;", BinaryOp::LtEq),
        ("1 >= 2;", BinaryOp::GtEq),
        ("true && false;", BinaryOp::And),
        ("true || false;", BinaryOp::Or),
    ];
    for (source, expected_op) in cases {
        let expr = parse_single_expr(source);
        match &expr.kind {
            ExprKind::Binary { op, .. } => {
                assert_eq!(*op, expected_op, "for source: {}", source)
            }
            other => panic!("expected Binary, got {:?} for source: {}", other, source),
        }
    }
}

// ── Precedence ───────────────────────────────────────────────────

#[test]
fn precedence_mul_over_add() {
    // 1 + 2 * 3 should be 1 + (2 * 3)
    let expr = parse_single_expr("1 + 2 * 3;");
    match &expr.kind {
        ExprKind::Binary { op, left, right } => {
            assert_eq!(*op, BinaryOp::Add);
            assert!(matches!(left.kind, ExprKind::IntLiteral(1)));
            match &right.kind {
                ExprKind::Binary { op, left, right } => {
                    assert_eq!(*op, BinaryOp::Mul);
                    assert!(matches!(left.kind, ExprKind::IntLiteral(2)));
                    assert!(matches!(right.kind, ExprKind::IntLiteral(3)));
                }
                other => panic!("expected Binary, got {:?}", other),
            }
        }
        other => panic!("expected Binary, got {:?}", other),
    }
}

#[test]
fn precedence_left_associative() {
    // 1 - 2 - 3 should be (1 - 2) - 3
    let expr = parse_single_expr("1 - 2 - 3;");
    match &expr.kind {
        ExprKind::Binary { op, left, right } => {
            assert_eq!(*op, BinaryOp::Sub);
            assert!(matches!(right.kind, ExprKind::IntLiteral(3)));
            match &left.kind {
                ExprKind::Binary { op, left, right } => {
                    assert_eq!(*op, BinaryOp::Sub);
                    assert!(matches!(left.kind, ExprKind::IntLiteral(1)));
                    assert!(matches!(right.kind, ExprKind::IntLiteral(2)));
                }
                other => panic!("expected Binary, got {:?}", other),
            }
        }
        other => panic!("expected Binary, got {:?}", other),
    }
}

#[test]
fn precedence_comparison_over_logical() {
    // a > 0 && b < 10 should be (a > 0) && (b < 10)
    let expr = parse_single_expr("a > 0 && b < 10;");
    match &expr.kind {
        ExprKind::Binary { op, left, right } => {
            assert_eq!(*op, BinaryOp::And);
            assert!(matches!(&left.kind, ExprKind::Binary { op, .. } if *op == BinaryOp::Gt));
            assert!(matches!(&right.kind, ExprKind::Binary { op, .. } if *op == BinaryOp::Lt));
        }
        other => panic!("expected Binary, got {:?}", other),
    }
}

// ── Unary Operators ──────────────────────────────────────────────

#[test]
fn unary_not() {
    let expr = parse_single_expr("!flag;");
    match &expr.kind {
        ExprKind::Unary { op, operand } => {
            assert_eq!(*op, UnaryOp::Not);
            assert!(matches!(&operand.kind, ExprKind::Ident(id) if id.name == "flag"));
        }
        other => panic!("expected Unary, got {:?}", other),
    }
}

#[test]
fn unary_neg() {
    let expr = parse_single_expr("-x;");
    match &expr.kind {
        ExprKind::Unary { op, operand } => {
            assert_eq!(*op, UnaryOp::Neg);
            assert!(matches!(&operand.kind, ExprKind::Ident(id) if id.name == "x"));
        }
        other => panic!("expected Unary, got {:?}", other),
    }
}

#[test]
fn double_negation() {
    let expr = parse_single_expr("!!flag;");
    match &expr.kind {
        ExprKind::Unary { op, operand } => {
            assert_eq!(*op, UnaryOp::Not);
            match &operand.kind {
                ExprKind::Unary { op, operand } => {
                    assert_eq!(*op, UnaryOp::Not);
                    assert!(matches!(&operand.kind, ExprKind::Ident(id) if id.name == "flag"));
                }
                other => panic!("expected inner Unary, got {:?}", other),
            }
        }
        other => panic!("expected Unary, got {:?}", other),
    }
}

// ── Postfix Operators ────────────────────────────────────────────

#[test]
fn try_operator() {
    let expr = parse_single_expr("result?;");
    match &expr.kind {
        ExprKind::Try(inner) => {
            assert!(matches!(&inner.kind, ExprKind::Ident(id) if id.name == "result"));
        }
        other => panic!("expected Try, got {:?}", other),
    }
}

#[test]
fn field_access() {
    let expr = parse_single_expr("self.request;");
    match &expr.kind {
        ExprKind::FieldAccess { object, field } => {
            assert!(matches!(object.kind, ExprKind::SelfExpr));
            assert_eq!(field.name, "request");
        }
        other => panic!("expected FieldAccess, got {:?}", other),
    }
}

#[test]
fn chained_field_access() {
    let expr = parse_single_expr("self.request.headers;");
    match &expr.kind {
        ExprKind::FieldAccess { object, field } => {
            assert_eq!(field.name, "headers");
            match &object.kind {
                ExprKind::FieldAccess { object, field } => {
                    assert!(matches!(object.kind, ExprKind::SelfExpr));
                    assert_eq!(field.name, "request");
                }
                other => panic!("expected inner FieldAccess, got {:?}", other),
            }
        }
        other => panic!("expected FieldAccess, got {:?}", other),
    }
}

#[test]
fn index_access() {
    let expr = parse_single_expr("items[0];");
    match &expr.kind {
        ExprKind::Index { object, index } => {
            assert!(matches!(&object.kind, ExprKind::Ident(id) if id.name == "items"));
            assert!(matches!(index.kind, ExprKind::IntLiteral(0)));
        }
        other => panic!("expected Index, got {:?}", other),
    }
}

#[test]
fn function_call_no_args() {
    let expr = parse_single_expr("foo();");
    match &expr.kind {
        ExprKind::Call {
            callee, args, type_args,
        } => {
            assert!(matches!(&callee.kind, ExprKind::Ident(id) if id.name == "foo"));
            assert!(args.is_empty());
            assert!(type_args.is_empty());
        }
        other => panic!("expected Call, got {:?}", other),
    }
}

#[test]
fn function_call_with_args() {
    let expr = parse_single_expr("add(1, 2);");
    match &expr.kind {
        ExprKind::Call { callee, args, .. } => {
            assert!(matches!(&callee.kind, ExprKind::Ident(id) if id.name == "add"));
            assert_eq!(args.len(), 2);
            assert!(matches!(args[0].kind, ExprKind::IntLiteral(1)));
            assert!(matches!(args[1].kind, ExprKind::IntLiteral(2)));
        }
        other => panic!("expected Call, got {:?}", other),
    }
}

#[test]
fn method_call_chain() {
    let expr = parse_single_expr("self.response.status(404);");
    match &expr.kind {
        ExprKind::Call { callee, args, .. } => {
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0].kind, ExprKind::IntLiteral(404)));
            match &callee.kind {
                ExprKind::FieldAccess { object, field } => {
                    assert_eq!(field.name, "status");
                    match &object.kind {
                        ExprKind::FieldAccess { object, field } => {
                            assert!(matches!(object.kind, ExprKind::SelfExpr));
                            assert_eq!(field.name, "response");
                        }
                        other => panic!("expected inner FieldAccess, got {:?}", other),
                    }
                }
                other => panic!("expected FieldAccess, got {:?}", other),
            }
        }
        other => panic!("expected Call, got {:?}", other),
    }
}

#[test]
fn chained_calls() {
    let expr = parse_single_expr("a.b().c();");
    match &expr.kind {
        ExprKind::Call { callee, .. } => {
            match &callee.kind {
                ExprKind::FieldAccess { object, field } => {
                    assert_eq!(field.name, "c");
                    match &object.kind {
                        ExprKind::Call { callee, .. } => {
                            match &callee.kind {
                                ExprKind::FieldAccess { object, field } => {
                                    assert_eq!(field.name, "b");
                                    assert!(
                                        matches!(&object.kind, ExprKind::Ident(id) if id.name == "a")
                                    );
                                }
                                other => panic!("expected FieldAccess, got {:?}", other),
                            }
                        }
                        other => panic!("expected Call, got {:?}", other),
                    }
                }
                other => panic!("expected FieldAccess, got {:?}", other),
            }
        }
        other => panic!("expected Call, got {:?}", other),
    }
}

#[test]
fn try_after_call() {
    let expr = parse_single_expr("db.find(id)?;");
    match &expr.kind {
        ExprKind::Try(inner) => {
            assert!(matches!(&inner.kind, ExprKind::Call { .. }));
        }
        other => panic!("expected Try, got {:?}", other),
    }
}

// ── Grouped Expression ───────────────────────────────────────────

#[test]
fn grouped_expression() {
    let expr = parse_single_expr("(1 + 2);");
    match &expr.kind {
        ExprKind::Grouped(inner) => {
            assert!(matches!(&inner.kind, ExprKind::Binary { op: BinaryOp::Add, .. }));
        }
        other => panic!("expected Grouped, got {:?}", other),
    }
}

#[test]
fn grouped_changes_precedence() {
    // (1 + 2) * 3
    let expr = parse_single_expr("(1 + 2) * 3;");
    match &expr.kind {
        ExprKind::Binary { op, left, right } => {
            assert_eq!(*op, BinaryOp::Mul);
            assert!(matches!(&left.kind, ExprKind::Grouped(_)));
            assert!(matches!(right.kind, ExprKind::IntLiteral(3)));
        }
        other => panic!("expected Binary, got {:?}", other),
    }
}
