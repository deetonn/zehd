# zehd

A programming language that *is* a web server. Next.js but its own language.

> **Work in progress.** Everything here is under active development and will change.

## What is zehd?

zehd (spoken "Zed") is a statically typed language where HTTP is not a library — it's the execution environment. Route files *are* endpoints. Method blocks *are* handlers. The type system *is* the validator.

```
// routes/index.z

import { provide, inject } from std;

get {
    return {
        name: "zehd",
        version: "0.1.0"
    };
}

post {
    const body = self.request.body;
    return { received: body };
}
```

No framework. No boilerplate. Just a `.z` file in a `routes/` directory and you have an API.

## Highlights

- **File-based routing** — directory path = URL path
- **Static typing with full inference** — no `any`, no `null`, just `Option<T>` and `Result<T, E>`
- **Built-in time literals** — `60s`, `5m`, `1h`, `500ms`
- **Invisible concurrency** — write sync code, get async performance (tokio underneath)
- **Type-safe dependency injection** — `provide<T>(value)` / `inject<T>()`
- **String interpolation** — `$"Hello, {name}"`

## Architecture

zehd is implemented in Rust as a pipeline of crates:

| Crate | Role |
|-------|------|
| `zehd-tome` | Lexer |
| `zehd-codex` | Parser |
| `zehd-sigil` | Type checker |
| `zehd-rune` | Bytecode compiler |
| `zehd-ward` | Stack VM |
| `zehd-server` | HTTP server (axum) |
| `zehd-lsp` | Language server |
| `zehd-cli` | CLI (`zehd dev`) |

## Quick Start

```sh
cargo build
cargo run -- dev
```

## File Extension

`.z`

## License

TBD
