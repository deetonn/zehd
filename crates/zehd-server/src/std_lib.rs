use std::collections::HashMap;

use zehd_rune::registry::NativeRegistry;
use zehd_rune::value::Value;
use zehd_sigil::types::{FunctionType, Type};
use zehd_sigil::ModuleTypes;
use zehd_ward::NativeFn;

/// Build the standard library: types, native registry, and implementations.
///
/// This is the single source of truth for all native functions. Each function
/// is registered once with its type signature, ID, and Rust implementation.
pub fn build_std() -> (ModuleTypes, NativeRegistry, Vec<NativeFn>) {
    let mut module_types = ModuleTypes::new();
    let mut registry = NativeRegistry::new();
    let mut native_fns: Vec<NativeFn> = Vec::new();

    // Helper: register a native function in all three structures.
    let mut next_id: u16 = 0;
    let mut register = |module: &str,
                        name: &str,
                        ty: Type,
                        implementation: NativeFn,
                        module_types: &mut ModuleTypes,
                        registry: &mut NativeRegistry,
                        native_fns: &mut Vec<NativeFn>| {
        let id = next_id;
        next_id += 1;

        module_types
            .entry(module.to_string())
            .or_insert_with(HashMap::new)
            .insert(name.to_string(), ty);

        registry.register(module, name, id);
        native_fns.push(implementation);
    };

    // ── std::env ────────────────────────────────────────────────
    // env(key: string) -> Option<string>
    register(
        "std",
        "env",
        Type::Function(FunctionType {
            params: vec![Type::String],
            return_type: Box::new(Type::Option(Box::new(Type::String))),
        }),
        native_env,
        &mut module_types,
        &mut registry,
        &mut native_fns,
    );

    // ── std::log ────────────────────────────────────────────────
    // log::info(msg: string) -> ()
    register(
        "std::log",
        "info",
        Type::Function(FunctionType {
            params: vec![Type::String],
            return_type: Box::new(Type::Unit),
        }),
        native_log_info,
        &mut module_types,
        &mut registry,
        &mut native_fns,
    );

    // log::warn(msg: string) -> ()
    register(
        "std::log",
        "warn",
        Type::Function(FunctionType {
            params: vec![Type::String],
            return_type: Box::new(Type::Unit),
        }),
        native_log_warn,
        &mut module_types,
        &mut registry,
        &mut native_fns,
    );

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
