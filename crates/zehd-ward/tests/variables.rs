mod helpers;

use zehd_rune::value::Value;

// ── Local Variables ────────────────────────────────────────────

#[test]
fn local_let_int() {
    let result = helpers::call_fn0(
        "fn f(): int { let x = 42; x }",
        vec![],
    );
    assert_eq!(result, Value::Int(42));
}

#[test]
fn local_const_string() {
    let result = helpers::call_fn0(
        r#"fn f(): string { const x = "hello"; x }"#,
        vec![],
    );
    assert_eq!(result, Value::String("hello".into()));
}

#[test]
fn local_reassign() {
    let result = helpers::call_fn0(
        "fn f(): int { let x = 1; x = 2; x }",
        vec![],
    );
    assert_eq!(result, Value::Int(2));
}

#[test]
fn local_multiple_vars() {
    let result = helpers::call_fn0(
        "fn f(): int { let a = 10; let b = 20; a + b }",
        vec![],
    );
    assert_eq!(result, Value::Int(30));
}

#[test]
fn local_var_with_param() {
    let result = helpers::call_fn0(
        "fn f(x: int): int { let y = x + 1; y * 2 }",
        vec![Value::Int(5)],
    );
    assert_eq!(result, Value::Int(12));
}

#[test]
fn local_scoped_block() {
    let result = helpers::call_fn0(
        r#"fn f(): int {
            let x = 1;
            {
                let y = 2;
            }
            x
        }"#,
        vec![],
    );
    assert_eq!(result, Value::Int(1));
}

// ── Global Variables ───────────────────────────────────────────

#[test]
fn global_var_in_server_init() {
    let result = helpers::run_init("const x = 99;");
    assert_eq!(result, Value::Unit);
}

#[test]
fn global_var_persists_across_handler() {
    let result = helpers::run_handler(
        r#"
        const version = "1.0";
        get {
            version
        }
        "#,
        0,
    );
    assert_eq!(result, Value::String("1.0".into()));
}
