mod helpers;

use helpers::*;
use zehd_codex::ast::HttpMethod;
use zehd_rune::op::{Instruction, Op};
use zehd_rune::value::Value;

#[test]
fn compile_get_handler() {
    let module = compile_module(
        r#"
        get {
            "hello";
        }
    "#,
    );

    assert_eq!(module.handlers.len(), 1);
    assert_eq!(module.handlers[0].method, HttpMethod::Get);

    let chunk = &module.handlers[0].chunk;
    assert_eq!(chunk.name, "get");
    assert_eq!(chunk.arity, 0);
}

#[test]
fn compile_post_handler() {
    let module = compile_module(
        r#"
        post {
            "created";
        }
    "#,
    );

    assert_eq!(module.handlers.len(), 1);
    assert_eq!(module.handlers[0].method, HttpMethod::Post);
    assert_eq!(module.handlers[0].chunk.name, "post");
}

#[test]
fn compile_multiple_handlers() {
    let module = compile_module(
        r#"
        get {
            "get response";
        }
        post {
            "post response";
        }
        delete {
            "deleted";
        }
    "#,
    );

    assert_eq!(module.handlers.len(), 3);
    assert_eq!(module.handlers[0].method, HttpMethod::Get);
    assert_eq!(module.handlers[1].method, HttpMethod::Post);
    assert_eq!(module.handlers[2].method, HttpMethod::Delete);
}

#[test]
fn compile_handler_with_self() {
    let module = compile_module(
        r#"
        get {
            self;
        }
    "#,
    );

    let chunk = &module.handlers[0].chunk;
    let ops = decode(chunk);

    assert!(ops.contains(&Instruction::Simple(Op::GetSelf)));
}

#[test]
fn compile_handler_with_logic() {
    let module = compile_module(
        r#"
        fn compute(a: int, b: int): int { a + b }
        get {
            let x = compute(1, 2);
            x
        }
    "#,
    );

    let chunk = &module.handlers[0].chunk;
    let ops = decode(chunk);

    // Should call the function and return the result.
    assert!(ops.iter().any(|op| matches!(op, Instruction::U8(Op::Call, _))));
    assert!(ops.contains(&Instruction::Simple(Op::Return)));
}

#[test]
fn compile_handler_return_value() {
    let module = compile_module(
        r#"
        get {
            let result = "success";
            result
        }
    "#,
    );

    let chunk = &module.handlers[0].chunk;
    assert!(chunk.constants.contains(&Value::String("success".to_string())));
}
