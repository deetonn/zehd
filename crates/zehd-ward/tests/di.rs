mod helpers;

use std::collections::HashMap;

use zehd_rune::value::Value;
use zehd_ward::vm::StackVm;
use zehd_ward::error::RuntimeErrorCode;

// ── Provide / Inject ────────────────────────────────────────────

#[test]
fn provide_inject_roundtrip() {
    // provide<AppName>("my-app") then inject<AppName>() should return "my-app"
    let source = r#"
        import { provide, inject } from std;
        provide<AppName>("my-app");
        const name = inject<AppName>();
        fn get_name(): string { name }
    "#;
    let module = helpers::compile_module_with_std(source);
    let context = helpers::context_with_std(module);
    let mut vm = StackVm::new();

    // Run server_init (provide + inject + const)
    if let Some(chunk) = &context.module.server_init {
        use zehd_ward::VmBackend;
        vm.execute(chunk, &context).unwrap();
    }

    // Call the function
    let result = vm.call_function(0, vec![], &context).unwrap();
    assert_eq!(result, Value::String("my-app".to_string()));
}

#[test]
fn inject_without_provide_fails() {
    let source = r#"
        import { inject } from std;
        const name = inject<MissingType>();
    "#;
    let module = helpers::compile_module_with_std(source);
    let context = helpers::context_with_std(module);
    let mut vm = StackVm::new();

    if let Some(chunk) = &context.module.server_init {
        use zehd_ward::VmBackend;
        let err = vm.execute(chunk, &context).unwrap_err();
        assert_eq!(err.code, RuntimeErrorCode::R170);
        assert!(err.message.contains("MissingType"));
    }
}

#[test]
fn provide_same_type_twice_fails() {
    let source = r#"
        import { provide } from std;
        provide<AppName>("first");
        provide<AppName>("second");
    "#;
    let module = helpers::compile_module_with_std(source);
    let context = helpers::context_with_std(module);
    let mut vm = StackVm::new();

    if let Some(chunk) = &context.module.server_init {
        use zehd_ward::VmBackend;
        let err = vm.execute(chunk, &context).unwrap_err();
        assert_eq!(err.code, RuntimeErrorCode::R171);
        assert!(err.message.contains("AppName"));
    }
}

#[test]
fn provide_different_types_ok() {
    let source = r#"
        import { provide, inject } from std;
        provide<AppName>("my-app");
        provide<Environment>("production");
        const name = inject<AppName>();
        const env = inject<Environment>();
        fn get_name(): string { name }
    "#;
    let module = helpers::compile_module_with_std(source);
    let context = helpers::context_with_std(module);
    let mut vm = StackVm::new();

    if let Some(chunk) = &context.module.server_init {
        use zehd_ward::VmBackend;
        vm.execute(chunk, &context).unwrap();
    }

    let result = vm.call_function(0, vec![], &context).unwrap();
    assert_eq!(result, Value::String("my-app".to_string()));
}

#[test]
fn di_registry_preloaded() {
    // Test that with_globals_and_di pre-loads DI
    let source = r#"
        import { inject } from std;
        const name = inject<AppName>();
        fn get_name(): string { name }
    "#;
    let module = helpers::compile_module_with_std(source);
    let context = helpers::context_with_std(module);

    let mut di = HashMap::new();
    di.insert("AppName".to_string(), Value::String("pre-loaded".to_string()));

    let mut vm = StackVm::with_globals_and_di(vec![], di);

    if let Some(chunk) = &context.module.server_init {
        use zehd_ward::VmBackend;
        vm.execute(chunk, &context).unwrap();
    }

    let result = vm.call_function(0, vec![], &context).unwrap();
    assert_eq!(result, Value::String("pre-loaded".to_string()));
}
