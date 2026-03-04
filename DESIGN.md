# zehd Language Design

> **zehd** (spoken as "Zed") — a programming language that *is* a web server.

## 1. Vision

Next.js but as its own language. The entire language is purpose-built for web servers. Routes, middleware, request/response, validation, database access — all first-class concepts, not library imports on top of a general-purpose language.

**Target audience:** Developers familiar with TypeScript/Next.js who want the same productivity with real performance and no framework overhead.

**File extension:** `.z`

---

## 2. Implementation

### 2.1 Language: Rust

Chosen for:
- Interpreter loop performance (no GC pauses)
- Mature web ecosystem baked into the runtime (tokio, hyper, rustls, serde)
- The web server *is* the runtime — Rust's async primitives become zehd's built-in capabilities
- Safety guarantees matter for a network-facing runtime
- PoC can scale to production without a rewrite

### 2.2 Execution Model: Bytecode VM → Cranelift JIT

**Architecture: Trait-based VM backend.** The compiler frontend (parser → AST → bytecode) is decoupled from execution via a `VmBackend` trait. This allows swapping the execution engine without touching the rest of the system.

```rust
// Conceptual — the trait boundary between compiler and execution
trait VmBackend {
    fn execute(&mut self, chunk: &Chunk, context: &mut Context) -> Result<Value, RuntimeError>;
    // Start narrow, expand as needed
}
```

**Phase 1 — Stack-based Bytecode VM:**
- Compile zehd source → bytecode → execute in a stack-based VM
- Stack-based chosen for simplicity — easier to implement, debug, and iterate on
- Implements `VmBackend` — the rest of the system is backend-agnostic
- Gets the language working quickly
- **Per-request VMs:** Each request gets a fresh `StackVm` with cloned globals from server_init. No mutex, full parallelism. Server-scope `let` mutations are request-local (not shared between requests). Shared mutable state is for `provide<T>/inject<T>` (future).

**Phase 2 — Cranelift JIT (future):**
- Second implementation of `VmBackend` — plugs into the same system
- Cranelift is Rust-native, designed for JIT (unlike LLVM which is AOT-first)
- Faster compile times than LLVM — critical for a web server that needs fast startup
- Used by Firefox (SpiderMonkey) and Wasmtime — battle-tested

**Why not LLVM?** Heavy dependency, slow JIT startup, designed for AOT compilation. A web server needs to boot fast.

**Why the trait boundary?** Minimal cost (one trait definition), enforces clean separation that you'd want anyway. The risk of getting the trait surface wrong early is low — start narrow, widen as needed. This is idiomatic Rust.

### 2.3 Concurrency Model: Implicit Green Threads

The user never writes `async` or `await`. Under the hood, tokio runs the async runtime. In zehd, you write synchronous-looking code:

```
get {
  const user = db.users.find(id);      // yields to runtime automatically
  const posts = http.get(postsUrl);    // yields to runtime automatically
  return { user, posts };
}
```

- Every request gets a lightweight task
- Every I/O operation yields transparently to the runtime
- The runtime handles thousands of concurrent requests
- No async/await, no colored functions — the runtime manages it

This is a major differentiator from JS/TS, Rust, and Python where async infects everything.

### 2.4 Memory Model

**Dual-scope allocation:**

**Request scope — Arena-based:**
- Each request gets its own memory arena
- All allocations within a route handler live in the request arena
- Arena is freed in one shot when the response is sent
- Extremely fast allocation (bump pointer), zero per-object GC overhead
- Natural fit: requests have clear lifetimes

**Server scope — Reference-counted (for long-lived state):**
- State that outlives individual requests (DB pools, caches, config, background task data)
- Reference-counted for deterministic cleanup
- Created in `main.z` or `init.z`, shared with handlers via read-only references
- See section 4.7 for how server-scoped state is accessed

This gives the best of both worlds: request handling is GC-free and fast, while shared state is safe and deterministic.

### 2.5 Value Semantics

**Reference by default.** Assignment and function arguments pass references, not copies. This is what most developers expect.

Explicit copying is available via the standard library:

```
import { copy } from std;

const original = { name: "zehd", tags: ["fast", "web"] };
const cloned = copy(original);  // deep copy
```

---

## 3. Syntax

### 3.1 General Direction

TypeScript-esque with predictable performance characteristics:

