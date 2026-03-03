mod helpers;

use helpers::*;
use zehd_rune::op::{Instruction, Op};

// NOTE: We use function parameters for operands because the optimizer
// constant-folds literal binary expressions like `1 + 2` → `3`.

#[test]
fn compile_int_addition() {
    let module = compile_module("fn f(a: int, b: int): int { a + b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::AddInt)));
}

#[test]
fn compile_int_subtraction() {
    let module = compile_module("fn f(a: int, b: int): int { a - b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::SubInt)));
}

#[test]
fn compile_int_multiplication() {
    let module = compile_module("fn f(a: int, b: int): int { a * b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::MulInt)));
}

#[test]
fn compile_int_division() {
    let module = compile_module("fn f(a: int, b: int): int { a / b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::DivInt)));
}

#[test]
fn compile_int_modulo() {
    let module = compile_module("fn f(a: int, b: int): int { a % b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::ModInt)));
}

#[test]
fn compile_float_addition() {
    let module = compile_module("fn f(a: float, b: float): float { a + b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::AddFloat)));
}

#[test]
fn compile_float_subtraction() {
    let module = compile_module("fn f(a: float, b: float): float { a - b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::SubFloat)));
}

#[test]
fn compile_float_multiplication() {
    let module = compile_module("fn f(a: float, b: float): float { a * b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::MulFloat)));
}

#[test]
fn compile_float_division() {
    let module = compile_module("fn f(a: float, b: float): float { a / b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::DivFloat)));
}

#[test]
fn compile_string_concatenation() {
    let module = compile_module("fn f(a: string, b: string): string { a + b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::AddStr)));
}

#[test]
fn compile_int_negation() {
    let module = compile_module("fn f(a: int): int { -a }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::NegInt)));
}

#[test]
fn compile_float_negation() {
    let module = compile_module("fn f(a: float): float { -a }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::NegFloat)));
}

#[test]
fn compile_boolean_not() {
    let module = compile_module("fn f(a: bool): bool { !a }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::Not)));
}

#[test]
fn compile_int_comparison_eq() {
    let module = compile_module("fn f(a: int, b: int): bool { a == b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::EqInt)));
}

#[test]
fn compile_int_comparison_neq() {
    let module = compile_module("fn f(a: int, b: int): bool { a != b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::NeqInt)));
}

#[test]
fn compile_int_comparison_lt() {
    let module = compile_module("fn f(a: int, b: int): bool { a < b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::LtInt)));
}

#[test]
fn compile_int_comparison_gt() {
    let module = compile_module("fn f(a: int, b: int): bool { a > b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::GtInt)));
}

#[test]
fn compile_int_comparison_leq() {
    let module = compile_module("fn f(a: int, b: int): bool { a <= b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::LeqInt)));
}

#[test]
fn compile_int_comparison_geq() {
    let module = compile_module("fn f(a: int, b: int): bool { a >= b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::GeqInt)));
}

#[test]
fn compile_float_comparison_gt() {
    let module = compile_module("fn f(a: float, b: float): bool { a > b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::GtFloat)));
}

#[test]
fn compile_float_comparison_lt() {
    let module = compile_module("fn f(a: float, b: float): bool { a < b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::LtFloat)));
}

#[test]
fn compile_string_equality() {
    let module = compile_module("fn f(a: string, b: string): bool { a == b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::EqStr)));
}

#[test]
fn compile_string_neq() {
    let module = compile_module("fn f(a: string, b: string): bool { a != b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::NeqStr)));
}

#[test]
fn compile_bool_equality() {
    let module = compile_module("fn f(a: bool, b: bool): bool { a == b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::EqBool)));
}

#[test]
fn compile_bool_neq() {
    let module = compile_module("fn f(a: bool, b: bool): bool { a != b }");
    let ops = decode(&module.functions[0].chunk);
    assert!(ops.contains(&Instruction::Simple(Op::NeqBool)));
}
