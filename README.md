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

## Editor Setup (VS Code)

The VS Code extension provides syntax highlighting, autocompletion, hover information, and diagnostics for `.z` files.

To run the extension in development mode:

1. **Build and install the CLI** (includes the language server):

   ```sh
   ./scripts/install.sh
   ```

2. **Open the extension project** in VS Code:

   ```sh
   code editors/vscode
   ```

3. **Press F5** to launch the Extension Development Host — a new VS Code window with the zehd extension active.

4. **Open a zehd project** (e.g. `test-app/`) in the new window and edit `.z` files with full language support.

## File Extension

`.z`

## License

TBD