- `let` / `const` bindings (same semantics as JavaScript)
- Object literals, destructuring
- Static type system — structural typing
- No implicit coercion, no prototype chain
- Semicolons required
- No `any` type — the type system is closed
- No `null` in the language — `Option<T>` for absence, `None` maps to JSON `null`
- Familiar to JS/TS developers but more predictable

### 3.2 Variable Bindings: let / const

Same as JavaScript:

- `const` — cannot be reassigned after declaration
- `let` — can be reassigned

```
const name: string = "zehd";     // immutable binding
let count: int = 0;              // mutable binding
count = count + 1;               // ok
name = "other";                  // compile error
```

### 3.3 Type System: Static + Structural

Statically typed with structural typing — types are compared by shape, not by name. If two types have the same fields, they are compatible.

The type system serves double duty — it defines structure AND validates data at boundaries.

```
type User {
  name: string;
  age: int;
  email: string;
}
```

Types are checked at compile time. At runtime boundaries (JSON parsing, form data, external input), the type system enforces validation automatically:

```
post {
  const user: User = self.request.json();
  // If we reach this line, 'user' is fully typed AND validated.
  // Invalid input returns 400 automatically.
}
```

**Type inference** is supported everywhere. The compiler infers types for local variables, function return types, and generic type arguments. Explicit annotations are always optional but allowed:

```
const name = "zehd";                    // inferred as string
const count = 42;                       // inferred as int
fn double(x: int) { return x * 2; }    // return type inferred as int
```

**Generics** are supported for user-defined types:

```
type ApiResponse<T> {
  data: T;
  status: int;
  timestamp: datetime;
}

const response: ApiResponse<User> = {
  data: user,
  status: 200,
  timestamp: now(),
};
```

### 3.4 Null and Option

**There is no `null` in the zehd language.** Absence is represented by `Option<T>`:

```
enum Option<T> {
  Some(T),
  None,
}
```

**JSON interop:** When serializing/deserializing JSON, `Option::None` maps directly to JSON `null`, and JSON `null` maps to `Option::None`. This is handled automatically by the runtime:

```
type UserProfile {
  name: string;              // required — JSON null here is a validation error
  bio: Option<string>;       // optional — JSON null maps to None
}

// Serializing back to JSON:
// { name: "Alice", bio: None } → { "name": "Alice", "bio": null }
```

Language APIs that interact with possibly-null JSON values always return `Option<T>`, forcing explicit handling.

### 3.5 Error Model: Result Types + Algebraic Enums

**Result types, not exceptions.** Code must handle errors explicitly. In the age of agents writing code, the type system must force correctness — not allow silent failures.

**Algebraic enums** are first-class:

```
enum Result<T, E> {
  Ok(T),
  Err(E),
}

// Custom enums
enum UserError {
  NotFound(string),
  Unauthorized,
  ValidationFailed(list<string>),
}
```

**Pattern matching** makes enums ergonomic:

```
get {
  match db.users.find(id) {
    Ok(user) => return user;
    Err(UserError.NotFound(msg)) => {
      self.response.status(404);
      return { error: msg };
    }
    Err(UserError.Unauthorized) => {
      self.response.status(403);
      return { error: "Forbidden" };
    }
  }
}
```

**The `?` operator** propagates errors — and in a route handler context, automatically maps to an appropriate HTTP error response:

```
get {
  const user = db.users.find(id)?;    // Err propagates as HTTP error
  const posts = db.posts.by_user(user.id)?;
  return { user, posts };
}
```

**Why Result types over exceptions?**
- Exceptions hide control flow — a function can throw without its signature saying so
- Result types are visible in the type signature — agents and humans see every error path
- Pattern matching forces exhaustive handling — you can't forget a case
- Better performance — no stack unwinding

### 3.6 If Expressions

If/else blocks are expressions — they can return values. This enables inline conditional assignment and early returns from within branches:

```
const label = if count > 10 { "many" } else { "few" };

const data = if input.valid() {
  input.get()
} else {
  return Err(ValidationError("Invalid input"));
};
```

If blocks used as statements (without capturing the return value) work as expected:

```
if !user {
  self.response.status(404);
  return { error: "not found" };
}
```

### 3.7 Functions

Functions are first-class citizens. Two syntaxes are supported:

**Named functions with `fn`:**
```
fn greet(name: string): string {
  return $"Hello, {name}!";
}

// Return type can be inferred
fn double(x: int) {
  return x * 2;
}
```

