mod helpers;

use zehd_rune::value::Value;

// ── Integer Arithmetic ─────────────────────────────────────────

#[test]
fn int_add() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): int { a + b }",
        vec![Value::Int(3), Value::Int(4)],
    );
    assert_eq!(result, Value::Int(7));
}

#[test]
fn int_sub() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): int { a - b }",
        vec![Value::Int(10), Value::Int(3)],
    );
    assert_eq!(result, Value::Int(7));
}

#[test]
fn int_mul() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): int { a * b }",
        vec![Value::Int(6), Value::Int(7)],
    );
    assert_eq!(result, Value::Int(42));
}

#[test]
fn int_div() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): int { a / b }",
        vec![Value::Int(10), Value::Int(3)],
    );
    assert_eq!(result, Value::Int(3));
}

#[test]
fn int_mod() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): int { a % b }",
        vec![Value::Int(10), Value::Int(3)],
    );
    assert_eq!(result, Value::Int(1));
}

#[test]
fn int_neg() {
    let result = helpers::call_fn0(
        "fn f(a: int): int { -a }",
        vec![Value::Int(5)],
    );
    assert_eq!(result, Value::Int(-5));
}

#[test]
fn int_neg_negative() {
    let result = helpers::call_fn0(
        "fn f(a: int): int { -a }",
        vec![Value::Int(-3)],
    );
    assert_eq!(result, Value::Int(3));
}

#[test]
fn int_div_by_zero() {
    let err = helpers::call_fn0_err(
        "fn f(a: int, b: int): int { a / b }",
        vec![Value::Int(1), Value::Int(0)],
    );
    assert_eq!(err.code.to_string(), "R100");
}

#[test]
fn int_mod_by_zero() {
    let err = helpers::call_fn0_err(
        "fn f(a: int, b: int): int { a % b }",
        vec![Value::Int(1), Value::Int(0)],
    );
    assert_eq!(err.code.to_string(), "R100");
}

#[test]
fn int_complex_expr() {
    // (2 + 3) * 4 - 1
    let result = helpers::call_fn0(
        "fn f(): int { (2 + 3) * 4 - 1 }",
        vec![],
    );
    assert_eq!(result, Value::Int(19));
}

// ── Float Arithmetic ───────────────────────────────────────────

#[test]
fn float_add() {
    let result = helpers::call_fn0(
        "fn f(a: float, b: float): float { a + b }",
        vec![Value::Float(1.5), Value::Float(2.5)],
    );
    assert_eq!(result, Value::Float(4.0));
}

#[test]
fn float_sub() {
    let result = helpers::call_fn0(
        "fn f(a: float, b: float): float { a - b }",
        vec![Value::Float(5.0), Value::Float(2.0)],
    );
    assert_eq!(result, Value::Float(3.0));
}

#[test]
fn float_mul() {
    let result = helpers::call_fn0(
        "fn f(a: float, b: float): float { a * b }",
        vec![Value::Float(3.0), Value::Float(2.5)],
    );
    assert_eq!(result, Value::Float(7.5));
}

#[test]
fn float_div() {
    let result = helpers::call_fn0(
        "fn f(a: float, b: float): float { a / b }",
        vec![Value::Float(10.0), Value::Float(4.0)],
    );
    assert_eq!(result, Value::Float(2.5));
}

#[test]
fn float_neg() {
    let result = helpers::call_fn0(
        "fn f(a: float): float { -a }",
        vec![Value::Float(3.14)],
    );
    assert_eq!(result, Value::Float(-3.14));
}

#[test]
fn float_div_by_zero() {
    let err = helpers::call_fn0_err(
        "fn f(a: float, b: float): float { a / b }",
        vec![Value::Float(1.0), Value::Float(0.0)],
    );
    assert_eq!(err.code.to_string(), "R100");
}

// ── String Arithmetic ──────────────────────────────────────────

#[test]
fn string_add() {
    let result = helpers::call_fn0(
        r#"fn f(a: string, b: string): string { a + b }"#,
        vec![
            Value::String("hello".into()),
            Value::String(" world".into()),
        ],
    );
    assert_eq!(result, Value::String("hello world".into()));
}

#[test]
fn string_add_three() {
    let result = helpers::call_fn0(
        r#"fn f(a: string, b: string, c: string): string { a + b + c }"#,
        vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
        ],
    );
    assert_eq!(result, Value::String("abc".into()));
}
