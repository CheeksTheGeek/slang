//! A runnable SystemVerilog language server (build with `--features server`).
//!
//! It wires [`sv_lang_lsp`]'s intelligence to `tower-lsp`: each open/change
//! feeds the text into the incremental [`Workspace`] and republishes
//! diagnostics; `textDocument/documentSymbol` lists the file's definitions.
//! Run it over stdio from any LSP client.

use std::sync::Mutex;

use sv_lang_db::Workspace;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,
    ws: Mutex<Workspace>,
}

fn uri_to_path(uri: &Url) -> String {
    uri.to_string()
}

fn to_pos(p: sv_lang_lsp::Position) -> Position {
    Position {
        line: p.line,
        character: p.character,
    }
}

fn to_range(r: sv_lang_lsp::Range) -> Range {
    Range {
        start: to_pos(r.start),
        end: to_pos(r.end),
    }
}

fn from_pos(p: Position) -> sv_lang_lsp::Position {
    sv_lang_lsp::Position {
        line: p.line,
        character: p.character,
    }
}

/// Turns a library [`sv_lang_lsp::Location`] back into an LSP `Location`. The
/// path is the document URI string (see [`uri_to_path`]); a URI that no longer
/// parses is dropped.
fn to_location(l: sv_lang_lsp::Location) -> Option<Location> {
    let uri = Url::parse(&l.path).ok()?;
    Some(Location {
        uri,
        range: to_range(l.range),
    })
}

impl Backend {
    fn compute_diagnostics(&self, path: &str) -> Vec<Diagnostic> {
        let mut ws = self.ws.lock().unwrap();
        sv_lang_lsp::diagnostics(&mut ws, path)
            .into_iter()
            .map(|d| Diagnostic {
                range: to_range(d.range),
                severity: Some(match d.severity {
                    sv_lang_lsp::Severity::Error => DiagnosticSeverity::ERROR,
                    sv_lang_lsp::Severity::Warning => DiagnosticSeverity::WARNING,
                    sv_lang_lsp::Severity::Information => DiagnosticSeverity::INFORMATION,
                    sv_lang_lsp::Severity::Hint => DiagnosticSeverity::HINT,
                }),
                code: Some(NumberOrString::String(d.code)),
                source: Some("sv-lang".into()),
                message: d.message,
                ..Default::default()
            })
            .collect()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_symbol_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "sv-lang-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "sv-lang-lsp ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, p: DidOpenTextDocumentParams) {
        let path = uri_to_path(&p.text_document.uri);
        self.ws
            .lock()
            .unwrap()
            .set_file(path.clone(), p.text_document.text);
        let diags = self.compute_diagnostics(&path);
        self.client
            .publish_diagnostics(p.text_document.uri, diags, None)
            .await;
    }

    async fn did_change(&self, p: DidChangeTextDocumentParams) {
        let path = uri_to_path(&p.text_document.uri);
        if let Some(change) = p.content_changes.into_iter().next_back() {
            self.ws.lock().unwrap().set_file(path.clone(), change.text);
        }
        let diags = self.compute_diagnostics(&path);
        self.client
            .publish_diagnostics(p.text_document.uri, diags, None)
            .await;
    }

    async fn document_symbol(
        &self,
        p: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let path = uri_to_path(&p.text_document.uri);
        let mut ws = self.ws.lock().unwrap();
        let syms = sv_lang_lsp::document_symbols(&mut ws, &path)
            .into_iter()
            .map(|s| {
                #[allow(deprecated)]
                DocumentSymbol {
                    name: s.name,
                    detail: Some(format!("{:?}", s.kind)),
                    kind: SymbolKind::MODULE,
                    tags: None,
                    deprecated: None,
                    range: to_range(s.range),
                    selection_range: to_range(s.range),
                    children: None,
                }
            })
            .collect();
        Ok(Some(DocumentSymbolResponse::Nested(syms)))
    }

    async fn hover(&self, p: HoverParams) -> Result<Option<Hover>> {
        let tdp = p.text_document_position_params;
        let path = uri_to_path(&tdp.text_document.uri);
        let mut ws = self.ws.lock().unwrap();
        Ok(
            sv_lang_lsp::hover(&mut ws, &path, from_pos(tdp.position)).map(|h| Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: h.contents,
                }),
                range: Some(to_range(h.range)),
            }),
        )
    }

    async fn goto_definition(
        &self,
        p: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let tdp = p.text_document_position_params;
        let path = uri_to_path(&tdp.text_document.uri);
        let mut ws = self.ws.lock().unwrap();
        let loc = sv_lang_lsp::definition(&mut ws, &path, from_pos(tdp.position))
            .and_then(to_location)
            .map(GotoDefinitionResponse::Scalar);
        Ok(loc)
    }

    async fn references(&self, p: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let tdp = p.text_document_position;
        let path = uri_to_path(&tdp.text_document.uri);
        let mut ws = self.ws.lock().unwrap();
        let locs: Vec<Location> = sv_lang_lsp::references(&mut ws, &path, from_pos(tdp.position))
            .into_iter()
            .filter_map(to_location)
            .collect();
        Ok(Some(locs))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Backend {
        client,
        ws: Mutex::new(Workspace::new()),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
