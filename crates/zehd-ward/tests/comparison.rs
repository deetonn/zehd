mod helpers;

use zehd_rune::value::Value;

// ── Integer Comparison ─────────────────────────────────────────

#[test]
fn int_eq_true() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): bool { a == b }",
        vec![Value::Int(5), Value::Int(5)],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn int_eq_false() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): bool { a == b }",
        vec![Value::Int(5), Value::Int(3)],
    );
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn int_neq_true() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): bool { a != b }",
        vec![Value::Int(5), Value::Int(3)],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn int_neq_false() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): bool { a != b }",
        vec![Value::Int(5), Value::Int(5)],
    );
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn int_lt_true() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): bool { a < b }",
        vec![Value::Int(3), Value::Int(5)],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn int_lt_false() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): bool { a < b }",
        vec![Value::Int(5), Value::Int(3)],
    );
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn int_gt_true() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): bool { a > b }",
        vec![Value::Int(5), Value::Int(3)],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn int_leq_true() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): bool { a <= b }",
        vec![Value::Int(5), Value::Int(5)],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn int_leq_false() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): bool { a <= b }",
        vec![Value::Int(6), Value::Int(5)],
    );
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn int_geq_true() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): bool { a >= b }",
        vec![Value::Int(5), Value::Int(5)],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn int_geq_false() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): bool { a >= b }",
        vec![Value::Int(4), Value::Int(5)],
    );
    assert_eq!(result, Value::Bool(false));
}

// ── Float Comparison ───────────────────────────────────────────

#[test]
fn float_eq_true() {
    let result = helpers::call_fn0(
        "fn f(a: float, b: float): bool { a == b }",
        vec![Value::Float(1.5), Value::Float(1.5)],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn float_neq_true() {
    let result = helpers::call_fn0(
        "fn f(a: float, b: float): bool { a != b }",
        vec![Value::Float(1.5), Value::Float(2.5)],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn float_lt_true() {
    let result = helpers::call_fn0(
        "fn f(a: float, b: float): bool { a < b }",
        vec![Value::Float(1.0), Value::Float(2.0)],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn float_gt_true() {
    let result = helpers::call_fn0(
        "fn f(a: float, b: float): bool { a > b }",
        vec![Value::Float(3.0), Value::Float(2.0)],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn float_leq_true() {
    let result = helpers::call_fn0(
        "fn f(a: float, b: float): bool { a <= b }",
        vec![Value::Float(2.0), Value::Float(2.0)],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn float_geq_false() {
    let result = helpers::call_fn0(
        "fn f(a: float, b: float): bool { a >= b }",
        vec![Value::Float(1.0), Value::Float(2.0)],
    );
    assert_eq!(result, Value::Bool(false));
}

// ── String Comparison ──────────────────────────────────────────

#[test]
fn string_eq_true() {
    let result = helpers::call_fn0(
        r#"fn f(a: string, b: string): bool { a == b }"#,
        vec![
            Value::String("hello".into()),
            Value::String("hello".into()),
        ],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn string_eq_false() {
    let result = helpers::call_fn0(
        r#"fn f(a: string, b: string): bool { a == b }"#,
        vec![
            Value::String("hello".into()),
            Value::String("world".into()),
        ],
    );
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn string_neq_true() {
    let result = helpers::call_fn0(
        r#"fn f(a: string, b: string): bool { a != b }"#,
        vec![
            Value::String("hello".into()),
            Value::String("world".into()),
        ],
    );
    assert_eq!(result, Value::Bool(true));
}

// ── Bool Comparison ────────────────────────────────────────────

#[test]
fn bool_eq_true() {
    let result = helpers::call_fn0(
        "fn f(a: bool, b: bool): bool { a == b }",
        vec![Value::Bool(true), Value::Bool(true)],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn bool_eq_false() {
    let result = helpers::call_fn0(
        "fn f(a: bool, b: bool): bool { a == b }",
        vec![Value::Bool(true), Value::Bool(false)],
    );
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn bool_neq_true() {
    let result = helpers::call_fn0(
        "fn f(a: bool, b: bool): bool { a != b }",
        vec![Value::Bool(true), Value::Bool(false)],
    );
    assert_eq!(result, Value::Bool(true));
}

// ── Logical ────────────────────────────────────────────────────

#[test]
fn not_true() {
    let result = helpers::call_fn0(
        "fn f(a: bool): bool { !a }",
        vec![Value::Bool(true)],
    );
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn not_false() {
    let result = helpers::call_fn0(
        "fn f(a: bool): bool { !a }",
        vec![Value::Bool(false)],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn double_not() {
    let result = helpers::call_fn0(
        "fn f(a: bool): bool { !!a }",
        vec![Value::Bool(true)],
    );
    assert_eq!(result, Value::Bool(true));
}
