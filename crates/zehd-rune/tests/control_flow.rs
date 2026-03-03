mod helpers;

use helpers::*;
use zehd_rune::op::{Instruction, Op};

#[test]
fn compile_if_expression() {
    let module = compile_module(
        r#"
        fn test(x: int): int {
            if x > 0 { 1 } else { 0 }
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    assert!(ops.contains(&Instruction::Simple(Op::GtInt)));
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::JumpIfFalse, _))));
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::Jump, _))));
}

#[test]
fn compile_if_without_else() {
    let module = compile_module(
        r#"
        fn test(x: int) {
            if x > 0 { 1; }
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::JumpIfFalse, _))));
    // Missing else branch produces Unit.
    assert!(ops.contains(&Instruction::Simple(Op::Unit)));
}

#[test]
fn compile_while_loop() {
    let module = compile_module(
        r#"
        fn countdown(n: int) {
            let x = n;
            while x > 0 {
                x = x - 1;
            }
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    assert!(ops.contains(&Instruction::Simple(Op::GtInt)));
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::JumpIfFalse, _))));
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::Loop, _))));
}

#[test]
fn compile_for_loop() {
    let module = compile_module(
        r#"
        fn sum_list(items: List<int>): int {
            let total = 0;
            for item in items {
                total = total + item;
            }
            total
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    assert!(ops.contains(&Instruction::Simple(Op::GetIndex)));
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::Loop, _))));
}

#[test]
fn compile_break_in_while() {
    let module = compile_module(
        r#"
        fn test(limit: int) {
            let x = 0;
            while true {
                if x > limit {
                    break;
                }
                x = x + 1;
            }
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    // Break compiles to a Jump that gets patched to after the loop.
    assert!(ops.iter().filter(|op| matches!(op, Instruction::U16(Op::Jump, _))).count() >= 1);
}

#[test]
fn compile_continue_in_while() {
    let module = compile_module(
        r#"
        fn test(limit: int) {
            let x = 0;
            while x < limit {
                x = x + 1;
                if x == 5 {
                    continue;
                }
            }
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    // Continue compiles to a Loop (backward jump).
    let loop_count = ops
        .iter()
        .filter(|op| matches!(op, Instruction::U16(Op::Loop, _)))
        .count();
    assert!(loop_count >= 2); // One for continue, one for the while.
}

#[test]
fn compile_short_circuit_and() {
    let module = compile_module("fn f(a: bool, b: bool): bool { a && b }");
    let ops = decode(&module.functions[0].chunk);

    // && compiles to JumpIfFalse (short-circuit).
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::JumpIfFalse, _))));
}

#[test]
fn compile_short_circuit_or() {
    let module = compile_module("fn f(a: bool, b: bool): bool { a || b }");
    let ops = decode(&module.functions[0].chunk);

    // || compiles to JumpIfTrue (short-circuit).
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::JumpIfTrue, _))));
}
