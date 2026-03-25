//! Core Language Server implementation for PatchLang.
//!
//! Manages open document state and dispatches LSP requests to the
//! diagnostics, symbols, completion, and hover modules.

use std::collections::HashMap;
use std::sync::Mutex;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use patchlang::ast::PatchProgram;
use patchlang::error::ParseResult;

use crate::completion;
use crate::diagnostics;
use crate::hover;
use crate::symbols;

/// Cached state for an open document.
struct DocumentState {
    source: String,
    parse_result: ParseResult,
}

/// The PatchLang language server.
pub struct PatchLangServer {
    client: Client,
    documents: Mutex<HashMap<Url, DocumentState>>,
}

impl PatchLangServer {
    /// Create a new server instance (called by tower-lsp).
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Mutex::new(HashMap::new()),
        }
    }

    /// Re-parse a document, update the cache, and publish diagnostics.
    async fn on_document_change(&self, uri: Url, source: String) {
        let (parse_result, lsp_diagnostics) = diagnostics::build_diagnostics(&source);

        {
            let mut docs = self.documents.lock().unwrap();
            docs.insert(
                uri.clone(),
                DocumentState {
                    source,
                    parse_result,
                },
            );
        }

        self.client
            .publish_diagnostics(uri, lsp_diagnostics, None)
            .await;
    }

    /// Get the cached program for a document, if available.
    fn with_document<F, R>(&self, uri: &Url, f: F) -> Option<R>
    where
        F: FnOnce(&str, &ParseResult, &PatchProgram) -> R,
    {
        let docs = self.documents.lock().unwrap();
        let state = docs.get(uri)?;
        Some(f(&state.source, &state.parse_result, &state.parse_result.program))
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for PatchLangServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".into(), " ".into()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "PatchLang LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let source = params.text_document.text;
        self.on_document_change(uri, source).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        // We use FULL sync, so the first content change has the entire text.
        if let Some(change) = params.content_changes.into_iter().next() {
            self.on_document_change(uri, change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        let mut docs = self.documents.lock().unwrap();
        docs.remove(&uri);
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let result = self.with_document(uri, |source, parse_result, _program| {
            symbols::goto_definition(source, uri, parse_result, position)
        });

        match result {
            Some(Some(location)) => {
                Ok(Some(GotoDefinitionResponse::Scalar(location)))
            }
            _ => Ok(None),
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        let result = self.with_document(uri, |source, _parse_result, program| {
            symbols::extract_symbols(source, program)
        });

        match result {
            Some(syms) if !syms.is_empty() => {
                Ok(Some(DocumentSymbolResponse::Nested(syms)))
            }
            _ => Ok(None),
        }
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let result = self.with_document(uri, |source, _parse_result, program| {
            completion::compute_completions(source, program, position)
        });

        match result {
            Some(items) if !items.is_empty() => {
                Ok(Some(CompletionResponse::Array(items)))
            }
            _ => Ok(None),
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let result = self.with_document(uri, |source, _parse_result, program| {
            hover::compute_hover(source, program, position)
        });

        Ok(result.flatten())
    }
}
