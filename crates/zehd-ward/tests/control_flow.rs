mod helpers;

use zehd_rune::value::Value;

// ── If/Else Expression ─────────────────────────────────────────

#[test]
fn if_true_branch() {
    let result = helpers::call_fn0(
        "fn f(x: int): int { if x > 0 { 1 } else { 0 } }",
        vec![Value::Int(5)],
    );
    assert_eq!(result, Value::Int(1));
}

#[test]
fn if_false_branch() {
    let result = helpers::call_fn0(
        "fn f(x: int): int { if x > 0 { 1 } else { 0 } }",
        vec![Value::Int(-1)],
    );
    assert_eq!(result, Value::Int(0));
}

#[test]
fn if_nested() {
    let result = helpers::call_fn0(
        r#"fn f(x: int): string {
            if x > 0 {
                if x > 100 {
                    "big"
                } else {
                    "small"
                }
            } else {
                "negative"
            }
        }"#,
        vec![Value::Int(50)],
    );
    assert_eq!(result, Value::String("small".into()));
}

#[test]
fn if_nested_big() {
    let result = helpers::call_fn0(
        r#"fn f(x: int): string {
            if x > 0 {
                if x > 100 {
                    "big"
                } else {
                    "small"
                }
            } else {
                "negative"
            }
        }"#,
        vec![Value::Int(200)],
    );
    assert_eq!(result, Value::String("big".into()));
}

#[test]
fn if_nested_negative() {
    let result = helpers::call_fn0(
        r#"fn f(x: int): string {
            if x > 0 {
                if x > 100 {
                    "big"
                } else {
                    "small"
                }
            } else {
                "negative"
            }
        }"#,
        vec![Value::Int(-5)],
    );
    assert_eq!(result, Value::String("negative".into()));
}

// ── While Loop ─────────────────────────────────────────────────

#[test]
fn while_countdown() {
    let result = helpers::call_fn0(
        r#"fn f(n: int): int {
            let x = n;
            while x > 0 {
                x = x - 1;
            }
            x
        }"#,
        vec![Value::Int(5)],
    );
    assert_eq!(result, Value::Int(0));
}

#[test]
fn while_accumulate() {
    let result = helpers::call_fn0(
        r#"fn f(n: int): int {
            let sum = 0;
            let i = 1;
            while i <= n {
                sum = sum + i;
                i = i + 1;
            }
            sum
        }"#,
        vec![Value::Int(10)],
    );
    assert_eq!(result, Value::Int(55));
}

#[test]
fn while_break() {
    let result = helpers::call_fn0(
        r#"fn f(limit: int): int {
            let x = 0;
            while true {
                if x > limit {
                    break;
                }
                x = x + 1;
            }
            x
        }"#,
        vec![Value::Int(5)],
    );
    assert_eq!(result, Value::Int(6));
}

#[test]
fn while_continue() {
    // Count to limit, but track how many times we don't skip
    let result = helpers::call_fn0(
        r#"fn f(): int {
            let x = 0;
            let count = 0;
            while x < 10 {
                x = x + 1;
                if x == 5 {
                    continue;
                }
                count = count + 1;
            }
            count
        }"#,
        vec![],
    );
    // x goes 1..10, count increments for all except x==5 → 9
    assert_eq!(result, Value::Int(9));
}

#[test]
fn while_zero_iterations() {
    let result = helpers::call_fn0(
        r#"fn f(): int {
            let x = 0;
            while false {
                x = x + 1;
            }
            x
        }"#,
        vec![],
    );
    assert_eq!(result, Value::Int(0));
}

// ── Early Return ───────────────────────────────────────────────

#[test]
fn early_return() {
    let result = helpers::call_fn0(
        r#"fn f(x: int): int {
            if x > 0 {
                return x;
            }
            0
        }"#,
        vec![Value::Int(42)],
    );
    assert_eq!(result, Value::Int(42));
}

#[test]
fn early_return_fallthrough() {
    let result = helpers::call_fn0(
        r#"fn f(x: int): int {
            if x > 0 {
                return x;
            }
            0
        }"#,
        vec![Value::Int(-1)],
    );
    assert_eq!(result, Value::Int(0));
}
