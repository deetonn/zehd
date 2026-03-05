use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use zehd_sigil::ModuleTypes;

use crate::{completion, config, diagnostics, hover, modules};

pub struct Backend {
    client: Client,
    documents: Mutex<HashMap<Url, String>>,
    project_root: Mutex<Option<PathBuf>>,
    module_types: Mutex<ModuleTypes>,
    analysis_cache: Mutex<HashMap<Url, diagnostics::AnalysisResult>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Mutex::new(HashMap::new()),
            project_root: Mutex::new(None),
            module_types: Mutex::new(zehd_sigil::std_module_types()),
            analysis_cache: Mutex::new(HashMap::new()),
        }
    }

    async fn publish_diagnostics(&self, uri: Url, source: &str) {
        let module_types = self.module_types.lock().unwrap().clone();
        let (diags, analysis) =
            diagnostics::compute_with_analysis(&uri, source, &module_types);
        if let Some(analysis) = analysis {
            self.analysis_cache
                .lock()
                .unwrap()
                .insert(uri.clone(), analysis);
        }
        self.client
            .publish_diagnostics(uri, diags, None)
            .await;
    }

    /// Rebuild module types by discovering and type-checking all user modules.
    fn rebuild_module_types(&self) {
        let root = self.project_root.lock().unwrap().clone();
        let Some(root) = root else { return };

        let module_dirs = config::load_module_dirs(&root);
        let discovered = modules::discover_modules(&module_dirs);
        let base = zehd_sigil::std_module_types();
        let types = modules::extract_module_types(discovered, &base);
        *self.module_types.lock().unwrap() = types;
    }

    /// Check if a URI points to a file inside a module directory.
    fn is_module_file(&self, uri: &Url) -> bool {
        let root = self.project_root.lock().unwrap().clone();
        let Some(root) = root else { return false };
        let Ok(file_path) = uri.to_file_path() else {
            return false;
        };

        let module_dirs = config::load_module_dirs(&root);
        module_dirs
            .iter()
            .any(|(_, dir)| file_path.starts_with(dir))
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Extract project root from root_uri or workspace folders.
        let root = params
            .root_uri
            .and_then(|uri| uri.to_file_path().ok())
            .or_else(|| {
                params
                    .workspace_folders
                    .as_ref()
                    .and_then(|folders| folders.first())
                    .and_then(|f| f.uri.to_file_path().ok())
            });

        if let Some(root) = root {
            *self.project_root.lock().unwrap() = Some(root);
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".into(), ":".into()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "zehd-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.rebuild_module_types();
        self.client
            .log_message(MessageType::INFO, "zehd language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.documents
            .lock()
            .unwrap()
            .insert(uri.clone(), text.clone());
        self.publish_diagnostics(uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        // FULL sync — single change event with entire document.
        if let Some(change) = params.content_changes.into_iter().last() {
            self.documents
                .lock()
                .unwrap()
                .insert(uri.clone(), change.text.clone());
            if self.is_module_file(&uri) {
                self.rebuild_module_types();
            }
            self.publish_diagnostics(uri, &change.text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        if self.is_module_file(&uri) {
            self.rebuild_module_types();
        }
        let text = self.documents.lock().unwrap().get(&uri).cloned();
        if let Some(text) = text {
            self.publish_diagnostics(uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.lock().unwrap().remove(&uri);
        self.analysis_cache.lock().unwrap().remove(&uri);
        // Clear diagnostics for the closed file.
        self.client
            .publish_diagnostics(uri, Vec::new(), None)
            .await;
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let source = self.documents.lock().unwrap().get(&uri).cloned();
        let Some(source) = source else {
            return Ok(None);
        };

        let items = {
            let module_types = self.module_types.lock().unwrap().clone();
            let cache = self.analysis_cache.lock().unwrap();
            let analysis = cache.get(&uri);
            completion::completions(&source, position, analysis, &module_types)
        };

        self.client
            .log_message(
                MessageType::INFO,
                format!("completion: {} items at {}:{}", items.len(), position.line, position.character),
            )
            .await;

        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CompletionResponse::Array(items)))
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let source = self.documents.lock().unwrap().get(&uri).cloned();
        let Some(source) = source else {
            return Ok(None);
        };

        let result = {
            let module_types = self.module_types.lock().unwrap().clone();
            let cache = self.analysis_cache.lock().unwrap();
            let analysis = cache.get(&uri);
            hover::hover_info(&source, position, analysis, &module_types)
        };

        Ok(result)
    }
}
