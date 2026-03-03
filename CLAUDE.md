# zehd

A programming language that *is* a web server. "Next.js but its own language."

## Key References

- [DESIGN.md](./DESIGN.md) — Complete language design specification
- [example.z](./example.z) — First syntax example
- Design decisions MUST be discussed before implementation and recorded in DESIGN.md

## Project Facts

- **Language name:** zehd (spoken as "Zed")
- **File extension:** `.z`
- **Implementation language:** Rust
- **Execution model:** Bytecode VM (Phase 1), Cranelift JIT (Phase 2), trait-based backend
- **Memory model:** Arena per-request, reference-counted for server-scoped state
- **Concurrency:** Implicit green threads — no async/await in user code, tokio underneath
- **Type system:** Static, structural, full inference, generics, no `any` type
- **Null:** No null — `Option<T>` only, `None` maps to JSON `null`
- **Errors:** `Result<T, E>` with algebraic enums, `?` operator, pattern matching
- **Routing:** File-based (directory path = URL path), HTTP methods are top-level blocks
- **Syntax style:** TypeScript-esque, semicolons required, `let`/`const` like JS

## Syntax Quick Reference

- `let` (reassignable) / `const` (not reassignable) — same as JS
- `self` — implicit context in route handlers (request, response, params)
- `import { name } from std::module;` — destructured imports only
- `#[module.attr()]` — attributes require the module to be imported
- `60s`, `5m`, `1h`, `500ms` — built-in time literals (resolve to milliseconds)
- `$"Hello, {name}"` — string interpolation (C# style)
- `get { }`, `post { }`, `delete { }` — HTTP method blocks
- `init { }` — startup configuration block in init.z
- `fn name(args): ReturnType { }` — named functions
- `(args) => expr` or `(args) => { ... }` — arrow functions
- `Result<T, E>`, `Option<T>` — algebraic enums with `?` operator
- `if cond { a } else { b }` — if is an expression, blocks return values
- `match value { ... }` — pattern matching
- Returns auto-serialize to JSON
- Reference semantics by default, `copy()` from std for explicit copies
- `env("KEY")` — environment variable access from std
- `on("event", handler)` — lifecycle hooks from std
- `provide<T>(value)` / `inject<T>()` — type-safe DI, type is the key
- `for item in iterable { }` / `while cond { }` — loops
- `error.z` — defines `?` operator error handling per route subtree (default: 500)
- Route files have two scopes: server scope (top-level, runs at load) and request scope (method blocks, runs per request)

## Project Structure

- `main.z` — entry point, global server config, lifecycle hooks
- `zehd.toml` — project configuration (port, paths, ignore dirs)
- `routes/` — file-based routing (configurable in zehd.toml)
- `lib/` — shared code, not parsed as routes (configurable in zehd.toml)
- `public/` — static files (configurable in zehd.toml)

## Architecture Principles

- The web server IS the runtime — HTTP is not a library, it's the execution environment
- The language stays small — behavior is configured via runtime APIs (init.z), not keywords
- Attributes are import-based reflection metadata — `#[module.attr()]` requires importing `module`
- The type system IS the validator — parsing and validation are the same operation
- Concurrency is invisible — sync code, async runtime
- Modules are the extensibility model — std and third-party use the same system
- Result types, not exceptions — errors are explicit and type-checked
- VM backend is trait-based — stack VM now, Cranelift JIT later
- All external boundaries use the same pipeline — opaque data → typed struct via parse/validation
- Structural typing — types are compared by shape, not name
- No null — Option<T> for absence

## Documentation Standards

- All design decisions must be documented in DESIGN.md before implementation begins
- Documentation is written for both humans and AI agents
- When adding new language features: update DESIGN.md first, then implement
- Code examples in docs should be valid zehd syntax as defined in DESIGN.md

## Development Guidelines

- PoC-first — favor working code over perfect architecture
- Performance is a core value — measure and benchmark early
- Discuss design decisions before implementing
- Keep the language small — resist adding keywords when runtime APIs suffice
