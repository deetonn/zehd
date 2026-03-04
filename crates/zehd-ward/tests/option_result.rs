mod helpers;

use zehd_rune::value::Value;

// ── WrapSome ────────────────────────────────────────────────────

#[test]
fn wrap_some_int() {
    let result = helpers::call_fn0(
        "fn f(x: int): Option<int> { Some(x) }",
        vec![Value::Int(42)],
    );
    assert_eq!(
        result,
        Value::Enum {
            type_idx: 0xFFFE,
            variant_idx: 0,
            payload: Some(Box::new(Value::Int(42))),
        }
    );
}

#[test]
fn wrap_some_string() {
    let result = helpers::call_fn0(
        r#"fn f(): Option<string> { Some("hello") }"#,
        vec![],
    );
    assert_eq!(
        result,
        Value::Enum {
            type_idx: 0xFFFE,
            variant_idx: 0,
            payload: Some(Box::new(Value::String("hello".into()))),
        }
    );
}

// ── WrapOk / WrapErr ────────────────────────────────────────────

#[test]
fn wrap_ok() {
    let result = helpers::call_fn0(
        "fn f(x: int): Result<int, string> { Ok(x) }",
        vec![Value::Int(10)],
    );
    assert_eq!(
        result,
        Value::Enum {
            type_idx: 0xFFFF,
            variant_idx: 0,
            payload: Some(Box::new(Value::Int(10))),
        }
    );
}

#[test]
fn wrap_err() {
    let result = helpers::call_fn0(
        r#"fn f(): Result<int, string> { Err("bad") }"#,
        vec![],
    );
    assert_eq!(
        result,
        Value::Enum {
            type_idx: 0xFFFF,
            variant_idx: 1,
            payload: Some(Box::new(Value::String("bad".into()))),
        }
    );
}

#[test]
fn result_conditional() {
    let ok = helpers::call_fn0(
        r#"
        fn f(x: int): Result<int, string> {
            if x > 0 { Ok(x) }
            else { Err("must be positive") }
        }
        "#,
        vec![Value::Int(5)],
    );
    assert_eq!(
        ok,
        Value::Enum {
            type_idx: 0xFFFF,
            variant_idx: 0,
            payload: Some(Box::new(Value::Int(5))),
        }
    );

    let err = helpers::call_fn0(
        r#"
        fn f(x: int): Result<int, string> {
            if x > 0 { Ok(x) }
            else { Err("must be positive") }
        }
        "#,
        vec![Value::Int(-1)],
    );
    assert_eq!(
        err,
        Value::Enum {
            type_idx: 0xFFFF,
            variant_idx: 1,
            payload: Some(Box::new(Value::String("must be positive".into()))),
        }
    );
}

// ── Match on Option ─────────────────────────────────────────────

#[test]
fn match_option_some() {
    let result = helpers::call_fn0(
        r#"
        fn f(x: Option<int>): int {
            match x {
                Some(v) => v + 1,
                None => 0,
            }
        }
        "#,
        vec![Value::Enum {
            type_idx: 0xFFFE,
            variant_idx: 0,
            payload: Some(Box::new(Value::Int(41))),
        }],
    );
    assert_eq!(result, Value::Int(42));
}

#[test]
fn match_option_none() {
    let result = helpers::call_fn0(
        r#"
        fn f(x: Option<int>): int {
            match x {
                Some(v) => v + 1,
                None => 0,
            }
        }
        "#,
        vec![Value::None],
    );
    assert_eq!(result, Value::Int(0));
}

#[test]
fn match_option_none_enum() {
    // Pass None as Enum variant (as WrapNone would produce)
    let result = helpers::call_fn0(
        r#"
        fn f(x: Option<int>): int {
            match x {
                Some(v) => v,
                None => -1,
            }
        }
        "#,
        vec![Value::Enum {
            type_idx: 0xFFFE,
            variant_idx: 1,
            payload: None,
        }],
    );
    assert_eq!(result, Value::Int(-1));
}

// ── Match on Result ─────────────────────────────────────────────

#[test]
fn match_result_ok() {
    let result = helpers::call_fn0(
        r#"
        fn f(r: Result<int, string>): int {
            match r {
                Ok(v) => v,
                Err(e) => 0,
            }
        }
        "#,
        vec![Value::Enum {
            type_idx: 0xFFFF,
            variant_idx: 0,
            payload: Some(Box::new(Value::Int(99))),
        }],
    );
    assert_eq!(result, Value::Int(99));
}

#[test]
fn match_result_err() {
    let result = helpers::call_fn0(
        r#"
        fn f(r: Result<int, string>): int {
            match r {
                Ok(v) => v,
                Err(e) => 0,
            }
        }
        "#,
        vec![Value::Enum {
            type_idx: 0xFFFF,
            variant_idx: 1,
            payload: Some(Box::new(Value::String("oops".into()))),
        }],
    );
    assert_eq!(result, Value::Int(0));
}

// ── Unwrap ──────────────────────────────────────────────────────

#[test]
fn unwrap_some_succeeds() {
    let result = helpers::call_fn0(
        r#"
        fn f(): int {
            let x: Option<int> = Some(42);
            match x {
                Some(v) => v,
                None => 0,
            }
        }
        "#,
        vec![],
    );
    assert_eq!(result, Value::Int(42));
}

// ── TryOp ───────────────────────────────────────────────────────

#[test]
fn try_op_unwraps_ok() {
    let result = helpers::call_fn0(
        r#"
        fn f(x: Result<int, string>): Result<int, string> {
            let val = x?;
            Ok(val + 1)
        }
        "#,
        vec![Value::Enum {
            type_idx: 0xFFFF,
            variant_idx: 0,
            payload: Some(Box::new(Value::Int(10))),
        }],
    );
    assert_eq!(
        result,
        Value::Enum {
            type_idx: 0xFFFF,
            variant_idx: 0,
            payload: Some(Box::new(Value::Int(11))),
        }
    );
}

#[test]
fn try_op_early_returns_err() {
    let result = helpers::call_fn0(
        r#"
        fn f(x: Result<int, string>): Result<int, string> {
            let val = x?;
            Ok(val + 1)
        }
        "#,
        vec![Value::Enum {
            type_idx: 0xFFFF,
            variant_idx: 1,
            payload: Some(Box::new(Value::String("fail".into()))),
        }],
    );
    // Should early-return the Err, not reach Ok(val + 1)
    assert_eq!(
        result,
        Value::Enum {
            type_idx: 0xFFFF,
            variant_idx: 1,
            payload: Some(Box::new(Value::String("fail".into()))),
        }
    );
}

// NOTE: TryOp on Option is implemented in the VM but the type checker
// only allows `?` on Result types (T117). Testable once the type checker
// supports `?` on Option.

// ── Combined: Option in list ────────────────────────────────────

#[test]
fn list_of_options() {
    let result = helpers::call_fn0(
        r#"
        fn f(): List<Option<int>> {
            [Some(1), Some(2), None]
        }
        "#,
        vec![],
    );
    assert_eq!(
        result,
        Value::List(vec![
            Value::Enum {
                type_idx: 0xFFFE,
                variant_idx: 0,
                payload: Some(Box::new(Value::Int(1))),
            },
            Value::Enum {
                type_idx: 0xFFFE,
                variant_idx: 0,
                payload: Some(Box::new(Value::Int(2))),
            },
            Value::None,
        ])
    );
}
