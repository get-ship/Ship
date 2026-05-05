//! LSP server loop — protocol surface and dispatch.
//!
//! Lifecycle: `initialize` -> `initialized` -> notifications loop ->
//! `shutdown` -> `exit`. We push diagnostics on open, change, save and
//! clear on close. Sync, single-threaded — `lsp-server` gives us a
//! pair of crossbeam channels we drive directly.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use lsp_server::{Connection, ExtractError, Message, Notification, Request, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
    Notification as _, PublishDiagnostics,
};
use lsp_types::request::{
    Completion, GotoDefinition, HoverRequest, Request as _, Shutdown,
};
use lsp_types::{
    CompletionOptions, CompletionResponse, Diagnostic, InitializeParams, OneOf,
    PublishDiagnosticsParams, ServerCapabilities, TextDocumentPositionParams,
    TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};

use crate::{diagnostics, expr_intel, uri, workspace};

/// Run the LSP server on stdio until the client sends `exit`.
pub fn run_stdio() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();
    let server_capabilities = serde_json::to_value(server_capabilities())
        .context("encode server capabilities")?;
    let init_params = connection
        .initialize(server_capabilities)
        .context("LSP initialize handshake")?;
    let _params: InitializeParams =
        serde_json::from_value(init_params).context("decode InitializeParams")?;

    main_loop(&connection)?;
    io_threads.join().context("LSP IO threads join")?;
    Ok(())
}

fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        definition_provider: Some(OneOf::Left(true)),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec![".".into()]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// In-memory cache of open documents — URI -> latest content. Needed
/// because `didChange` carries new content but `didSave` doesn't.
#[derive(Default)]
struct Docs {
    text: HashMap<Url, String>,
}

fn main_loop(connection: &Connection) -> Result<()> {
    let mut docs: Docs = Docs::default();
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req).context("shutdown check")? {
                    return Ok(());
                }
                handle_request(connection, &docs, req)?;
            }
            Message::Response(_) => {
                // We don't issue server->client requests in v1.
            }
            Message::Notification(note) => {
                handle_notification(connection, &mut docs, note)?;
            }
        }
    }
    Ok(())
}

fn handle_request(conn: &Connection, docs: &Docs, req: Request) -> Result<()> {
    match req.method.as_str() {
        HoverRequest::METHOD => {
            let id = req.id.clone();
            let params = cast_req::<HoverRequest>(req)?;
            let result = expr_intel_at(docs, &params.text_document_position_params)
                .and_then(|(text, path, off)| expr_intel::hover(&text, &path, off));
            send_response(conn, id, result)
        }
        GotoDefinition::METHOD => {
            let id = req.id.clone();
            let params = cast_req::<GotoDefinition>(req)?;
            let result = expr_intel_at(docs, &params.text_document_position_params)
                .and_then(|(text, path, off)| expr_intel::goto_definition(&text, &path, off));
            send_response(conn, id, result)
        }
        Completion::METHOD => {
            let id = req.id.clone();
            let params = cast_req::<Completion>(req)?;
            let items = expr_intel_at(docs, &params.text_document_position)
                .map(|(text, path, off)| expr_intel::completion(&text, &path, off))
                .unwrap_or_default();
            let result: Option<CompletionResponse> = if items.is_empty() {
                None
            } else {
                Some(CompletionResponse::Array(items))
            };
            send_response(conn, id, result)
        }
        Shutdown::METHOD => Ok(()),
        other => {
            tracing::debug!(method = %other, "unhandled LSP request");
            Ok(())
        }
    }
}

fn expr_intel_at(
    docs: &Docs,
    params: &TextDocumentPositionParams,
) -> Option<(String, std::path::PathBuf, usize)> {
    let uri = &params.text_document.uri;
    let text = docs.text.get(uri).cloned()?;
    let path = uri::url_to_path(uri).ok()?;
    if !uri::is_view_source(&path) {
        return None;
    }
    let offset = expr_intel::position_to_offset(&text, params.position);
    Some((text, path, offset))
}

fn send_response<T: serde::Serialize>(
    conn: &Connection,
    id: lsp_server::RequestId,
    result: T,
) -> Result<()> {
    let payload = serde_json::to_value(result).context("encode LSP response")?;
    let resp = Response { id, result: Some(payload), error: None };
    conn.sender
        .send(Message::Response(resp))
        .context("send LSP response")?;
    Ok(())
}

fn cast_req<R>(req: Request) -> Result<R::Params>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract::<R::Params>(R::METHOD)
        .map(|(_, p)| p)
        .map_err(|e: ExtractError<Request>| anyhow::anyhow!("request cast {}: {:?}", R::METHOD, e))
}

