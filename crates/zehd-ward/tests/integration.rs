mod helpers;

use zehd_rune::value::Value;

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
    let context = zehd_ward::Context { module };
    let mut vm = zehd_ward::vm::StackVm::new();

    // Run server_init to set up globals
    if let Some(chunk) = &context.module.server_init {
        use zehd_ward::VmBackend;
        vm.execute(chunk, &context).unwrap();
    }

    // First handler call
    let r1 = vm.execute_handler(0, &context).unwrap();
    assert_eq!(r1, Value::String("hi".into()));

    // Second handler call — globals should still be there
    let r2 = vm.execute_handler(1, &context).unwrap();
    assert_eq!(r2, Value::String("hi".into()));
}
