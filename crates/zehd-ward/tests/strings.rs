mod helpers;

use zehd_rune::value::Value;

// ── String Concatenation ───────────────────────────────────────

#[test]
fn concat_two_strings() {
    let result = helpers::call_fn0(
        r#"fn f(a: string, b: string): string { a + b }"#,
        vec![
            Value::String("hello".into()),
            Value::String(" world".into()),
        ],
    );
    assert_eq!(result, Value::String("hello world".into()));
}

// ── String Interpolation ───────────────────────────────────────

#[test]
fn interpolation_simple() {
    let result = helpers::call_fn0(
        r#"fn f(name: string): string { $"Hello, {name}!" }"#,
        vec![Value::String("world".into())],
    );
    assert_eq!(result, Value::String("Hello, world!".into()));
}

#[test]
fn interpolation_int() {
    let result = helpers::call_fn0(
        r#"fn f(x: int): string { $"value: {x}" }"#,
        vec![Value::Int(42)],
    );
    assert_eq!(result, Value::String("value: 42".into()));
}

#[test]
fn interpolation_multiple() {
    let result = helpers::call_fn0(
        r#"fn f(a: string, b: int): string { $"{a} = {b}" }"#,
        vec![Value::String("x".into()), Value::Int(10)],
    );
    assert_eq!(result, Value::String("x = 10".into()));
}

// ── ToString ───────────────────────────────────────────────────

#[test]
fn to_string_int() {
    let result = helpers::call_fn0(
        r#"fn f(x: int): string { $"{x}" }"#,
        vec![Value::Int(42)],
    );
    assert_eq!(result, Value::String("42".into()));
}

#[test]
fn to_string_bool() {
    let result = helpers::call_fn0(
        r#"fn f(x: bool): string { $"{x}" }"#,
        vec![Value::Bool(true)],
    );
    assert_eq!(result, Value::String("true".into()));
}
