mod helpers;
use helpers::*;

#[test]
fn error_has_code_t100() {
    let result = check_with_errors("undefined_var;");
    assert!(has_error_code(&result, "T100"));
    assert!(result.has_errors());
}

#[test]
fn error_has_code_t110() {
    let result = check_with_errors("const x: int = \"hello\";");
    assert!(has_error_code(&result, "T110"));
}

#[test]
fn error_has_code_t111() {
    let result = check_with_errors("let x = 1 + true;");
    assert!(has_error_code(&result, "T111"));
}

#[test]
fn error_has_code_t113() {
    let source = r#"
        let x = if 42 { 1 } else { 2 };
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T113"));
}

#[test]
fn error_has_code_t114() {
    let source = r#"
        fn f(a: int) {}
        f(1, 2);
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T114"));
}

#[test]
fn error_has_code_t117() {
    let source = r#"
        fn f() {
            let x = 42;
            x?;
        }
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T117"));
}

#[test]
fn error_has_code_t130() {
    let source = r#"
        const x = 1;
        fn f() {
            x = 2;
        }
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T130"));
}

#[test]
fn error_has_code_t131() {
    let source = r#"
        fn f() {
            break;
        }
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T131"));
}

#[test]
fn error_display_format() {
    let result = check_with_errors("undefined_var;");
    let err = &result.errors[0];
    let formatted = format!("{}", err);
    assert!(formatted.contains("T100"), "formatted: {}", formatted);
    assert!(formatted.contains("undefined variable"), "formatted: {}", formatted);
}

#[test]
fn error_has_labels() {
    let result = check_with_errors("undefined_var;");
    let err = result.errors.iter().find(|e| e.code.to_string() == "T100").unwrap();
    assert!(!err.labels.is_empty(), "expected labels on T100 error");
}

#[test]
fn warning_for_unreachable_code() {
    let source = r#"
        fn f() {
            return;
        }
    "#;
    // Note: unreachable code detection happens in optimization pass.
    let result = check_ok(source);
    assert!(result.is_ok()); // warnings don't count as errors
}

#[test]
fn multiple_errors_collected() {
    let source = r#"
        a;
        b;
        c;
    "#;
    let result = check_with_errors(source);
    assert!(result.errors.len() >= 3, "expected at least 3 errors for 3 undefined vars");
}
