#![allow(dead_code)]

use std::sync::Arc;

use zehd_rune::module::CompiledModule;
use zehd_rune::value::Value;
use zehd_ward::error::RuntimeError;
use zehd_ward::vm::StackVm;
use zehd_ward::Context;

/// Compile source through the full pipeline (lex -> parse -> check -> compile).
/// Panics on parse, type, or compile errors.
pub fn compile_module(source: &str) -> CompiledModule {
    let parse_result = zehd_codex::parse(source);
    if !parse_result.is_ok() {
        let msgs: Vec<String> =
            parse_result.errors.iter().map(|e| format!("  {e}")).collect();
        panic!("parse errors:\n{}", msgs.join("\n"));
    }
    let check_result = zehd_sigil::check(&parse_result.program, source, &Default::default());
    if check_result.has_errors() {
        let msgs: Vec<String> =
            check_result.errors.iter().map(|e| format!("  {e}")).collect();
        panic!("type errors:\n{}", msgs.join("\n"));
    }
    let compile_result = zehd_rune::compile(&parse_result.program, check_result, &Default::default());
    if compile_result.has_errors() {
        let msgs: Vec<String> =
            compile_result.errors.iter().map(|e| format!("  {e}")).collect();
        panic!("compile errors:\n{}", msgs.join("\n"));
    }
    compile_result.module
}

/// Compile source and call the function at the given index with provided args.
pub fn call_fn(source: &str, func_index: u16, args: Vec<Value>) -> Value {
    let module = compile_module(source);
    let context = Context { module, native_fns: Arc::new(vec![]) };
    let mut vm = StackVm::new();
    vm.call_function(func_index, args, &context)
        .unwrap_or_else(|e| panic!("runtime error: {e}"))
}

/// Compile source and call the first function (index 0) with provided args.
pub fn call_fn0(source: &str, args: Vec<Value>) -> Value {
    call_fn(source, 0, args)
}

/// Compile source and execute the server_init chunk.
pub fn run_init(source: &str) -> Value {
    let module = compile_module(source);
    let chunk = module
        .server_init
        .as_ref()
        .expect("no server_init chunk");
    let context = Context { module: module.clone(), native_fns: Arc::new(vec![]) };
    let mut vm = StackVm::new();
    use zehd_ward::VmBackend;
    vm.execute(chunk, &context)
        .unwrap_or_else(|e| panic!("runtime error: {e}"))
}

/// Compile source, run server_init, then execute handler at index.
pub fn run_handler(source: &str, handler_index: usize) -> Value {
    let module = compile_module(source);
    let context = Context { module, native_fns: Arc::new(vec![]) };
    let mut vm = StackVm::new();

    // Run server_init first if present
    if let Some(chunk) = &context.module.server_init {
        use zehd_ward::VmBackend;
        vm.execute(chunk, &context)
            .unwrap_or_else(|e| panic!("runtime error in server_init: {e}"));
    }

    let self_value = Value::Object(vec![
        ("request".to_string(), Value::Object(vec![
            ("method".to_string(), Value::String("GET".to_string())),
            ("path".to_string(), Value::String("/".to_string())),
            ("headers".to_string(), Value::Object(vec![])),
            ("body".to_string(), Value::String(String::new())),
            ("query".to_string(), Value::String(String::new())),
        ])),
        ("response".to_string(), Value::Object(vec![
            ("status".to_string(), Value::Int(200)),
        ])),
        ("params".to_string(), Value::Object(vec![])),
    ]);
    vm.execute_handler(handler_index, &context, self_value)
        .unwrap_or_else(|e| panic!("runtime error in handler: {e}"))
}

/// Compile source and call the first function, expecting a RuntimeError.
pub fn call_fn0_err(source: &str, args: Vec<Value>) -> RuntimeError {
    let module = compile_module(source);
    let context = Context { module, native_fns: Arc::new(vec![]) };
    let mut vm = StackVm::new();
    vm.call_function(0, args, &context)
        .expect_err("expected runtime error but got success")
}

/// Get a fresh VM and context from source.
pub fn vm_and_context(source: &str) -> (StackVm, Context) {
    let module = compile_module(source);
    let context = Context { module, native_fns: Arc::new(vec![]) };
    let vm = StackVm::new();
    (vm, context)
}
