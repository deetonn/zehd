use std::collections::HashMap;

/// Identifier for a native (Rust-implemented) function.
pub type NativeFnId = u16;

/// Maps `(module_path, export_name)` to a `NativeFnId`.
///
/// The compiler uses this to emit `CallNative` instructions instead of
/// the normal `GetGlobal → Call` path when a function call targets an
/// imported native.
#[derive(Debug, Clone)]
pub struct NativeRegistry {
    map: HashMap<(String, String), NativeFnId>,
}

impl NativeRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Register a native function under `module::name` with the given id.
    pub fn register(&mut self, module: impl Into<String>, name: impl Into<String>, id: NativeFnId) {
        self.map.insert((module.into(), name.into()), id);
    }

    /// Look up a native function by module path and export name.
    pub fn lookup(&self, module: &str, name: &str) -> Option<NativeFnId> {
        self.map.get(&(module.to_string(), name.to_string())).copied()
    }
}

impl Default for NativeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