**Arrow functions (for callbacks and assignment):**
```
const greet = (name: string): string => $"Hello, {name}!";
const double = (x: int) => x * 2;

// Multi-line
const transform = (req) => {
  req.headers.set("Authorization", env("API_KEY"));
};
```

Both forms are interchangeable as values — functions are always first-class:

```
fn applyTwice(f: (int) => int, x: int): int {
  return f(f(x));
}

const result = applyTwice(double, 5);  // 20
```

Functions can be defined anywhere — at the top level of route files, in `lib/` modules, or inline.

### 3.8 String Interpolation

C#-style interpolated strings using `$"..."`:

```
const name = "world";
const greeting = $"Hello, {name}!";

const user = { name: "Alice", age: 30 };
const info = $"User {user.name} is {user.age} years old";

// Expressions are allowed inside braces
const label = $"Total: {items.len() * price}";
```

Regular strings use double quotes and do not interpolate: `"Hello, {name}"` is a literal string.

### 3.9 Attributes: Import-Based Reflection Metadata

Attributes are **metadata attached to types and fields**. They are not magic — they come from imported modules and are backed by reflection. This makes them hackable and extensible from day one.

**Syntax:** `#[module.attribute(...)]`

**Attributes are custom-first.** The standard library provides modules like `std::validation`, but the attribute system itself is general-purpose reflection. Anyone can create attribute modules.

```
import { validate } from std::validation;

type CreateUser {
  #[validate.min(1)]
  #[validate.max(100)]
  #[validate.fail(message="Name must be 1-100 characters")]
  name: string;

  #[validate.range(18, 150)]
  age: int;

  #[validate.email]
  #[validate.fail(message="Invalid email address")]
  email: string;

  #[validate.optional]
  #[validate.max(500)]
  bio: string;
}
```

**How it works under the hood:**
- Attributes are metadata baked into the object at initialization
- They are accessible via reflection at runtime
- The `self.request.json()` call reads the target type's attributes and applies them
- Modules like `std::validation` provide attribute definitions and the logic to interpret them
- Custom modules can define their own attributes — the system is open

**Other standard library attribute modules (future):**
```
import { json } from std::json;
import { db } from std::db;

type User {
  #[json.rename("user_id")]
  id: int;

  #[json.skip]
  passwordHash: string;

  #[db.column("created_at")]
  #[db.auto]
  createdAt: datetime;
}
```

**Key principle:** `#[module.attr()]` requires that `module` is imported. No magic globals. This ensures every attribute is traceable to its source and encourages a modular ecosystem from the start.

### 3.10 Time Literals

Built-in time value syntax for durations. Auto-converts to milliseconds at compile time. Essential for a web server language where timeouts, rate limits, and windows are everywhere.

```
const timeout = 30s;         // 30000 (milliseconds)
const window = 5m;           // 300000
const ttl = 1h;              // 3600000
const debounce = 500ms;      // 500

use(rateLimit("rollingWindow", 60s));
use(cache({ ttl: 5m }));
```

Inspired by the `ms` npm package, but as a language feature — no parsing overhead, no string ambiguity.

### 3.11 Trailing Commas

Trailing commas are allowed everywhere — object literals, enum variants, function arguments, import lists:

```
const user = {
  name: "zehd",
  version: "1.2.3",    // trailing comma ok
};

enum Status {
  Active,
  Inactive,             // trailing comma ok
}

import { use, rateLimit, } from std;  // trailing comma ok
```

### 3.12 Loops

**`for...in`** iterates over any iterable:

```
for item in items {
  log.info(item);
}

// With index (TBD — may need enumerate or similar)
for user in db.users.all() {
  log.info($"User: {user.name}");
}
```

**`while`** for condition-based loops:

```
let attempts = 0;
while attempts < 3 {
  // retry logic
  attempts = attempts + 1;
}
```

**`break` and `continue`** work as expected.

**Iterator protocol:** Libraries can create custom iterables. The exact trait/interface for iterators needs further design, but the syntax is `for item in <anything iterable>`. This will be modelled after a trait/interface pattern so any type can be iterable.

### 3.13 Module System

Modules use destructured imports only. The standard library lives under `std::`:

```
import { validate } from std::validation;
import { json } from std::json;
import { proxy, use, rateLimit } from std;
```

