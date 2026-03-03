mod helpers;

use helpers::*;
use zehd_rune::op::{Instruction, Op};
use zehd_rune::value::Value;

#[test]
fn compile_interpolated_string() {
    let module = compile_module(
        r#"
        fn greet(name: string): string {
            $"Hello, {name}!"
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    // Should have: Constant("Hello, "), GetLocal(name), ToString, Constant("!"), Concat(3).
    assert!(func.constants.contains(&Value::String("Hello, ".to_string())));
    assert!(func.constants.contains(&Value::String("!".to_string())));
    assert!(ops.contains(&Instruction::Simple(Op::ToString)));
    assert!(ops.contains(&Instruction::U16(Op::Concat, 3)));
}

#[test]
fn compile_simple_string() {
    let module = compile_module(r#"const x = "hello world";"#);
    let chunk = module.server_init.as_ref().unwrap();

    assert!(chunk.constants.contains(&Value::String("hello world".to_string())));
}

#[test]
fn compile_interpolated_string_single_expr() {
    let module = compile_module(
        r#"
        fn to_str(x: int): string {
            $"{x}"
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    // Single part: just ToString, no Concat needed (count=1).
    assert!(ops.contains(&Instruction::Simple(Op::ToString)));
}

#[test]
fn compile_string_concat_via_plus() {
    // Use function params to avoid constant folding.
    let module = compile_module(
        r#"fn f(a: string, b: string, c: string): string { a + b + c }"#,
    );
    let ops = decode(&module.functions[0].chunk);

    // "+" on strings compiles to AddStr.
    let add_str_count = ops
        .iter()
        .filter(|op| matches!(op, Instruction::Simple(Op::AddStr)))
        .count();
    assert_eq!(add_str_count, 2); // Two + operations.
}
