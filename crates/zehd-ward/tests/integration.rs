mod helpers;

use std::sync::Arc;

use zehd_rune::registry::NativeRegistry;
use zehd_rune::value::Value;
use zehd_sigil::ModuleTypes;

// ── Server Init + Handler ──────────────────────────────────────

#[test]
fn server_init_then_handler() {
    let result = helpers::run_handler(
        r#"
        const version = "1.0";
        get {
            version
        }
        "#,
        0,
    );
    assert_eq!(result, Value::String("1.0".into()));
}

#[test]
fn multiple_handlers() {
    let source = r#"
        get {
            "hello"
        }
        post {
            "created"
        }
    "#;
    let get_result = helpers::run_handler(source, 0);
    assert_eq!(get_result, Value::String("hello".into()));

    let post_result = helpers::run_handler(source, 1);
    assert_eq!(post_result, Value::String("created".into()));
}

// ── Handler Calling Function ───────────────────────────────────

#[test]
fn handler_calls_function() {
    let result = helpers::run_handler(
        r#"
        fn greet(): string {
            "hello world"
        }
        get {
            greet()
        }
        "#,
        0,
    );
    assert_eq!(result, Value::String("hello world".into()));
}

#[test]
fn handler_calls_function_with_global() {
    let result = helpers::run_handler(
        r#"
        const base_url = "https://api.example.com";
        fn make_url(path: string): string {
            base_url + path
        }
        get {
            make_url("/users")
        }
        "#,
        0,
    );
    assert_eq!(result, Value::String("https://api.example.com/users".into()));
}

// ── Multi-Function Programs ────────────────────────────────────

#[test]
fn multi_function_program() {
    let result = helpers::call_fn(
        r#"
        fn add(a: int, b: int): int { a + b }
        fn double(x: int): int { x * 2 }
        fn compute(x: int, y: int): int {
            double(add(x, y))
        }
        "#,
        2,
        vec![Value::Int(3), Value::Int(4)],
    );
    // add(3,4) = 7, double(7) = 14
    assert_eq!(result, Value::Int(14));
}

// ── Complex Programs ───────────────────────────────────────────

#[test]
fn iterative_factorial() {
    let result = helpers::call_fn0(
        r#"fn factorial(n: int): int {
            let result = 1;
            let i = 2;
            while i <= n {
                result = result * i;
                i = i + 1;
            }
            result
        }"#,
        vec![Value::Int(6)],
    );
    assert_eq!(result, Value::Int(720));
}

#[test]
fn fizzbuzz_single() {
    let result = helpers::call_fn0(
        r#"fn fizzbuzz(n: int): string {
            if n % 15 == 0 {
                "FizzBuzz"
            } else if n % 3 == 0 {
                "Fizz"
            } else if n % 5 == 0 {
                "Buzz"
            } else {
                $"{n}"
            }
        }"#,
        vec![Value::Int(15)],
    );
    assert_eq!(result, Value::String("FizzBuzz".into()));
}

#[test]
fn fizzbuzz_fizz() {
    let result = helpers::call_fn0(
        r#"fn fizzbuzz(n: int): string {
            if n % 15 == 0 {
                "FizzBuzz"
            } else if n % 3 == 0 {
                "Fizz"
            } else if n % 5 == 0 {
                "Buzz"
            } else {
                $"{n}"
            }
        }"#,
        vec![Value::Int(9)],
    );
    assert_eq!(result, Value::String("Fizz".into()));
}

#[test]
fn fizzbuzz_number() {
    let result = helpers::call_fn0(
        r#"fn fizzbuzz(n: int): string {
            if n % 15 == 0 {
                "FizzBuzz"
            } else if n % 3 == 0 {
                "Fizz"
            } else if n % 5 == 0 {
                "Buzz"
            } else {
                $"{n}"
            }
        }"#,
        vec![Value::Int(7)],
    );
    assert_eq!(result, Value::String("7".into()));
}

// ── Globals Persist Across Calls ───────────────────────────────

