mod helpers;

use helpers::*;
use zehd_rune::op::{Instruction, Op};

#[test]
fn compile_simple_function() {
    let module = compile_module(
        r#"
        fn greet(): string {
            "hello"
        }
    "#,
    );

    assert_eq!(module.functions.len(), 1);
    let func = &module.functions[0];
    assert_eq!(func.name, "greet");
    assert_eq!(func.chunk.arity, 0);

    let ops = decode(&func.chunk);
    assert!(ops.contains(&Instruction::Simple(Op::Return)));
}

#[test]
fn compile_function_with_params() {
    let module = compile_module(
        r#"
        fn add(a: int, b: int): int {
            a + b
        }
    "#,
    );

    let func = &module.functions[0];
    assert_eq!(func.name, "add");
    assert_eq!(func.chunk.arity, 2);

    let ops = decode(&func.chunk);
    assert!(ops.contains(&Instruction::U16(Op::GetLocal, 0))); // a
    assert!(ops.contains(&Instruction::U16(Op::GetLocal, 1))); // b
    assert!(ops.contains(&Instruction::Simple(Op::AddInt)));
    assert!(ops.contains(&Instruction::Simple(Op::Return)));
}

#[test]
fn compile_function_with_return_stmt() {
    let module = compile_module(
        r#"
        fn early(x: int): int {
            if x > 0 {
                return x;
            }
            0
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    // Should have explicit Return for early return.
    let return_count = ops
        .iter()
        .filter(|op| matches!(op, Instruction::Simple(Op::Return)))
        .count();
    assert!(return_count >= 1);
}

#[test]
fn compile_function_call() {
    let module = compile_module(
        r#"
        fn double(x: int): int {
            x * 2
        }
        fn test(): int {
            double(21)
        }
    "#,
    );

    assert_eq!(module.functions.len(), 2);
    let test_fn = &module.functions[1].chunk;
    let ops = decode(test_fn);

    // Should have Closure (function ref) + Constant (arg) + Call.
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::Closure, 0))));
    assert!(ops.contains(&Instruction::U8(Op::Call, 1)));
}

#[test]
fn compile_arrow_function() {
    let module = compile_module(
        r#"
        const double = (x: int): int => x * 2;
    "#,
    );

    // Arrow functions are compiled as separate function entries.
    assert!(module.functions.len() >= 1);

    let server_init = module.server_init.as_ref().unwrap();
    let ops = decode(server_init);

    // Should have Closure referencing the arrow function.
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::Closure, _))));
}

#[test]
fn compile_arrow_function_with_block_body() {
    let module = compile_module(
        r#"
        const compute = (x: int): int => {
            let y = x + 1;
            y * 2
        };
    "#,
    );

    assert!(module.functions.len() >= 1);
    let arrow_fn = &module.functions[0].chunk;
    let ops = decode(arrow_fn);

    assert!(ops.contains(&Instruction::Simple(Op::AddInt)));
    assert!(ops.contains(&Instruction::Simple(Op::MulInt)));
}

#[test]
fn compile_multiple_functions() {
    let module = compile_module(
        r#"
        fn first(): int { 1 }
        fn second(): int { 2 }
        fn third(): int { 3 }
    "#,
    );

    assert_eq!(module.functions.len(), 3);
    assert_eq!(module.functions[0].name, "first");
    assert_eq!(module.functions[1].name, "second");
    assert_eq!(module.functions[2].name, "third");
}

#[test]
fn compile_recursive_function() {
    let module = compile_module(
        r#"
        fn factorial(n: int): int {
            if n <= 1 { 1 }
            else { n * factorial(n - 1) }
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    // Should reference itself via Closure and Call.
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::Closure, 0))));
    assert!(ops.iter().any(|op| matches!(op, Instruction::U8(Op::Call, 1))));
}
