use zehd_rune::registry::NativeRegistry;
use zehd_rune::value::Value;
use zehd_sigil::ModuleTypes;
use zehd_ward::NativeFn;

/// Build the standard library: types, native registry, and implementations.
///
/// Type signatures come from `zehd_sigil::std_module_types()` (shared with
/// the LSP). This function adds the native registry IDs and Rust implementations.
pub fn build_std() -> (ModuleTypes, NativeRegistry, Vec<NativeFn>) {
    let module_types = zehd_sigil::std_module_types();

    let mut registry = NativeRegistry::new();
    let mut native_fns: Vec<NativeFn> = Vec::new();

    // Register each native function with a sequential ID.
    // ORDER MATTERS — IDs must match the index in native_fns.
    let natives: &[(&str, &str, NativeFn)] = &[
        ("std", "env", native_env),
        ("std::log", "info", native_log_info),
        ("std::log", "warn", native_log_warn),
    ];

    for (i, (module, name, implementation)) in natives.iter().enumerate() {
        registry.register(*module, *name, i as u16);
        native_fns.push(*implementation);
    }

    (module_types, registry, native_fns)
}

// ── Native Implementations ──────────────────────────────────────

fn native_env(args: &[Value]) -> Result<Value, zehd_ward::error::RuntimeError> {
    let key = match args.first() {
        Some(Value::String(s)) => s,
        _ => {
            return Err(zehd_ward::error::RuntimeError::err(
                zehd_ward::error::RuntimeErrorCode::R120,
                "env() expects a string argument",
            )
            .build());
        }
    };
    match std::env::var(key) {
        Ok(val) => Ok(Value::String(val)),
        Err(_) => Ok(Value::None),
    }
}

fn native_log_info(args: &[Value]) -> Result<Value, zehd_ward::error::RuntimeError> {
    let msg = match args.first() {
        Some(Value::String(s)) => s.as_str(),
        _ => {
            return Err(zehd_ward::error::RuntimeError::err(
                zehd_ward::error::RuntimeErrorCode::R120,
                "info() expects a string argument",
            )
            .build());
        }
    };
    println!("[INFO] {msg}");
    Ok(Value::Unit)
}

fn native_log_warn(args: &[Value]) -> Result<Value, zehd_ward::error::RuntimeError> {
    let msg = match args.first() {
        Some(Value::String(s)) => s.as_str(),
        _ => {
            return Err(zehd_ward::error::RuntimeError::err(
                zehd_ward::error::RuntimeErrorCode::R120,
                "warn() expects a string argument",
            )
            .build());
        }
    };
    eprintln!("[WARN] {msg}");
    Ok(Value::Unit)
}
