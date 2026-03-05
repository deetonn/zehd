mod helpers;

use zehd_rune::value::Value;

// ── String Methods ──────────────────────────────────────────────

#[test]
fn string_length() {
    let result = helpers::call_fn0(
        r#"fn f(): int { "hello".length }"#,
        vec![],
    );
    assert_eq!(result, Value::Int(5));
}

#[test]
fn string_length_call() {
    let result = helpers::call_fn0(
        r#"fn f(): int { "hello".length() }"#,
        vec![],
    );
    assert_eq!(result, Value::Int(5));
}

#[test]
fn string_contains_true() {
    let result = helpers::call_fn0(
        r#"fn f(): bool { "hello world".contains("world") }"#,
        vec![],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn string_contains_false() {
    let result = helpers::call_fn0(
        r#"fn f(): bool { "hello".contains("xyz") }"#,
        vec![],
    );
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn string_starts_with() {
    let result = helpers::call_fn0(
        r#"fn f(): bool { "hello".starts_with("hel") }"#,
        vec![],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn string_ends_with() {
    let result = helpers::call_fn0(
        r#"fn f(): bool { "hello".ends_with("llo") }"#,
        vec![],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn string_trim() {
    let result = helpers::call_fn0(
        r#"fn f(): string { "  hello  ".trim() }"#,
        vec![],
    );
    assert_eq!(result, Value::String("hello".to_string()));
}

#[test]
fn string_to_upper() {
    let result = helpers::call_fn0(
        r#"fn f(): string { "hello".to_upper() }"#,
        vec![],
    );
    assert_eq!(result, Value::String("HELLO".to_string()));
}

#[test]
fn string_to_lower() {
    let result = helpers::call_fn0(
        r#"fn f(): string { "HELLO".to_lower() }"#,
        vec![],
    );
    assert_eq!(result, Value::String("hello".to_string()));
}

#[test]
fn string_split() {
    let result = helpers::call_fn0(
        r#"fn f(): List<string> { "a,b,c".split(",") }"#,
        vec![],
    );
    assert_eq!(
        result,
        Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ])
    );
}

#[test]
fn string_replace() {
    let result = helpers::call_fn0(
        r#"fn f(): string { "hello world".replace("world", "rust") }"#,
        vec![],
    );
    assert_eq!(result, Value::String("hello rust".to_string()));
}

#[test]
fn string_substring() {
    let result = helpers::call_fn0(
        r#"fn f(): Result<string, string> { "hello".substring(1, 4) }"#,
        vec![],
    );
    assert_eq!(
        result,
        Value::Enum {
            type_idx: 0xFFFF,
            variant_idx: 0,
            payload: Some(Box::new(Value::String("ell".to_string()))),
        }
    );
}

#[test]
fn string_index_of_found() {
    let result = helpers::call_fn0(
        r#"fn f(): Option<int> { "hello".index_of("ll") }"#,
        vec![],
    );
    assert_eq!(
        result,
        Value::Enum {
            type_idx: 0xFFFE,
            variant_idx: 0,
            payload: Some(Box::new(Value::Int(2))),
        }
    );
}

#[test]
fn string_index_of_not_found() {
    let result = helpers::call_fn0(
        r#"fn f(): Option<int> { "hello".index_of("xyz") }"#,
        vec![],
    );
    assert_eq!(
        result,
        Value::Enum {
            type_idx: 0xFFFE,
            variant_idx: 1,
            payload: None,
        }
    );
}

#[test]
fn string_char_at_valid() {
    let result = helpers::call_fn0(
        r#"fn f(): Option<string> { "hello".char_at(1) }"#,
        vec![],
    );
    assert_eq!(
        result,
        Value::Enum {
            type_idx: 0xFFFE,
            variant_idx: 0,
            payload: Some(Box::new(Value::String("e".to_string()))),
        }
    );
}

#[test]
fn string_char_at_out_of_bounds() {
    let result = helpers::call_fn0(
        r#"fn f(): Option<string> { "hello".char_at(10) }"#,
        vec![],
    );
    assert_eq!(
        result,
        Value::Enum {
            type_idx: 0xFFFE,
            variant_idx: 1,
            payload: None,
        }
    );
}

// ── List Methods ────────────────────────────────────────────────

#[test]
fn list_length() {
    let result = helpers::call_fn0(
        r#"fn f(): int { [1, 2, 3].length }"#,
        vec![],
    );
    assert_eq!(result, Value::Int(3));
}

#[test]
fn list_length_call() {
    let result = helpers::call_fn0(
        r#"fn f(): int { [1, 2, 3].length() }"#,
        vec![],
    );
    assert_eq!(result, Value::Int(3));
}

#[test]
fn list_push() {
    let result = helpers::call_fn0(
        r#"fn f(): List<int> { [1, 2].push(3) }"#,
        vec![],
    );
    assert_eq!(
        result,
        Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
    );
}

#[test]
fn list_contains_true() {
    let result = helpers::call_fn0(
        r#"fn f(): bool { [1, 2, 3].contains(2) }"#,
        vec![],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn list_contains_false() {
    let result = helpers::call_fn0(
        r#"fn f(): bool { [1, 2, 3].contains(5) }"#,
        vec![],
    );
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn list_join() {
    let result = helpers::call_fn0(
        r#"fn f(): string { ["a", "b", "c"].join(", ") }"#,
        vec![],
    );
    assert_eq!(result, Value::String("a, b, c".to_string()));
}

#[test]
fn list_reverse() {
    let result = helpers::call_fn0(
        r#"fn f(): List<int> { [1, 2, 3].reverse() }"#,
        vec![],
    );
    assert_eq!(
        result,
        Value::List(vec![Value::Int(3), Value::Int(2), Value::Int(1)])
    );
}

#[test]
fn list_slice() {
    let result = helpers::call_fn0(
        r#"fn f(): Result<List<int>, string> { [1, 2, 3, 4, 5].slice(1, 4) }"#,
        vec![],
    );
    assert_eq!(
        result,
        Value::Enum {
            type_idx: 0xFFFF,
            variant_idx: 0,
            payload: Some(Box::new(Value::List(vec![Value::Int(2), Value::Int(3), Value::Int(4)]))),
        }
    );
}

#[test]
fn list_slice_out_of_bounds() {
    let result = helpers::call_fn0(
        r#"fn f(): Result<List<int>, string> { [1, 2, 3].slice(0, 10) }"#,
        vec![],
    );
    match result {
        Value::Enum { type_idx: 0xFFFF, variant_idx: 1, payload: Some(msg) } => {
            let Value::String(s) = *msg else { panic!("expected string error message") };
            assert!(s.contains("out of bounds"), "expected out of bounds message, got: {s}");
        }
        other => panic!("expected Err result, got: {other:?}"),
    }
}

// ── Int Methods ─────────────────────────────────────────────────

#[test]
fn int_to_string() {
    let result = helpers::call_fn0(
        r#"fn f(): string { 42.to_string() }"#,
        vec![],
    );
    assert_eq!(result, Value::String("42".to_string()));
}

#[test]
fn int_abs_positive() {
    let result = helpers::call_fn0(
        r#"fn f(): int { 5.abs() }"#,
        vec![],
    );
    assert_eq!(result, Value::Int(5));
}

#[test]
fn int_abs_negative() {
    let result = helpers::call_fn0(
        r#"
        fn f(): int {
            let x = -5;
            x.abs()
        }
        "#,
        vec![],
    );
    assert_eq!(result, Value::Int(5));
}

#[test]
fn int_to_float() {
    let result = helpers::call_fn0(
        r#"fn f(): float { 42.to_float() }"#,
        vec![],
    );
    assert_eq!(result, Value::Float(42.0));
}

// ── Float Methods ───────────────────────────────────────────────

#[test]
fn float_to_string() {
    let result = helpers::call_fn0(
        r#"fn f(): string { 3.14.to_string() }"#,
        vec![],
    );
    assert_eq!(result, Value::String("3.14".to_string()));
}

#[test]
fn float_abs() {
    let result = helpers::call_fn0(
        r#"
        fn f(): float {
            let x = -3.14;
            x.abs()
        }
        "#,
        vec![],
    );
    assert_eq!(result, Value::Float(3.14));
}

#[test]
fn float_floor() {
    let result = helpers::call_fn0(
        r#"fn f(): int { 3.7.floor() }"#,
        vec![],
    );
    assert_eq!(result, Value::Int(3));
}

#[test]
fn float_ceil() {
    let result = helpers::call_fn0(
        r#"fn f(): int { 3.2.ceil() }"#,
        vec![],
    );
    assert_eq!(result, Value::Int(4));
}

#[test]
fn float_round() {
    let result = helpers::call_fn0(
        r#"fn f(): int { 3.5.round() }"#,
        vec![],
    );
    assert_eq!(result, Value::Int(4));
}

// ── Chaining ────────────────────────────────────────────────────

#[test]
fn method_chaining() {
    let result = helpers::call_fn0(
        r#"fn f(): string { "  HELLO  ".trim().to_lower() }"#,
        vec![],
    );
    assert_eq!(result, Value::String("hello".to_string()));
}

#[test]
fn split_then_length() {
    let result = helpers::call_fn0(
        r#"fn f(): int { "a,b,c".split(",").length }"#,
        vec![],
    );
    assert_eq!(result, Value::Int(3));
}

// ── Method with let binding ─────────────────────────────────────

#[test]
fn method_on_variable() {
    let result = helpers::call_fn0(
        r#"
        fn f(): int {
            let s = "hello world";
            s.length
        }
        "#,
        vec![],
    );
    assert_eq!(result, Value::Int(11));
}

#[test]
fn method_on_nested_field_access() {
    let result = helpers::call_fn0(
        r#"
        fn f(): int {
            let obj = { name: "hello" };
            obj.name.length
        }
        "#,
        vec![],
    );
    assert_eq!(result, Value::Int(5));
}

#[test]
fn method_call_on_nested_field_access() {
    let result = helpers::call_fn0(
        r#"
        fn f(): bool {
            let obj = { name: "hello world" };
            obj.name.contains("world")
        }
        "#,
        vec![],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn triple_nested_field_method() {
    let result = helpers::call_fn0(
        r#"
        fn f(): int {
            let obj = { inner: { name: "hello" } };
            obj.inner.name.length
        }
        "#,
        vec![],
    );
    assert_eq!(result, Value::Int(5));
}

#[test]
fn method_on_list_field() {
    let result = helpers::call_fn0(
        r#"
        fn f(): int {
            let obj = { items: [1, 2, 3] };
            obj.items.length
        }
        "#,
        vec![],
    );
    assert_eq!(result, Value::Int(3));
}

#[test]
fn method_call_on_list_field() {
    let result = helpers::call_fn0(
        r#"
        fn f(): bool {
            let obj = { items: [1, 2, 3] };
            obj.items.contains(2)
        }
        "#,
        vec![],
    );
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn substring_out_of_bounds() {
    // substring with out-of-range indices returns Err
    let result = helpers::call_fn0(
        r#"fn f(): Result<string, string> { "H".substring(4, 5) }"#,
        vec![],
    );
    match result {
        Value::Enum { type_idx: 0xFFFF, variant_idx: 1, payload: Some(msg) } => {
            let Value::String(s) = *msg else { panic!("expected string error message") };
            assert!(s.contains("out of bounds"), "expected out of bounds message, got: {s}");
        }
        other => panic!("expected Err result, got: {other:?}"),
    }
}

#[test]
fn method_result_in_let() {
    let result = helpers::call_fn0(
        r#"
        fn f(): List<string> {
            let csv = "a,b,c";
            let parts = csv.split(",");
            parts
        }
        "#,
        vec![],
    );
    assert_eq!(
        result,
        Value::List(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ])
    );
}

// ── Handler context (self.request.field.method) ────────────────

#[test]
fn method_on_self_request_query() {
    // Use compile_module_with_std + context_with_std for handler tests.
    let module = helpers::compile_module_with_std(
        r#"
        import { Request } from std::http;
        get {
            self.request.query.length
        }
        "#,
    );
    let context = helpers::context_with_std(module);
    let mut vm = zehd_ward::vm::StackVm::new();

    // Run server_init if present
    if let Some(chunk) = &context.module.server_init {
        use zehd_ward::VmBackend;
        vm.execute(chunk, &context).unwrap();
    }

    let self_value = Value::Object(vec![
        ("request".to_string(), Value::Object(vec![
            ("method".to_string(), Value::String("GET".to_string())),
            ("path".to_string(), Value::String("/test".to_string())),
            ("headers".to_string(), Value::Object(vec![])),
            ("body".to_string(), Value::String(String::new())),
            ("query".to_string(), Value::String("foo=bar".to_string())),
        ])),
        ("response".to_string(), Value::Object(vec![
            ("status".to_string(), Value::Int(200)),
        ])),
        ("params".to_string(), Value::Object(vec![])),
    ]);
    let result = vm.execute_handler(0, &context, self_value).unwrap();
    assert_eq!(result, Value::Int(7)); // "foo=bar".length == 7
}

#[test]
fn method_call_on_self_request_query() {
    let module = helpers::compile_module_with_std(
        r#"
        import { Request } from std::http;
        get {
            self.request.query.contains("foo")
        }
        "#,
    );
    let context = helpers::context_with_std(module);
    let mut vm = zehd_ward::vm::StackVm::new();
    if let Some(chunk) = &context.module.server_init {
        use zehd_ward::VmBackend;
        vm.execute(chunk, &context).unwrap();
    }
    let self_value = Value::Object(vec![
        ("request".to_string(), Value::Object(vec![
            ("method".to_string(), Value::String("GET".to_string())),
            ("path".to_string(), Value::String("/test".to_string())),
            ("headers".to_string(), Value::Object(vec![])),
            ("body".to_string(), Value::String(String::new())),
            ("query".to_string(), Value::String("foo=bar".to_string())),
        ])),
        ("response".to_string(), Value::Object(vec![
            ("status".to_string(), Value::Int(200)),
        ])),
        ("params".to_string(), Value::Object(vec![])),
    ]);
    let result = vm.execute_handler(0, &context, self_value).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn split_on_nested_field() {
    let module = helpers::compile_module_with_std(
        r#"
        import { Request } from std::http;
        get {
            self.request.query.split("&")
        }
        "#,
    );
    let context = helpers::context_with_std(module);
    let mut vm = zehd_ward::vm::StackVm::new();
    if let Some(chunk) = &context.module.server_init {
        use zehd_ward::VmBackend;
        vm.execute(chunk, &context).unwrap();
    }
    let self_value = Value::Object(vec![
        ("request".to_string(), Value::Object(vec![
            ("method".to_string(), Value::String("GET".to_string())),
            ("path".to_string(), Value::String("/test".to_string())),
            ("headers".to_string(), Value::Object(vec![])),
            ("body".to_string(), Value::String(String::new())),
            ("query".to_string(), Value::String("a=1&b=2".to_string())),
        ])),
        ("response".to_string(), Value::Object(vec![
            ("status".to_string(), Value::Int(200)),
        ])),
        ("params".to_string(), Value::Object(vec![])),
    ]);
    let result = vm.execute_handler(0, &context, self_value).unwrap();
    assert_eq!(
        result,
        Value::List(vec![
            Value::String("a=1".to_string()),
            Value::String("b=2".to_string()),
        ])
    );
}

#[test]
fn chain_method_on_nested_field() {
    let result = helpers::call_fn0(
        r#"
        fn f(): int {
            let obj = { data: "a,b,c" };
            obj.data.split(",").length
        }
        "#,
        vec![],
    );
    assert_eq!(result, Value::Int(3));
}

#[test]
fn chain_method_on_fn_result() {
    let result = helpers::call_fn(
        r#"
        fn get_name(): string {
            "hello world"
        }
        fn f(): bool {
            get_name().contains("world")
        }
        "#,
        1, // f is at index 1 (get_name is 0)
        vec![],
    );
    assert_eq!(result, Value::Bool(true));
}