**Only destructured imports are supported.** No default imports, no wildcard imports. Every imported name is explicit and traceable.

Third-party modules TBD — package manager and registry are future concerns.

---

## 4. File-Based Routing

### 4.1 Directory Structure

The filesystem defines routes. The directory path IS the URL path. The routes root directory is configurable in `zehd.toml` (defaults to `routes/`):

```
routes/
  index.z                → GET /
  users/
    index.z              → /users
    [id].z               → /users/:id
    [id]/
      settings.z         → /users/:id/settings
  api/
    health.z             → /api/health
    external/
      [...path].z        → /api/external/*
```

**Conventions:**
- `[param]` — dynamic route segment, available as a typed variable in the handler
- `[...param]` — catch-all segment
- `index.z` — handles the directory's root path
- **Specific routes always win over dynamic routes** — `/users/settings.z` beats `/users/[id].z` for the path `/users/settings`

### 4.2 Route Files: HTTP Method Blocks

Inside a route file, HTTP methods are top-level blocks. `self` is the implicit context — always available, never imported. It carries everything: request, response, route params, cookies.

```
// routes/users/[id].z

type UserParams {
  id: string;
}

get {
  const params: UserParams = self.params.parse();
  const user = db.users.find(params.id);

  if !user {
    self.response.status(404);
    return { error: "User not found" };
  }

  return user;                       // auto-serialized to JSON
}

post {
  const params: UserParams = self.params.parse();
  const body: CreateUser = self.request.json();

  const user = db.users.create(body);
  self.response.status(201);
  return user;
}

delete {
  const params: UserParams = self.params.parse();
  db.users.delete(params.id);
  self.response.status(204);
}
```

### 4.3 The `self` Context

`self` is implicitly available in every HTTP method block. It is the single entry point to all request/response state:

| Property | Description |
|----------|-------------|
| `self.request` | Incoming request — headers, query, body, method, url |
| `self.response` | Response builder — status, headers, cookies, redirect, stream |
| `self.params` | Route parameters — opaque type, must be parsed into a typed struct |
| `self.request.headers` | Request headers |
| `self.request.query` | Query string parameters — opaque, parse to typed struct |
| `self.request.cookies` | Request cookies |
| `self.response.headers` | Response headers |
| `self.response.cookies` | Response cookie setter |

**Simple case:** just `return data` — auto-serialized, 200 OK.
**Complex case:** use `self.response` for full control over status, headers, cookies, redirects, streaming.

```
get {
  const res = self.response;
  res.headers.set("X-Server-Location", "lhr1:233");
  res.status(200);

  return {
    name: "zehd",
    version: "1.2.3",
  };
}
```

### 4.4 Route Parameters: Typed Validation

`self.params` is an **opaque type** — it holds the raw values from URL segments but must be explicitly parsed into a typed struct, just like `self.request.json()`. This keeps the validation model consistent: every external boundary goes through type validation.

```
// routes/users/[id].z

type UserParams {
  id: string;
}

get {
  const params: UserParams = self.params.parse();
  const user = db.users.find(params.id);

  if !user {
    self.response.status(404);
    return { error: "User not found" };
  }

  return user;
}
```

Params support the same attribute system as any other type:

```
import { validate } from std::validation;

type UserParams {
  #[validate.uuid]
  #[validate.fail(message="Invalid user ID format")]
  id: string;
}

get {
  const params: UserParams = self.params.parse();  // validates UUID format, returns 400 on failure
  const user = db.users.find(params.id);
  return user;
}
```

**Principle:** Params, JSON bodies, query strings — all external input goes through the same pipeline: opaque data → typed struct via parse/validation. No special cases.

### 4.5 Special Files

| File | Purpose | Scope |
|------|---------|-------|
| `init.z` | Runtime configuration — middleware, proxies, CORS, rate limiting | Applies to directory and all children |
| `error.z` | Error handling | Catches errors in directory and all children |
| `layout.z` | Wraps HTML responses | Applies to directory and all children |
| `middleware.z` | Request/response middleware | Runs before handlers in directory and all children |

### 4.6 init.z: Runtime Configuration

`init.z` runs at server startup for its route subtree. It configures behavior through runtime APIs — not keywords. This keeps the language small and the configuration explicit:

