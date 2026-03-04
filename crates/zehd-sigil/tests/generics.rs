mod helpers;
use helpers::*;

// ── Generic Function Inference ──────────────────────────────────

#[test]
fn generic_identity_infers_int() {
    // identity<T>(x: T): T called with int should infer T = int
    let source = r#"
        fn identity<T>(x: T): T { x }
        const result: int = identity(42);
    "#;
    check_ok(source);
}

#[test]
fn generic_identity_infers_string() {
    let source = r#"
        fn identity<T>(x: T): T { x }
        const result: string = identity("hello");
    "#;
    check_ok(source);
}

#[test]
fn generic_identity_infers_bool() {
    let source = r#"
        fn identity<T>(x: T): T { x }
        const result: bool = identity(true);
    "#;
    check_ok(source);
}

#[test]
fn generic_two_params_same_type_ok() {
    let source = r#"
        fn first<T>(a: T, b: T): T { a }
        const result: int = first(1, 2);
    "#;
    check_ok(source);
}

#[test]
fn generic_two_params_same_type_mismatch() {
    // T can't be both int and string
    let source = r#"
        fn first<T>(a: T, b: T): T { a }
        first(1, "hi");
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T110"));
}

#[test]
fn generic_explicit_type_arg_ok() {
    let source = r#"
        fn identity<T>(x: T): T { x }
        const result: string = identity<string>("hello");
    "#;
    check_ok(source);
}

#[test]
fn generic_explicit_type_arg_mismatch() {
    // Explicit T=string but passing int should error
    let source = r#"
        fn identity<T>(x: T): T { x }
        identity<string>(42);
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T110"));
}

#[test]
fn generic_multiple_type_params() {
    let source = r#"
        fn second<A, B>(a: A, b: B): B { b }
        const result: string = second(42, "hello");
    "#;
    check_ok(source);
}

#[test]
fn generic_each_call_site_independent() {
    // Two calls to the same generic function with different types
    let source = r#"
        fn identity<T>(x: T): T { x }
        const a: int = identity(42);
        const b: string = identity("hello");
    "#;
    check_ok(source);
}

// ── Type-Safe provide/inject ────────────────────────────────────

#[test]
fn provide_with_known_type_checks_value() {
    let source = r#"
        import { provide } from std;
        provide<string>("hello");
    "#;
    check_ok_with_std(source);
}

#[test]
fn provide_type_mismatch() {
    let source = r#"
        import { provide } from std;
        provide<int>("hello");
    "#;
    let result = check_with_errors_std(source);
    assert!(has_error_code(&result, "T110"));
}

#[test]
fn inject_returns_declared_type() {
    let source = r#"
        import { inject } from std;
        const name: string = inject<string>();
    "#;
    check_ok_with_std(source);
}

#[test]
fn provide_unknown_type_errors() {
    // AppName isn't a known type — must produce T101
    let source = r#"
        import { provide } from std;
        provide<AppName>("my-app");
    "#;
    let result = check_with_errors_std(source);
    assert!(has_error_code(&result, "T101"));
}