fn handle_notification(
    conn: &Connection,
    docs: &mut Docs,
    note: Notification,
) -> Result<()> {
    match note.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let params = cast_note::<DidOpenTextDocument>(note)?;
            let uri = params.text_document.uri.clone();
            let text = params.text_document.text;
            docs.text.insert(uri.clone(), text.clone());
            publish_for(conn, &uri, &text, /* prefer_disk */ true)?;
        }
        DidChangeTextDocument::METHOD => {
            let params = cast_note::<DidChangeTextDocument>(note)?;
            let uri = params.text_document.uri.clone();
            // FULL sync: each change is the entire new content.
            if let Some(change) = params.content_changes.into_iter().last() {
                docs.text.insert(uri.clone(), change.text.clone());
                publish_for(conn, &uri, &change.text, /* prefer_disk */ false)?;
            }
        }
        DidSaveTextDocument::METHOD => {
            let params = cast_note::<DidSaveTextDocument>(note)?;
            let uri = params.text_document.uri.clone();
            let text = docs
                .text
                .get(&uri)
                .cloned()
                .or(params.text)
                .unwrap_or_default();
            publish_for(conn, &uri, &text, /* prefer_disk */ true)?;
        }
        DidCloseTextDocument::METHOD => {
            let params = cast_note::<DidCloseTextDocument>(note)?;
            let uri = params.text_document.uri.clone();
            docs.text.remove(&uri);
            // Clear diagnostics on close so stale squiggles don't linger.
            send_diagnostics(conn, &uri, Vec::new())?;
        }
        _ => {
            tracing::trace!(method = %note.method, "ignored notification");
        }
    }
    Ok(())
}

/// Run lint and push diagnostics for `uri`. When `prefer_disk` is true
/// we route through `workspace::lint_for_path` so pack rules fire.
fn publish_for(conn: &Connection, uri: &Url, text: &str, prefer_disk: bool) -> Result<()> {
    let path: PathBuf = match uri::url_to_path(uri) {
        Ok(p) => p,
        Err(e) => {
            tracing::debug!(error = %e, %uri, "skip non-file URI");
            return Ok(());
        }
    };
    if !uri::is_view_source(&path) {
        // Not a Ship view — clear any stale diagnostics and bail.
        send_diagnostics(conn, uri, Vec::new())?;
        return Ok(());
    }
    let findings = if prefer_disk {
        workspace::lint_for_path(&path, text)
    } else {
        workspace::lint_in_memory(&path, text)
    };
    // Only emit findings whose `file` matches this open document.
    // `lint_view_dir` also returns siblings — we attach those to their
    // own URIs in a follow-up so multi-file views don't lose context.
    let (own, others) = split_by_file(&path, findings);
    let diagnostics = diagnostics::findings_to_diagnostics(&own);
    send_diagnostics(conn, uri, diagnostics)?;

    for (other_path, other_findings) in others {
        if let Ok(other_uri) = uri::path_to_url(&other_path) {
            let diags = diagnostics::findings_to_diagnostics(&other_findings);
            send_diagnostics(conn, &other_uri, diags)?;
        }
    }
    Ok(())
}

fn split_by_file(
    own_path: &std::path::Path,
    findings: Vec<lint::report::Finding>,
) -> (Vec<lint::report::Finding>, HashMap<PathBuf, Vec<lint::report::Finding>>) {
    let mut own: Vec<lint::report::Finding> = Vec::new();
    let mut others: HashMap<PathBuf, Vec<lint::report::Finding>> = HashMap::new();
    for f in findings {
        if f.file == own_path {
            own.push(f);
        } else {
            others.entry(f.file.clone()).or_default().push(f);
        }
    }
    (own, others)
}

fn send_diagnostics(conn: &Connection, uri: &Url, diagnostics: Vec<Diagnostic>) -> Result<()> {
    let params = PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics,
        version: None,
    };
    let note = Notification {
        method: PublishDiagnostics::METHOD.to_string(),
        params: serde_json::to_value(params).context("encode PublishDiagnosticsParams")?,
    };
    conn.sender
        .send(Message::Notification(note))
        .context("send PublishDiagnostics")?;
    Ok(())
}

fn cast_note<N>(note: Notification) -> Result<N::Params>
where
    N: lsp_types::notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    note.extract::<N::Params>(N::METHOD)
        .map_err(|e: ExtractError<Notification>| anyhow::anyhow!("notification cast {}: {:?}", N::METHOD, e))
}

// ── Internal helper exposed for tests ────────────────────────────────
#[cfg(test)]
pub(crate) fn _publish_for_test(
    conn: &Connection,
    uri: &Url,
    text: &str,
    prefer_disk: bool,
) -> Result<()> {
    publish_for(conn, uri, text, prefer_disk)
}

#[allow(dead_code)]
fn _silence_response_warning(_r: Response) {}
