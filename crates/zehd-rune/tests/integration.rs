mod helpers;

use helpers::*;
use zehd_codex::ast::HttpMethod;
use zehd_rune::op::{Instruction, Op};
use zehd_rune::value::Value;

#[test]
fn compile_full_route_file() {
    let module = compile_module(
        r#"
        const version = "1.0";

        fn format_response(data: string): string {
            $"[v{version}] {data}"
        }

        get {
            format_response("hello")
        }

        post {
            let body = "received";
            format_response(body)
        }
    "#,
    );

    // Should have server_init for `const version`.
    assert!(module.server_init.is_some());
    let init = module.server_init.as_ref().unwrap();
    assert!(init.constants.contains(&Value::String("1.0".to_string())));

    // Should have 1 function.
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "format_response");

    // Should have 2 handlers.
    assert_eq!(module.handlers.len(), 2);
    assert_eq!(module.handlers[0].method, HttpMethod::Get);
    assert_eq!(module.handlers[1].method, HttpMethod::Post);
}

#[test]
fn compile_init_block() {
    let module = compile_module(
        r#"
        init {
            let port = 8080;
        }
    "#,
    );

    assert!(module.init_block.is_some());
    let init = module.init_block.as_ref().unwrap();
    assert_eq!(init.name, "init");
    assert!(init.constants.contains(&Value::Int(8080)));
}

#[test]
fn compile_error_handler() {
    let module = compile_module(
        r#"
        error(e) {
            e;
        }
    "#,
    );

    assert!(module.error_handler.is_some());
    let handler = module.error_handler.as_ref().unwrap();
    assert_eq!(handler.name, "error");
    assert_eq!(handler.arity, 1);
}

#[test]
fn compile_function_with_locals() {
    let module = compile_module(
        r#"
        fn fibonacci(n: int): int {
            if n <= 1 { n }
            else { fibonacci(n - 1) + fibonacci(n - 2) }
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    assert_eq!(func.arity, 1);
    let ops = decode(func);

    // Should have recursive calls.
    let call_count = ops
        .iter()
        .filter(|op| matches!(op, Instruction::U8(Op::Call, 1)))
        .count();
    assert_eq!(call_count, 2);
}

#[test]
fn compile_option_some() {
    let module = compile_module(
        r#"
        fn wrap(x: int): Option<int> {
            Some(x)
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    assert!(ops.contains(&Instruction::Simple(Op::WrapSome)));
}

#[test]
fn compile_result_ok_err() {
    let module = compile_module(
        r#"
        fn validate(x: int): Result<int, string> {
            if x > 0 { Ok(x) }
            else { Err("must be positive") }
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    assert!(ops.contains(&Instruction::Simple(Op::WrapOk)));
    assert!(ops.contains(&Instruction::Simple(Op::WrapErr)));
}

#[test]
fn compile_try_operator() {
    let module = compile_module(
        r#"
        fn parse_input(x: Result<int, string>): Result<int, string> {
            let val = x?;
            Ok(val + 1)
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    assert!(ops.contains(&Instruction::Simple(Op::TryOp)));
}

#[test]
fn compile_handler_self_field_access() {
    let module = compile_module(
        r#"
        get {
            let req = self.request;
            req
        }
    "#,
    );

    let chunk = &module.handlers[0].chunk;
    let ops = decode(chunk);

    assert!(ops.contains(&Instruction::Simple(Op::GetSelf)));
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::GetField, _))));
}

#[test]
fn compile_assignment() {
    let module = compile_module(
        r#"
        fn counter(): int {
            let x = 0;
            x = x + 1;
            x = x + 1;
            x
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    // Should have multiple SetLocal for assignments.
    let set_local_count = ops
        .iter()
        .filter(|op| matches!(op, Instruction::U16(Op::SetLocal, _)))
        .count();
    assert!(set_local_count >= 3); // Initial + 2 assignments.
}

#[test]
fn compile_nested_if() {
    let module = compile_module(
        r#"
        fn classify(x: int): string {
            if x > 0 {
                if x > 100 {
                    "big"
                } else {
                    "small"
                }
            } else {
                "negative"
            }
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    // Should have multiple JumpIfFalse for nested ifs.
    let jump_count = ops
        .iter()
        .filter(|op| matches!(op, Instruction::U16(Op::JumpIfFalse, _)))
        .count();
    assert!(jump_count >= 2);
}

#[test]
fn compile_empty_program() {
    let module = compile_module("");
    assert!(module.server_init.is_none());
    assert!(module.handlers.is_empty());
    assert!(module.functions.is_empty());
    assert!(module.init_block.is_none());
    assert!(module.error_handler.is_none());
}

#[test]
fn compile_server_scope_and_handler_scope() {
    // Use `let` instead of `const` to avoid const-inlining by optimizer.
    let module = compile_module(
        r#"
        let base_url = "https://api.example.com";

        fn make_url(path: string): string {
            base_url + path
        }

        get {
            make_url("/users")
        }
    "#,
    );

    // Server init should set the global.
    assert!(module.server_init.is_some());

    // Function should reference the global.
    let func = &module.functions[0].chunk;
    let ops = decode(func);
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::GetGlobal, _))));

    // Handler should call the function.
    let handler = &module.handlers[0].chunk;
    let ops = decode(handler);
    assert!(ops.iter().any(|op| matches!(op, Instruction::U8(Op::Call, 1))));
}
