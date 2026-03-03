mod helpers;
use helpers::*;

#[test]
fn resolves_variable_reference() {
    let result = check_ok("let x = 42; x;");
    assert!(result.is_ok());
}

#[test]
fn undefined_variable_error() {
    let result = check_with_errors("y;");
    assert!(has_error_code(&result, "T100"));
}

#[test]
fn duplicate_definition_error() {
    let result = check_with_errors("let x = 1; let x = 2;");
    assert!(has_error_code(&result, "T102"));
}

#[test]
fn function_forward_reference() {
    // Functions are collected before bodies are walked.
    let source = r#"
        greet();
        fn greet() {}
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn variable_scoping_block() {
    let source = r#"
        fn f() {
            let x = 1;
            {
                let y = 2;
                y;
            }
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn break_outside_loop_error() {
    let source = r#"
        fn f() {
            break;
        }
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T131"));
}

#[test]
fn continue_outside_loop_error() {
    let source = r#"
        fn f() {
            continue;
        }
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T132"));
}

#[test]
fn break_inside_loop_ok() {
    let source = r#"
        fn f() {
            let items = [1, 2, 3];
            for item in items {
                break;
            }
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn self_outside_handler_error() {
    let result = check_with_errors("self;");
    assert!(has_error_code(&result, "T133"));
}

#[test]
fn self_inside_http_handler_ok() {
    let source = r#"
        get {
            self;
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn function_params_in_scope() {
    let source = r#"
        fn add(a: int, b: int): int {
            a + b
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn for_loop_binding_in_scope() {
    let source = r#"
        fn f() {
            let items = [1, 2, 3];
            for item in items {
                item;
            }
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn match_binding_in_arm_scope() {
    let source = r#"
        let x = Some(42);
        match x {
            Some(val) => val,
            _ => 0,
        };
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}
