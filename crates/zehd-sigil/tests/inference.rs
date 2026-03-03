mod helpers;
use helpers::*;

#[test]
fn infer_int_literal() {
    let result = check_ok("let x = 42;");
    assert!(result.is_ok());
}

#[test]
fn infer_float_literal() {
    let result = check_ok("let x = 3.14;");
    assert!(result.is_ok());
}

#[test]
fn infer_string_literal() {
    let result = check_ok("let x = \"hello\";");
    assert!(result.is_ok());
}

#[test]
fn infer_bool_literal() {
    let result = check_ok("let x = true;");
    assert!(result.is_ok());
}

#[test]
fn infer_time_literal() {
    let result = check_ok("let x = 5s;");
    assert!(result.is_ok());
}

#[test]
fn infer_binary_add_ints() {
    let result = check_ok("let x = 1 + 2;");
    assert!(result.is_ok());
}

#[test]
fn infer_binary_add_floats() {
    let result = check_ok("let x = 1.0 + 2.0;");
    assert!(result.is_ok());
}

#[test]
fn infer_comparison_returns_bool() {
    let result = check_ok("let x = 1 < 2;");
    assert!(result.is_ok());
}

#[test]
fn infer_logical_and() {
    let result = check_ok("let x = true && false;");
    assert!(result.is_ok());
}

#[test]
fn infer_function_return_type() {
    let source = r#"
        fn double(x: int): int {
            x * 2
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn infer_list_literal() {
    let result = check_ok("let xs = [1, 2, 3];");
    assert!(result.is_ok());
}

#[test]
fn infer_none_as_option() {
    let result = check_ok("let x = None;");
    assert!(result.is_ok());
}

#[test]
fn infer_some_wraps_value() {
    let result = check_ok("let x = Some(42);");
    assert!(result.is_ok());
}

#[test]
fn infer_ok_result() {
    let result = check_ok("let x = Ok(42);");
    assert!(result.is_ok());
}

#[test]
fn infer_err_result() {
    let result = check_ok("let x = Err(\"oops\");");
    assert!(result.is_ok());
}

#[test]
fn infer_arrow_function() {
    let source = r#"
        let add = (a: int, b: int) => a + b;
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn infer_object_literal() {
    let source = r#"
        let obj = { x: 1, y: "hello" };
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn infer_interpolated_string() {
    let source = r#"
        let name = "world";
        let msg = $"hello, {name}";
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}
