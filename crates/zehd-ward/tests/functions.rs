mod helpers;

use zehd_rune::value::Value;

// ── Basic Functions ────────────────────────────────────────────

#[test]
fn call_no_args() {
    let result = helpers::call_fn0(
        r#"fn f(): string { "hello" }"#,
        vec![],
    );
    assert_eq!(result, Value::String("hello".into()));
}

#[test]
fn call_one_arg() {
    let result = helpers::call_fn0(
        "fn f(x: int): int { x * 2 }",
        vec![Value::Int(21)],
    );
    assert_eq!(result, Value::Int(42));
}

#[test]
fn call_two_args() {
    let result = helpers::call_fn0(
        "fn f(a: int, b: int): int { a + b }",
        vec![Value::Int(3), Value::Int(4)],
    );
    assert_eq!(result, Value::Int(7));
}

// ── Function Calling Function ──────────────────────────────────

#[test]
fn function_calls_function() {
    let result = helpers::call_fn(
        r#"
        fn double(x: int): int { x * 2 }
        fn test(): int { double(21) }
        "#,
        1,
        vec![],
    );
    assert_eq!(result, Value::Int(42));
}

#[test]
fn chained_function_calls() {
    let result = helpers::call_fn(
        r#"
        fn add_one(x: int): int { x + 1 }
        fn double(x: int): int { x * 2 }
        fn test(n: int): int { double(add_one(n)) }
        "#,
        2,
        vec![Value::Int(5)],
    );
    // add_one(5) = 6, double(6) = 12
    assert_eq!(result, Value::Int(12));
}

// ── Recursion ──────────────────────────────────────────────────

#[test]
fn factorial() {
    let result = helpers::call_fn0(
        r#"fn factorial(n: int): int {
            if n <= 1 { 1 }
            else { n * factorial(n - 1) }
        }"#,
        vec![Value::Int(5)],
    );
    assert_eq!(result, Value::Int(120));
}

#[test]
fn fibonacci() {
    let result = helpers::call_fn0(
        r#"fn fib(n: int): int {
            if n <= 1 { n }
            else { fib(n - 1) + fib(n - 2) }
        }"#,
        vec![Value::Int(10)],
    );
    assert_eq!(result, Value::Int(55));
}

// ── Argument Count Mismatch ────────────────────────────────────

#[test]
fn wrong_arg_count() {
    let err = helpers::call_fn0_err(
        "fn f(a: int, b: int): int { a + b }",
        vec![Value::Int(1)],
    );
    assert_eq!(err.code.to_string(), "R150");
}

// ── Call Stack Overflow ────────────────────────────────────────

#[test]
fn stack_overflow() {
    let err = helpers::call_fn0_err(
        r#"fn f(n: int): int {
            f(n + 1)
        }"#,
        vec![Value::Int(0)],
    );
    assert_eq!(err.code.to_string(), "R151");
}

// ── Arrow Functions ────────────────────────────────────────────

#[test]
fn arrow_function_simple() {
    // Arrow fn as const needs server_init to run first (sets up global)
    let result = helpers::run_handler(
        r#"
        const double = (x: int): int => x * 2;
        get {
            double(21)
        }
        "#,
        0,
    );
    assert_eq!(result, Value::Int(42));
}

#[test]
fn arrow_function_block_body() {
    let result = helpers::run_handler(
        r#"
        const compute = (x: int): int => {
            let y = x + 1;
            y * 2
        };
        get {
            compute(5)
        }
        "#,
        0,
    );
    assert_eq!(result, Value::Int(12));
}