```
// routes/api/init.z

import { use, cors, rateLimit } from std;
import { auth } from std::auth;

init {
  use(cors({ origins: ["https://myapp.com"] }));
  use(rateLimit("rollingWindow", 60s));
  use(auth.bearer());
}
```

```
// routes/api/external/init.z

import { proxy } from std;

init {
  proxy("https://api.external.com", (req) => {
    req.headers.set("Authorization", env("API_KEY"));
  });
}
```

**Principle:** Behavior like proxies, CORS, and rate limiting are runtime configuration, not language keywords. `init.z` is where you configure what a route subtree *does*. This keeps the core language small and makes configuration discoverable (just read the init file).

### 4.7 Dependency Injection: provide / inject

Server-scoped state (DB pools, caches, config) flows into route handlers via a type-safe `provide`/`inject` system. **The type is the key** — no string lookups.

**Providing state** — in `main.z` (global) or `init.z` (subtree-scoped):

```
// main.z
import { provide } from std;
import { DbPool, createPool } from lib/db;

const pool = createPool({ url: env("DATABASE_URL") });
provide<DbPool>(pool);   // available to all routes
```

```
// routes/api/init.z
import { provide } from std;
import { Cache, createCache } from lib/cache;

init {
  provide<Cache>(createCache({ ttl: 5m }));   // available under /api only
}
```

**Injecting state** — at the file level in route files. Top-level code in route files runs at load time (server scope), not per-request. Injected dependencies are resolved once and available to all method blocks:

```
// routes/api/users/[id].z
import { inject } from std;
import { DbPool } from lib/db;
import { Cache } from lib/cache;

const db = inject<DbPool>();      // resolved at load time, once
const cache = inject<Cache>();    // provided by api/init.z

type UserParams {
  id: string;
}

get {
  const params: UserParams = self.params.parse();
  const user = db.users.find(params.id);  // just use it
  return user;
}

post {
  const body: CreateUser = self.request.json();
  const user = db.users.create(body);     // same db reference
  self.response.status(201);
  return user;
}
```

**Provider scoping** follows the route tree:
- `main.z` provides → available everywhere
- `routes/api/init.z` provides → available under `/api` and children
- Child providers can shadow parent providers of the same type

**Startup validation:** Before the server accepts traffic, the runtime walks all route files, finds every `inject<T>()` call, and verifies a matching `provide<T>()` exists in the ancestor chain. If anything is missing, the server **fails to start** with a clear error. No runtime surprises.

**Reducing boilerplate** — utility functions in `lib/` can wrap inject for convenience:

```
// lib/services.z
import { inject } from std;
import { DbPool } from lib/db;

fn getDb(): DbPool { return inject<DbPool>(); }
```

```
// routes/api/users/[id].z
import { getDb } from lib/services;

const db = getDb();

get {
  return db.users.find(params.id);
}
```

### 4.8 Route File Scoping Model

Route files have two scopes:

| Scope | Where | Lifetime | Memory |
|-------|-------|----------|--------|
| **Server scope** | Top-level code (imports, const, inject, type definitions, fn definitions) | Server lifetime | Reference-counted |
| **Request scope** | Inside method blocks (`get { }`, `post { }`, etc.) | Request lifetime | Arena-allocated |

```
// Server scope — runs once at load time
import { inject } from std;
const db = inject<DbPool>();
type UserParams { id: string; }

// Request scope — runs per request
get {
  const params: UserParams = self.params.parse();  // arena-allocated
  const user = db.users.find(params.id);           // db is server-scoped ref
  return user;
}
```

This distinction is how DI works (inject at server scope) and how arena-per-request works (allocations in method blocks are freed when the response completes).

### 4.9 error.z: Error Handler for `?` Operator

`error.z` defines how the `?` operator handles propagated errors within its route subtree. When `?` encounters an `Err`, it delegates to the nearest `error.z` in the ancestor chain. If no `error.z` exists, the default behavior is a 500 response.

```
// routes/api/error.z

error(err) {
  match err {
    UserError.NotFound(msg) => {
      self.response.status(404);
      return { error: msg };
    }
    UserError.Unauthorized => {
      self.response.status(403);
      return { error: "Forbidden" };
    }
    _ => {
      self.response.status(500);
      return { error: "Internal server error" };
    }
  }
}
```

