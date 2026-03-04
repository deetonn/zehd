mod helpers;

use zehd_rune::value::Value;
use zehd_ward::error::RuntimeErrorCode;

// ── MakeList ────────────────────────────────────────────────────

#[test]
fn make_list_empty() {
    let result = helpers::call_fn0(
        "fn f(): List<int> { [] }",
        vec![],
    );
    assert_eq!(result, Value::List(vec![]));
}

#[test]
fn make_list_integers() {
    let result = helpers::call_fn0(
        "fn f(): List<int> { [1, 2, 3] }",
        vec![],
    );
    assert_eq!(
        result,
        Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
    );
}

#[test]
fn make_list_strings() {
    let result = helpers::call_fn0(
        r#"fn f(): List<string> { ["a", "b"] }"#,
        vec![],
    );
    assert_eq!(
        result,
        Value::List(vec![
            Value::String("a".into()),
            Value::String("b".into()),
        ])
    );
}

#[test]
fn make_list_from_args() {
    let result = helpers::call_fn0(
        "fn f(x: int): List<int> { [x, x + 1, x + 2] }",
        vec![Value::Int(10)],
    );
    assert_eq!(
        result,
        Value::List(vec![Value::Int(10), Value::Int(11), Value::Int(12)])
    );
}

// ── GetIndex (list) ─────────────────────────────────────────────

#[test]
fn get_index_list_first() {
    let result = helpers::call_fn0(
        "fn f(): int { let xs = [10, 20, 30]; xs[0] }",
        vec![],
    );
    assert_eq!(result, Value::Int(10));
}

#[test]
fn get_index_list_middle() {
    let result = helpers::call_fn0(
        "fn f(): int { let xs = [10, 20, 30]; xs[1] }",
        vec![],
    );
    assert_eq!(result, Value::Int(20));
}

#[test]
fn get_index_list_last() {
    let result = helpers::call_fn0(
        "fn f(): int { let xs = [10, 20, 30]; xs[2] }",
        vec![],
    );
    assert_eq!(result, Value::Int(30));
}

#[test]
fn get_index_list_out_of_bounds() {
    let err = helpers::call_fn0_err(
        "fn f(): int { let xs = [1, 2]; xs[5] }",
        vec![],
    );
    assert_eq!(err.code, RuntimeErrorCode::R161);
}

// NOTE: GetIndex on objects and list.length are implemented in the VM
// but the type checker doesn't yet support indexing user types (T116)
// or field access on List<T> (T104). These will be testable once
// the type checker is updated.

// ── TestEqual ───────────────────────────────────────────────────

#[test]
fn match_literal_first_arm() {
    let result = helpers::call_fn0(
        r#"
        fn f(x: int): string {
            match x {
                1 => "one",
                2 => "two",
                _ => "other",
            }
        }
        "#,
        vec![Value::Int(1)],
    );
    assert_eq!(result, Value::String("one".into()));
}

#[test]
fn match_literal_second_arm() {
    let result = helpers::call_fn0(
        r#"
        fn f(x: int): string {
            match x {
                1 => "one",
                2 => "two",
                _ => "other",
            }
        }
        "#,
        vec![Value::Int(2)],
    );
    assert_eq!(result, Value::String("two".into()));
}

#[test]
fn match_literal_wildcard() {
    let result = helpers::call_fn0(
        r#"
        fn f(x: int): string {
            match x {
                1 => "one",
                2 => "two",
                _ => "other",
            }
        }
        "#,
        vec![Value::Int(99)],
    );
    assert_eq!(result, Value::String("other".into()));
}

#[test]
fn match_string_literal() {
    let result = helpers::call_fn0(
        r#"
        fn f(s: string): int {
            match s {
                "hello" => 1,
                "world" => 2,
                _ => 0,
            }
        }
        "#,
        vec![Value::String("world".into())],
    );
    assert_eq!(result, Value::Int(2));
}

#[test]
fn match_with_binding() {
    let result = helpers::call_fn0(
        r#"
        fn f(x: int): int {
            match x {
                n => n + 1,
            }
        }
        "#,
        vec![Value::Int(41)],
    );
    assert_eq!(result, Value::Int(42));
}

// ── For-loop over list ──────────────────────────────────────────

#[test]
fn for_loop_sum() {
    let result = helpers::call_fn0(
        r#"
        fn f(): int {
            let xs = [1, 2, 3, 4];
            let sum = 0;
            for x in xs {
                sum = sum + x;
            }
            sum
        }
        "#,
        vec![],
    );
    assert_eq!(result, Value::Int(10));
}

#[test]
fn for_loop_empty_list() {
    let result = helpers::call_fn0(
        r#"
        fn f(): int {
            let xs: List<int> = [];
            let sum = 0;
            for x in xs {
                sum = sum + x;
            }
            sum
        }
        "#,
        vec![],
    );
    assert_eq!(result, Value::Int(0));
}
