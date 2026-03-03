mod helpers;

use helpers::*;
use zehd_rune::op::{Instruction, Op};
use zehd_rune::value::Value;

#[test]
fn compile_int_literal() {
    let module = compile_module("const x = 42;");
    let chunk = module.server_init.as_ref().unwrap();
    let ops = decode(chunk);

    assert!(ops.contains(&Instruction::U16(Op::Constant, 0)));
    assert_eq!(chunk.constants[0], Value::Int(42));
}

#[test]
fn compile_float_literal() {
    let module = compile_module("const x = 3.14;");
    let chunk = module.server_init.as_ref().unwrap();

    assert_eq!(chunk.constants[0], Value::Float(3.14));
}

#[test]
fn compile_string_literal() {
    let module = compile_module("const x = \"hello\";");
    let chunk = module.server_init.as_ref().unwrap();

    assert_eq!(chunk.constants[0], Value::String("hello".to_string()));
}

#[test]
fn compile_bool_true() {
    let module = compile_module("const x = true;");
    let chunk = module.server_init.as_ref().unwrap();
    let ops = decode(chunk);

    assert!(ops.contains(&Instruction::Simple(Op::True)));
}

#[test]
fn compile_bool_false() {
    let module = compile_module("const x = false;");
    let chunk = module.server_init.as_ref().unwrap();
    let ops = decode(chunk);

    assert!(ops.contains(&Instruction::Simple(Op::False)));
}

#[test]
fn compile_none_literal() {
    let module = compile_module("const x: Option<int> = None;");
    let chunk = module.server_init.as_ref().unwrap();
    let ops = decode(chunk);

    assert!(ops.contains(&Instruction::Simple(Op::None)));
}

#[test]
fn compile_time_literal() {
    let module = compile_module("const x = 5s;");
    let chunk = module.server_init.as_ref().unwrap();

    // 5s = 5000ms
    assert_eq!(chunk.constants[0], Value::Int(5000));
}

#[test]
fn compile_constant_deduplication() {
    let module = compile_module("const x = 42;\nconst y = 42;");
    let chunk = module.server_init.as_ref().unwrap();

    // Should only have one constant for 42.
    let int_constants: Vec<_> = chunk
        .constants
        .iter()
        .filter(|v| matches!(v, Value::Int(42)))
        .collect();
    assert_eq!(int_constants.len(), 1);
}
