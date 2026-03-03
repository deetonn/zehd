mod helpers;

use helpers::*;
use zehd_rune::op::{Instruction, Op};
use zehd_rune::value::Value;

#[test]
fn compile_let_with_initializer() {
    let module = compile_module("let x = 10;");
    let chunk = module.server_init.as_ref().unwrap();
    let ops = decode(chunk);

    // Should have: Constant(0), SetGlobal(0) [server scope = global]
    assert!(ops.contains(&Instruction::U16(Op::Constant, 0)));
    assert!(ops.contains(&Instruction::U16(Op::SetGlobal, 0)));
    assert_eq!(chunk.constants[0], Value::Int(10));
}

#[test]
fn compile_const_with_initializer() {
    let module = compile_module("const x = 99;");
    let chunk = module.server_init.as_ref().unwrap();
    let ops = decode(chunk);

    assert!(ops.contains(&Instruction::U16(Op::Constant, 0)));
    assert!(ops.contains(&Instruction::U16(Op::SetGlobal, 0)));
}

#[test]
fn compile_variable_read_in_handler() {
    // Use `let` to avoid const-inlining by the optimizer.
    let module = compile_module(
        r#"
        let greeting = "hello";
        get {
            greeting;
        }
    "#,
    );

    // greeting is a global.
    let handler = &module.handlers[0].chunk;
    let ops = decode(handler);

    // Should read the global.
    assert!(ops.contains(&Instruction::U16(Op::GetGlobal, 0)));
}

#[test]
fn compile_local_in_function() {
    let module = compile_module(
        r#"
        fn add(a: int, b: int): int {
            let sum = a + b;
            sum
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    assert_eq!(func.name, "add");
    assert_eq!(func.arity, 2);

    let ops = decode(func);
    // Should have GetLocal for params, SetLocal for sum.
    assert!(ops.contains(&Instruction::U16(Op::GetLocal, 0))); // a
    assert!(ops.contains(&Instruction::U16(Op::GetLocal, 1))); // b
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::SetLocal, 2)))); // sum
}

#[test]
fn compile_scope_cleanup() {
    // Variables declared in inner scopes should not leak.
    let module = compile_module(
        r#"
        fn test(): int {
            let x = 1;
            {
                let y = 2;
            }
            x
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    // x should be accessible, y was in inner scope.
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::GetLocal, 0)))); // x
}
