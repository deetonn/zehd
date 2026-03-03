mod helpers;

use helpers::*;
use zehd_rune::op::{Instruction, Op};
use zehd_rune::value::Value;

#[test]
fn compile_list_literal() {
    let module = compile_module("const xs = [1, 2, 3];");
    let chunk = module.server_init.as_ref().unwrap();
    let ops = decode(chunk);

    assert!(ops.contains(&Instruction::U16(Op::MakeList, 3)));
    assert!(chunk.constants.contains(&Value::Int(1)));
    assert!(chunk.constants.contains(&Value::Int(2)));
    assert!(chunk.constants.contains(&Value::Int(3)));
}

#[test]
fn compile_empty_list() {
    let module = compile_module("const xs: List<int> = [];");
    let chunk = module.server_init.as_ref().unwrap();
    let ops = decode(chunk);

    assert!(ops.contains(&Instruction::U16(Op::MakeList, 0)));
}

#[test]
fn compile_object_literal() {
    let module = compile_module(
        r#"
        type User {
            name: string;
            age: int;
        }
        const u: User = { name: "Alice", age: 30 };
    "#,
    );
    let chunk = module.server_init.as_ref().unwrap();
    let ops = decode(chunk);

    // Should have MakeObject(2) for 2 key-value pairs.
    assert!(ops.contains(&Instruction::U16(Op::MakeObject, 2)));
}

#[test]
fn compile_field_access() {
    let module = compile_module(
        r#"
        type Point {
            x: int;
            y: int;
        }
        fn get_x(p: Point): int {
            p.x
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    assert!(ops.iter().any(|op| matches!(op, Instruction::U16(Op::GetField, _))));
    assert!(func.constants.contains(&Value::String("x".to_string())));
}

#[test]
fn compile_index_access() {
    let module = compile_module(
        r#"
        fn first(xs: List<int>): int {
            xs[0]
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    assert!(ops.contains(&Instruction::Simple(Op::GetIndex)));
}

#[test]
fn compile_list_in_function() {
    let module = compile_module(
        r#"
        fn make_list(): List<int> {
            [10, 20, 30]
        }
    "#,
    );

    let func = &module.functions[0].chunk;
    let ops = decode(func);

    assert!(ops.contains(&Instruction::U16(Op::MakeList, 3)));
}