#[test]
fn globals_persist_across_handler_calls() {
    let module = helpers::compile_module(
        r#"
        const greeting = "hi";
        get {
            greeting
        }
        post {
            greeting
        }
        "#,
    );
    let context = zehd_ward::Context { module, native_fns: std::sync::Arc::new(vec![]) };
    let mut vm = zehd_ward::vm::StackVm::new();

    // Run server_init to set up globals
    if let Some(chunk) = &context.module.server_init {
        use zehd_ward::VmBackend;
        vm.execute(chunk, &context).unwrap();
    }

    let self_value = || Value::Object(vec![
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

    // First handler call
    let r1 = vm.execute_handler(0, &context, self_value()).unwrap();
    assert_eq!(r1, Value::String("hi".into()));

    // Second handler call — globals should still be there
    let r2 = vm.execute_handler(1, &context, self_value()).unwrap();
    assert_eq!(r2, Value::String("hi".into()));
}

// ── Per-Request Globals Isolation ─────────────────────────────────

#[test]
fn globals_snapshot_isolates_requests() {
    // A handler that mutates a server-scope `let` should not affect subsequent requests.
    let module = helpers::compile_module(
        r#"
        let counter = 0;
        get {
            counter = counter + 1;
            counter
        }
        "#,
    );
    let context = zehd_ward::Context { module, native_fns: std::sync::Arc::new(vec![]) };

    // Run server_init to populate globals
    let mut init_vm = zehd_ward::vm::StackVm::new();
    if let Some(chunk) = &context.module.server_init {
        use zehd_ward::VmBackend;
        init_vm.execute(chunk, &context).unwrap();
    }
    let globals_snapshot = init_vm.globals().to_vec();

    let self_value = || Value::Object(vec![
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

    // Request 1: fresh VM from snapshot
    let mut vm1 = zehd_ward::vm::StackVm::with_globals(globals_snapshot.clone());
    let r1 = vm1.execute_handler(0, &context, self_value()).unwrap();
    assert_eq!(r1, Value::Int(1), "first request should see counter = 1");

    // Request 2: fresh VM from same snapshot — mutation did NOT leak
    let mut vm2 = zehd_ward::vm::StackVm::with_globals(globals_snapshot.clone());
    let r2 = vm2.execute_handler(0, &context, self_value()).unwrap();
    assert_eq!(r2, Value::Int(1), "second request should also see counter = 1 (isolated)");
}

// ── Native Function Calls ────────────────────────────────────────

/// Helper: compile with module types and native registry.
fn compile_with_natives(
    source: &str,
    module_types: &ModuleTypes,
    native_registry: &NativeRegistry,
) -> zehd_rune::module::CompiledModule {
    let parse_result = zehd_codex::parse(source);
    if !parse_result.is_ok() {
        let msgs: Vec<String> =
            parse_result.errors.iter().map(|e| format!("  {e}")).collect();
        panic!("parse errors:\n{}", msgs.join("\n"));
    }
    let check_result =
        zehd_sigil::check(&parse_result.program, source, module_types);
    if check_result.has_errors() {
        let msgs: Vec<String> =
            check_result.errors.iter().map(|e| format!("  {e}")).collect();
        panic!("type errors:\n{}", msgs.join("\n"));
    }
    let compile_result =
        zehd_rune::compile(&parse_result.program, check_result, native_registry);
    if compile_result.has_errors() {
        let msgs: Vec<String> =
            compile_result.errors.iter().map(|e| format!("  {e}")).collect();
        panic!("compile errors:\n{}", msgs.join("\n"));
    }
    compile_result.module
}

#[test]
fn native_function_call() {
    use std::collections::HashMap;
    use zehd_sigil::types::{FunctionType, Type};

    // Set up a test native function: add_one(n: int) -> int
    let mut module_types = ModuleTypes::new();
    module_types.insert(
        "std::test".to_string(),
        HashMap::from([(
            "add_one".to_string(),
            Type::Function(FunctionType {
                type_params: vec![],
                type_param_vars: vec![],
                params: vec![Type::Int],
                return_type: Box::new(Type::Int),
            }),
        )]),
    );

    let mut native_registry = NativeRegistry::new();
    native_registry.register("std::test", "add_one", 0);

    let native_fns: Vec<zehd_ward::NativeFn> = vec![|args| {
        match args.first() {
            Some(Value::Int(n)) => Ok(Value::Int(n + 1)),
            _ => Err(zehd_ward::error::RuntimeError::err(
                zehd_ward::error::RuntimeErrorCode::R120,
                "expected int",
            )
            .build()),
        }
    }];

    let module = compile_with_natives(
        r#"
        import { add_one } from std::test;
        fn test_it(x: int): int {
            add_one(x)
        }
        "#,
        &module_types,
        &native_registry,
    );

    let context = zehd_ward::Context {
        module,
        native_fns: Arc::new(native_fns),
    };
    let mut vm = zehd_ward::vm::StackVm::new();
    let result = vm
        .call_function(0, vec![Value::Int(41)], &context)
        .unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn native_env_returns_none_for_missing() {
    use std::collections::HashMap;
    use zehd_sigil::types::{FunctionType, Type};

    let mut module_types = ModuleTypes::new();
    module_types.insert(
        "std".to_string(),
        HashMap::from([(
            "env".to_string(),
            Type::Function(FunctionType {
                type_params: vec![],
                type_param_vars: vec![],
                params: vec![Type::String],
                return_type: Box::new(Type::Option(Box::new(Type::String))),
            }),
        )]),
    );

    let mut native_registry = NativeRegistry::new();
    native_registry.register("std", "env", 0);

    let native_fns: Vec<zehd_ward::NativeFn> = vec![|args| match args.first() {
        Some(Value::String(key)) => match std::env::var(key) {
            Ok(val) => Ok(Value::String(val)),
            Err(_) => Ok(Value::None),
        },
        _ => Ok(Value::None),
    }];

    let module = compile_with_natives(
        r#"
        import { env } from std;
        fn get_env(): Option<string> {
            env("ZEHD_TEST_NONEXISTENT_VAR_12345")
        }
        "#,
        &module_types,
        &native_registry,
    );

    let context = zehd_ward::Context {
        module,
        native_fns: Arc::new(native_fns),
    };
    let mut vm = zehd_ward::vm::StackVm::new();
    let result = vm.call_function(0, vec![], &context).unwrap();
    assert_eq!(result, Value::None);
}

#[test]
fn import_unknown_module_errors() {
    let source = r#"import { foo } from std::fake;"#;
    let parse_result = zehd_codex::parse(source);
    assert!(parse_result.is_ok());
    let check_result =
        zehd_sigil::check(&parse_result.program, source, &Default::default());
    assert!(check_result.has_errors());
    assert!(check_result
        .errors
        .iter()
        .any(|e| e.code.to_string() == "T103"));
}

#[test]
fn import_unknown_export_errors() {
    use std::collections::HashMap;
    use zehd_sigil::types::Type;

    let mut module_types = ModuleTypes::new();
    module_types.insert("std".to_string(), HashMap::from([
        ("env".to_string(), Type::Unit),
    ]));

    let source = r#"import { nonexistent } from std;"#;
    let parse_result = zehd_codex::parse(source);
    assert!(parse_result.is_ok());
    let check_result =
        zehd_sigil::check(&parse_result.program, source, &module_types);
    assert!(check_result.has_errors());
    assert!(check_result
        .errors
        .iter()
        .any(|e| e.code.to_string() == "T103"));
}
