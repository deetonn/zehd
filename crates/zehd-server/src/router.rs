use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zehd_codex::ast::HttpMethod as ZehdMethod;
use zehd_ward::vm::StackVm;
use zehd_ward::{Context, NativeFn, VmBackend};

use crate::compile::CompiledRoute;
use crate::error::StartupError;

// ── HTTP Method ─────────────────────────────────────────────────

/// HTTP methods we dispatch on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
        }
    }

    /// Convert from axum's HTTP method.
    pub fn from_axum(method: &axum::http::Method) -> Option<Self> {
        match *method {
            axum::http::Method::GET => Some(Method::Get),
            axum::http::Method::POST => Some(Method::Post),
            axum::http::Method::PUT => Some(Method::Put),
            axum::http::Method::PATCH => Some(Method::Patch),
            axum::http::Method::DELETE => Some(Method::Delete),
            _ => None,
        }
    }
}

fn from_zehd_method(m: &ZehdMethod) -> Method {
    match m {
        ZehdMethod::Get => Method::Get,
        ZehdMethod::Post => Method::Post,
        ZehdMethod::Put => Method::Put,
        ZehdMethod::Patch => Method::Patch,
        ZehdMethod::Delete => Method::Delete,
    }
}

// ── Route Entry ─────────────────────────────────────────────────

/// A single route with its method handlers, VM, and context.
pub struct RouteEntry {
    /// Maps HTTP method to handler index in `context.module.handlers`.
    pub method_map: HashMap<Method, usize>,
    /// VM instance for this route (owns globals from server_init).
    pub vm: Mutex<StackVm>,
    /// Immutable compiled module.
    pub context: Context,
    /// Sorted list of allowed methods (for 405 Allow header).
    pub allowed_methods: Vec<Method>,
}

// ── Route Table ─────────────────────────────────────────────────

/// URL path → RouteEntry lookup table.
pub struct RouteTable {
    pub routes: HashMap<String, Arc<RouteEntry>>,
}

impl RouteTable {
    /// Build a route table from compiled routes.
    ///
    /// For each route:
    /// 1. Build the method_map from compiled handlers
    /// 2. Create a fresh VM
    /// 3. Run server_init to populate globals
    /// 4. Wrap in Arc<RouteEntry>
    pub fn build(
        compiled_routes: Vec<CompiledRoute>,
        native_fns: Arc<Vec<NativeFn>>,
    ) -> Result<Self, StartupError> {
        let mut routes = HashMap::new();

        for route in compiled_routes {
            let mut method_map = HashMap::new();
            for (idx, handler) in route.module.handlers.iter().enumerate() {
                let method = from_zehd_method(&handler.method);
                method_map.insert(method, idx);
            }

            let mut allowed: Vec<Method> = method_map.keys().copied().collect();
            allowed.sort_by_key(|m| match m {
                Method::Get => 0,
                Method::Post => 1,
                Method::Put => 2,
                Method::Patch => 3,
                Method::Delete => 4,
            });

            let context = Context {
                module: route.module,
                native_fns: Arc::clone(&native_fns),
            };

            let mut vm = StackVm::new();

            // Run server_init to populate globals (top-level let/const).
            if let Some(ref init_chunk) = context.module.server_init {
                vm.execute(init_chunk, &context).map_err(|e| {
                    StartupError::InitFailed {
                        url_path: route.url_path.clone(),
                        message: e.message,
                    }
                })?;
            }

            let entry = RouteEntry {
                method_map,
                vm: Mutex::new(vm),
                context,
                allowed_methods: allowed,
            };

            routes.insert(route.url_path, Arc::new(entry));
        }

        Ok(RouteTable { routes })
    }
}