**How it works:**
- `?` in any handler within the subtree delegates errors to this `error.z`
- The `err` parameter receives the `Err` value from the `Result`
- Pattern matching maps error types to HTTP responses
- The `_` wildcard catches unhandled error types
- If no `error.z` exists in the ancestor chain, the runtime returns `500 { "error": "Internal server error" }`
- Different subtrees can define different error handling — `/api/error.z` can return JSON while `/pages/error.z` could return HTML

This uses the existing special file convention and provides scoped, discoverable error handling without any new keywords.

---

## 5. Project Structure

### 5.1 Standard Layout

```
my-app/
  main.z               ← entry point — server config, global setup
  zehd.toml            ← project configuration
  routes/              ← file-based routing (configurable root)
    index.z
    ...
  lib/                 ← shared code — not parsed as routes (configurable)
    types.z
    helpers.z
    ...
  public/              ← static files (configurable)
    styles.css
    favicon.ico
    ...
```

### 5.2 main.z: Entry Point

`main.z` is the entry point of every zehd application. It runs once at server startup before any routes are loaded. Default-generated by tooling:

```
// main.z

import { on } from std;

// Global server configuration happens here
// The server starts automatically after main.z executes

on("shutdown", () => {
  // cleanup logic
});
```

### 5.3 zehd.toml: Project Configuration

```toml
[server]
port = 3000
host = "0.0.0.0"

[paths]
routes = "./routes"
lib = "./lib"
static = "./public"

[paths.ignore]
dirs = []
```

### 5.4 Environment Variables

Environment variables are accessed via a function from std:

```
import { env } from std;

const apiKey = env("API_KEY");           // returns Option<string>
const port = env("PORT");               // returns Option<string>
```

`.env` file support TBD — may be built-in or via tooling.

---

## 6. Standard Library

### 6.1 Core Types (built-in, no import needed)

- `string`, `int`, `float`, `bool`
- `list<T>`, `map<K, V>`
- `Result<T, E>`, `Option<T>` — with `?` operator and pattern matching
- Time literals: `500ms`, `30s`, `5m`, `1h`

### 6.2 Implicit Route Context (available in method blocks)

- `self` — the unified context object carrying request, response, and params

### 6.3 Standard Library Modules

**Phase 1 (ships with PoC):**
- `std` — core runtime APIs: `use()`, `proxy()`, `cors()`, `rateLimit()`, `env()`, `on()`, `copy()`, `provide<T>()`, `inject<T>()`
- `std::http` — HTTP types: `Request { method, path, headers, body, query }`, `Response { status }` — used by `self` in handlers
- `std::types` — built-in type exports (aliases for common types)
- `std::validation` — validation attributes
- `std::json` — JSON serialization attributes
- `std::log` — structured logging: `log.info()`, `log.error()`, `log.warn()`, `log.debug()`

**Phase 2 (future):**
- `std::db` — database access and mapping attributes
- `std::crypto` — hashing, encryption, tokens
- `std::auth` — authentication primitives

### 6.4 Event Hooks

Global lifecycle events are hookable via `on()` from std. Multiple handlers can be registered for the same event:

```
import { on } from std;

on("shutdown", () => {
  // close DB pool, flush logs, etc.
});

on("shutdown", () => {
  // another cleanup handler — both run
});
```

---

## 7. Reference: example.z

The first example of zehd syntax, demonstrating key language features:

```
import { proxy } from std;
import { Response } from std::types;

get {
  const res: Response = self.response;

  res.headers.set("x-server-location", "lhr1:233");

  return {
    name: "Zehd",
    version: "1.2.3",
  };
}
```

```
// init.z

import { use, rateLimit } from std;

use(rateLimit("rollingWindow", 60s));
```

---

## 8. Open Questions

### Needs Further Discussion
- **Iterator protocol:** The exact trait/interface that makes a type iterable — how do libraries define custom iterators?
- **WebSocket routes:** Deferred to Phase 2. What does a WebSocket handler look like?
- **Long-lived memory:** Arena per-request works for HTTP. WebSockets, SSE streams, background workers, and queues need server-scoped (ref-counted) allocation. Exact strategy TBD when these features are designed.

### Future Design (not blocking Phase 1)
- Template/HTML story: JSX-like, template strings, or separate template files?
- Package manager / module registry design
- Database access story — ORM-like, query builder, raw SQL, or all three?
- Hot reloading / dev server story
- Testing framework — built-in or separate?
- `.env` file support — built-in or tooling?
