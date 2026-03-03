mod helpers;

use helpers::*;
use zehd_rune::op::{Instruction, Op};

#[test]
fn compile_match_with_literals() {
    let module = compile_module(
        r#"
        fn describe(x: int): string {
            match x {
                1 => "one",
                2 => "two",
                _ => "other",
            }
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    // Should have Dup + TestEqual + JumpIfFalse for literal arms.
    assert!(ops.contains(&Instruction::Simple(Op::Dup)));
    assert!(ops.contains(&Instruction::Simple(Op::TestEqual)));
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::JumpIfFalse, _))));
}

#[test]
fn compile_match_with_binding() {
    let module = compile_module(
        r#"
        fn identity(x: int): int {
            match x {
                n => n,
            }
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    // Binding arm: Dup, SetLocal, body (GetLocal), Return.
    assert!(ops.contains(&Instruction::Simple(Op::Dup)));
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::SetLocal, _))));
}

#[test]
fn compile_match_with_wildcard() {
    let module = compile_module(
        r#"
        fn test(x: int): int {
            match x {
                _ => 42,
            }
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    // Wildcard doesn't need a test — just the body.
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::Constant, _))));
}

#[test]
fn compile_match_option() {
    let module = compile_module(
        r#"
        fn unwrap_or(x: Option<int>): int {
            match x {
                Some(v) => v,
                None => 0,
            }
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    // Should have TestVariant for Some and None.
    assert!(ops.iter().any(|op| matches!(op, Instruction::U16U16(Op::TestVariant, _, _))));
    // Should have UnwrapVariant for the Some(v) binding.
    assert!(ops.contains(&Instruction::Simple(Op::UnwrapVariant)));
}
