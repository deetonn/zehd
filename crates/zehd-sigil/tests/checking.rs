mod helpers;
use helpers::*;

#[test]
fn type_mismatch_int_string() {
    let result = check_with_errors("const count: int = \"hello\";");
    assert!(has_error_code(&result, "T110"));
}

#[test]
fn type_mismatch_binary_op() {
    let result = check_with_errors("let x = 1 + true;");
    assert!(has_error_code(&result, "T111"));
}

#[test]
fn type_mismatch_if_branches() {
    let source = r#"
        let x = if true { 1 } else { "hello" };
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T112"));
}

#[test]
fn non_boolean_condition() {
    let source = r#"
        let x = if 42 { 1 } else { 2 };
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T113"));
}

#[test]
fn wrong_number_of_arguments() {
    let source = r#"
        fn add(a: int, b: int): int { a + b }
        add(1);
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T114"));
}

#[test]
fn call_on_non_callable() {
    let source = r#"
        let x = 42;
        x(1);
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T115"));
}

#[test]
fn index_on_non_indexable() {
    let source = r#"
        let x = 42;
        x[0];
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T116"));
}

#[test]
fn assignment_to_const() {
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
fn assignment_to_let_ok() {
    let source = r#"
        fn f() {
            let x = 1;
            x = 2;
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn compatible_if_branches() {
    let source = r#"
        let x = if true { 1 } else { 2 };
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn string_concat_ok() {
    let source = r#"
        let x = "hello" + " " + "world";
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn unary_neg_on_string_error() {
    let result = check_with_errors("let x = -\"hello\";");
    assert!(has_error_code(&result, "T111"));
}

#[test]
fn unary_not_on_int_error() {
    let result = check_with_errors("let x = !42;");
    assert!(has_error_code(&result, "T113"));
}

#[test]
fn field_access_ok() {
    let source = r#"
        let obj = { name: "alice", age: 30 };
        obj.name;
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn undefined_field_error() {
    let source = r#"
        let obj = { name: "alice" };
        obj.missing;
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T104"));
}

#[test]
fn list_index_ok() {
    let source = r#"
        let xs = [1, 2, 3];
        xs[0];
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn function_call_type_checked() {
    let source = r#"
        fn add(a: int, b: int): int { a + b }
        let result = add(1, 2);
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn for_loop_non_iterable_error() {
    let source = r#"
        fn f() {
            for item in 42 {
                item;
            }
        }
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T116"));
}

#[test]
fn while_non_bool_condition_error() {
    let source = r#"
        fn f() {
            while 42 {
                break;
            }
        }
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T113"));
}

#[test]
fn heterogeneous_list_error() {
    let result = check_with_errors("let xs = [1, \"two\", 3];");
    assert!(has_error_code(&result, "T110"));
}

#[test]
fn annotated_var_checked_against_init() {
    let source = r#"
        let x: int = 42;
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn logical_non_bool_error() {
    let result = check_with_errors("let x = 1 && 2;");
    assert!(has_error_code(&result, "T113"));
}

#[test]
fn match_incompatible_arms_error() {
    let source = r#"
        let x = match 1 {
            1 => 42,
            _ => "hello",
        };
    "#;
    let result = check_with_errors(source);
    assert!(has_error_code(&result, "T119"));
}
