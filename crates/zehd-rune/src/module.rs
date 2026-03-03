use zehd_codex::ast::HttpMethod;

use crate::chunk::Chunk;

// ── Function Entry ─────────────────────────────────────────────

/// A compiled named function.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionEntry {
    /// Function name for debug/lookup.
    pub name: String,
    /// The compiled function body.
    pub chunk: Chunk,
}

// ── HTTP Handler ───────────────────────────────────────────────

/// A compiled HTTP method handler (get/post/put/patch/delete block).
#[derive(Debug, Clone, PartialEq)]
pub struct HttpHandler {
    /// The HTTP method this handler responds to.
    pub method: HttpMethod,
    /// The compiled handler body.
    pub chunk: Chunk,
}

// ── Compiled Module ────────────────────────────────────────────

/// The compilation output for a single `.z` file.
///
/// Contains all compiled chunks organized by their role:
/// - `server_init`: top-level VarDecls and ExprStmts (runs once at load)
/// - `handlers`: HTTP method blocks (run per-request)
/// - `init_block`: the `init { }` block (run at startup)
/// - `error_handler`: the `error(e) { }` block
/// - `functions`: named function definitions
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledModule {
    /// Top-level server-scoped code (VarDecl, ExprStmt outside handlers).
    pub server_init: Option<Chunk>,
    /// HTTP method handlers.
    pub handlers: Vec<HttpHandler>,
    /// The init { } block.
    pub init_block: Option<Chunk>,
    /// The error(e) { } handler.
    pub error_handler: Option<Chunk>,
    /// Named function definitions.
    pub functions: Vec<FunctionEntry>,
}
