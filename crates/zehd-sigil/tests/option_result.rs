mod helpers;
use helpers::*;

#[test]
fn some_constructor_ok() {
    let result = check_ok("let x = Some(42);");
    assert!(result.is_ok());
}

#[test]
fn none_literal_ok() {
    let result = check_ok("let x = None;");
    assert!(result.is_ok());
}

#[test]
fn ok_constructor_ok() {
    let result = check_ok("let x = Ok(\"success\");");
    assert!(result.is_ok());
}

#[test]
fn err_constructor_ok() {
    let result = check_ok("let x = Err(\"oops\");");
    assert!(result.is_ok());
}

#[test]
fn try_on_result_ok() {
    let source = r#"
        fn might_fail(): Result<int, string> {
            let x = Ok(42);
            let val = x?;
            Ok(val)
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn try_on_non_result_error() {
    let source = r#"
        fn example() {
            let x = 42;
            x?;
        }
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T117"));
}

#[test]
fn option_typed_annotation() {
    let source = r#"
        let x: Option<int> = Some(42);
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn result_typed_annotation() {
    let source = r#"
        let x: Result<int, string> = Ok(42);
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}
