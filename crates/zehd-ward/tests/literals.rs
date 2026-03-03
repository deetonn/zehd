mod helpers;

use zehd_rune::value::Value;

#[test]
fn int_literal() {
    let result = helpers::call_fn0("fn f(): int { 42 }", vec![]);
    assert_eq!(result, Value::Int(42));
}

#[test]
fn negative_int_literal() {
    let result = helpers::call_fn0("fn f(): int { -7 }", vec![]);
    assert_eq!(result, Value::Int(-7));
}

#[test]
fn zero_literal() {
    let result = helpers::call_fn0("fn f(): int { 0 }", vec![]);
    assert_eq!(result, Value::Int(0));
}

#[test]
fn float_literal() {
    let result = helpers::call_fn0("fn f(): float { 3.14 }", vec![]);
    assert_eq!(result, Value::Float(3.14));
}

#[test]
fn negative_float_literal() {
    let result = helpers::call_fn0("fn f(): float { -2.5 }", vec![]);
    assert_eq!(result, Value::Float(-2.5));
}

#[test]
fn string_literal() {
    let result = helpers::call_fn0(r#"fn f(): string { "hello" }"#, vec![]);
    assert_eq!(result, Value::String("hello".to_string()));
}

#[test]
fn empty_string_literal() {
    let result = helpers::call_fn0(r#"fn f(): string { "" }"#, vec![]);
    assert_eq!(result, Value::String("".to_string()));
}

#[test]
fn bool_true() {
    let result = helpers::call_fn0("fn f(): bool { true }", vec![]);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn bool_false() {
    let result = helpers::call_fn0("fn f(): bool { false }", vec![]);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn unit_return() {
    let result = helpers::call_fn0("fn f() { }", vec![]);
    assert_eq!(result, Value::Unit);
}

#[test]
fn identity_int() {
    let result = helpers::call_fn0(
        "fn f(x: int): int { x }",
        vec![Value::Int(99)],
    );
    assert_eq!(result, Value::Int(99));
}

#[test]
fn identity_string() {
    let result = helpers::call_fn0(
        r#"fn f(x: string): string { x }"#,
        vec![Value::String("world".into())],
    );
    assert_eq!(result, Value::String("world".into()));
}
