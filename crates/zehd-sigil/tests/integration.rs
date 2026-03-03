mod helpers;
use helpers::*;

#[test]
fn full_route_file() {
    let source = r#"
        type User {
            name: string;
            age: int;
        }

        fn create_user(name: string, age: int): User {
            { name, age }
        }

        get {
            let user = create_user("alice", 30);
            user.name;
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn function_with_control_flow() {
    let source = r#"
        fn max(a: int, b: int): int {
            if a > b { a } else { b }
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn http_handler_with_self() {
    let source = r#"
        get {
            let method = self.request.method;
            let path = self.request.path;
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn loop_with_list() {
    let source = r#"
        fn f() {
            let numbers = [1, 2, 3, 4, 5];
            let sum = 0;
            for n in numbers {
                n;
            }
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn while_loop_ok() {
    let source = r#"
        fn f() {
            let x = 0;
            while x < 10 {
                x;
            }
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn match_expression_ok() {
    let source = r#"
        let status = 200;
        let msg = match status {
            200 => "ok",
            404 => "not found",
            _ => "error",
        };
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn result_with_try_operator() {
    let source = r#"
        fn process(): Result<int, string> {
            let x = Ok(42);
            let val = x?;
            Ok(val + 1)
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn arrow_function_as_callback() {
    let source = r#"
        let double = (x: int) => x * 2;
        let result = double(21);
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn nested_object_and_field_access() {
    let source = r#"
        let config = {
            port: 8080,
            host: "localhost",
        };
        config.port;
        config.host;
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn list_operations() {
    let source = r#"
        let xs = [10, 20, 30];
        let first = xs[0];
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn error_handler_block() {
    let source = r#"
        error(e) {
            e;
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn init_block() {
    let source = r#"
        init {
            let port = 8080;
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn optimized_program_returned() {
    let source = r#"
        const x = 1 + 2;
    "#;
    let result = check_ok(source);
    assert!(result.optimized_program.is_some());
}

#[test]
fn interpolated_string_full() {
    let source = r#"
        let name = "world";
        let age = 30;
        let msg = $"hello {name}, age {age}";
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn block_expression_as_value() {
    let source = r#"
        let x = {
            let a = 1;
            let b = 2;
            a + b
        };
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}

#[test]
fn complex_program_with_errors() {
    let source = r#"
        fn validate(input: string): Result<int, string> {
            Ok(42)
        }

        get {
            let result = validate("test");
            let value = result?;
            value;
        }
    "#;
    let result = check_ok(source);
    assert!(result.is_ok());
}
