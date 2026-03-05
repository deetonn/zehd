mod backend;
mod completion;
mod config;
mod diagnostics;
mod hover;
mod modules;

use backend::Backend;
use tower_lsp::{LspService, Server};

/// Start the language server on stdin/stdout.
pub async fn run() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
