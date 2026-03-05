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

/// Identifier for a user-defined module function.
pub type ModuleFnId = u16;

/// Maps `(module_path, export_name)` to a `ModuleFnId`.
///
/// Used by the compiler to emit `CallModule` instructions for calls to
/// functions defined in user modules (e.g., `lib/auth.z`).
#[derive(Debug, Clone)]
pub struct ModuleFnRegistry {
    map: HashMap<(String, String), ModuleFnId>,
}

impl ModuleFnRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Register a module function under `module_path::name` with the given id.
    pub fn register(&mut self, module: impl Into<String>, name: impl Into<String>, id: ModuleFnId) {
        self.map.insert((module.into(), name.into()), id);
    }

    /// Look up a module function by module path and export name.
    pub fn lookup(&self, module: &str, name: &str) -> Option<ModuleFnId> {
        self.map.get(&(module.to_string(), name.to_string())).copied()
    }
}

impl Default for ModuleFnRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_fn_registry_register_and_lookup() {
        let mut reg = ModuleFnRegistry::new();
        reg.register("lib::math", "add", 0);
        reg.register("lib::math", "sub", 1);
        reg.register("lib::auth", "hash", 2);

        assert_eq!(reg.lookup("lib::math", "add"), Some(0));
        assert_eq!(reg.lookup("lib::math", "sub"), Some(1));
        assert_eq!(reg.lookup("lib::auth", "hash"), Some(2));
        assert_eq!(reg.lookup("lib::math", "missing"), None);
        assert_eq!(reg.lookup("lib::missing", "add"), None);
    }

    #[test]
    fn module_fn_registry_default_is_empty() {
        let reg = ModuleFnRegistry::default();
        assert_eq!(reg.lookup("lib::math", "add"), None);
    }
}
