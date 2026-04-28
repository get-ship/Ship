//! LSP protocol surface — capabilities, dispatch, push diagnostics.
//!
//! Lifecycle: `initialize` → `initialized` → notification loop →
//! `shutdown` → `exit`. Sync, single-threaded — driven directly off
//! `lsp-server`'s crossbeam channels. Diagnostics are pushed via
//! `textDocument/publishDiagnostics` on every `didOpen`/`didChange`
//! and cleared on `didClose`.

use anyhow::{Context, Result};
use lsp_server::{Connection, ExtractError, Message, Notification, Request};
use lsp_types::notification::{
    DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
    Notification as _, PublishDiagnostics,
};
use lsp_types::request::{
    CodeActionRequest, Completion, GotoDefinition, HoverRequest, Request as _,
};
use lsp_types::{
    CodeActionProviderCapability, CompletionOptions, Diagnostic, HoverProviderCapability,
    InitializeParams, OneOf, PublishDiagnosticsParams, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};

use crate::diagnostics::compile_diagnostics;
use crate::document::Documents;

/// Run the server on stdio until the client sends `exit`.
pub fn run_stdio() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();
    run(connection)?;
    io_threads.join().context("LSP IO threads join")?;
    Ok(())
}

/// Run the server over an established connection. Factored out of
/// [`run_stdio`] so the smoke test can drive it in-process.
pub fn run(connection: Connection) -> Result<()> {
    let caps = serde_json::to_value(server_capabilities())
        .context("encode server capabilities")?;
    let init_params = connection
        .initialize(caps)
        .context("LSP initialize handshake")?;
    let _params: InitializeParams =
        serde_json::from_value(init_params).context("decode InitializeParams")?;
    main_loop(&connection)
}

fn server_capabilities() -> ServerCapabilities {
    // FULL sync in v0 — `didChange` carries the entire new text. We
    // pivot to INCREMENTAL once tree-sitter-set is wired and we can
    // amortize reparse cost via `Tree::edit`. See `document` module.
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::FULL,
        )),
        // V0 stubs — see README. Capabilities are advertised so editors
        // can wire keybindings; handlers return empty results.
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions::default()),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        ..Default::default()
    }
}

fn main_loop(conn: &Connection) -> Result<()> {
    let mut docs = Documents::new();
    for msg in &conn.receiver {
        match msg {
            Message::Request(req) => {
                if conn.handle_shutdown(&req).context("shutdown check")? {
                    return Ok(());
                }
                handle_request(conn, req)?;
            }
            Message::Notification(note) => {
                handle_notification(conn, &mut docs, note)?;
            }
            Message::Response(_) => {
                // We don't issue server -> client requests in v0.
            }
        }
    }
    Ok(())
}

fn handle_request(conn: &Connection, req: Request) -> Result<()> {
    // V0 stubs: every advertised request returns `null`, which the LSP
    // spec accepts as "no result" for hover, completion, code actions,
    // and definition. Real handlers land in subsequent work.
    match req.method.as_str() {
        HoverRequest::METHOD
        | Completion::METHOD
        | CodeActionRequest::METHOD
        | GotoDefinition::METHOD => empty_response(conn, req.id),
        other => {
            tracing::debug!(method = %other, "unhandled LSP request");
            empty_response(conn, req.id)
        }
    }
}

fn empty_response(conn: &Connection, id: lsp_server::RequestId) -> Result<()> {
    let resp = lsp_server::Response {
        id,
        result: Some(serde_json::Value::Null),
        error: None,
    };
    conn.sender
        .send(Message::Response(resp))
        .context("send response")
}

fn handle_notification(
    conn: &Connection,
    docs: &mut Documents,
    note: Notification,
) -> Result<()> {
    match note.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let params = cast_note::<DidOpenTextDocument>(note)?;
            let uri = params.text_document.uri.clone();
            let text = params.text_document.text;
            refresh(conn, docs, uri, text)?;
        }
        DidChangeTextDocument::METHOD => {
            let params = cast_note::<DidChangeTextDocument>(note)?;
            let uri = params.text_document.uri.clone();
            // FULL sync: the last change carries the entire new text.
            if let Some(change) = params.content_changes.into_iter().last() {
                refresh(conn, docs, uri, change.text)?;
            }
        }
        DidCloseTextDocument::METHOD => {
            let params = cast_note::<DidCloseTextDocument>(note)?;
            let uri = params.text_document.uri.clone();
            docs.close(&uri);
            // Clear stale squiggles on close.
            send_diagnostics(conn, &uri, Vec::new())?;
        }
        _ => tracing::trace!(method = %note.method, "ignored notification"),
    }
    Ok(())
}

fn refresh(
    conn: &Connection,
    docs: &mut Documents,
    uri: Url,
    text: String,
) -> Result<()> {
    let tag = tag_for_uri(&uri);
    let diagnostics = compile_diagnostics(&text, &tag);
    docs.upsert(uri.clone(), text, diagnostics.clone());
    send_diagnostics(conn, &uri, diagnostics)
}

/// Derive the custom-element tag from the URI's file stem. Editor URIs
/// may not be paths (untitled buffers, virtual schemes), so we extract
/// the stem from the URI's last path segment and delegate the kebab +
/// `sail-` policy to `sail_compiler::tag_for_stem`.
fn tag_for_uri(uri: &Url) -> String {
    let stem = uri
        .path_segments()
        .and_then(|s| s.last())
        .and_then(|name| name.rsplit_once('.').map(|(s, _)| s).or(Some(name)))
        .filter(|s| !s.is_empty())
        .unwrap_or("component");
    sail_compiler::tag_for_stem(stem)
}

fn send_diagnostics(
    conn: &Connection,
    uri: &Url,
    diagnostics: Vec<Diagnostic>,
) -> Result<()> {
    let params = PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics,
        version: None,
    };
    let note = Notification {
        method: PublishDiagnostics::METHOD.to_string(),
        params: serde_json::to_value(params)
            .context("encode PublishDiagnosticsParams")?,
    };
    conn.sender
        .send(Message::Notification(note))
        .context("send PublishDiagnostics")
}

fn cast_note<N>(note: Notification) -> Result<N::Params>
where
    N: lsp_types::notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    note.extract::<N::Params>(N::METHOD).map_err(
        |e: ExtractError<Notification>| {
            anyhow::anyhow!("notification cast {}: {:?}", N::METHOD, e)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_server::{Connection, Message, Request, RequestId};
    use lsp_types::notification::{Exit, Initialized};
    use lsp_types::request::{Initialize, Shutdown};
    use lsp_types::{ClientCapabilities, InitializeParams, InitializedParams};
    use std::thread;

    fn req(id: i32, method: &str, params: serde_json::Value) -> Request {
        Request { id: RequestId::from(id), method: method.to_string(), params }
    }
    fn note(method: &str, params: serde_json::Value) -> Notification {
        Notification { method: method.to_string(), params }
    }
    fn recv_response(c: &Connection) -> lsp_server::Response {
        match c.receiver.recv().expect("recv") {
            Message::Response(r) => r,
            other => panic!("expected response, got {other:?}"),
        }
    }

    #[test]
    fn server_initializes_and_shuts_down_cleanly() {
        let (server_conn, client) = Connection::memory();
        let server = thread::spawn(move || run(server_conn));

        let init = serde_json::to_value(InitializeParams {
            capabilities: ClientCapabilities::default(),
            ..Default::default()
        })
        .unwrap();
        client.sender.send(Message::Request(req(1, Initialize::METHOD, init))).unwrap();
        let r = recv_response(&client);
        assert_eq!(r.id, RequestId::from(1));
        assert!(r.error.is_none(), "init error: {:?}", r.error);

        let initialized = serde_json::to_value(InitializedParams {}).unwrap();
        client.sender.send(Message::Notification(note(Initialized::METHOD, initialized))).unwrap();
        client.sender.send(Message::Request(req(2, Shutdown::METHOD, serde_json::Value::Null))).unwrap();
        assert_eq!(recv_response(&client).id, RequestId::from(2));
        client.sender.send(Message::Notification(note(Exit::METHOD, serde_json::Value::Null))).unwrap();

        server.join().expect("server panicked").expect("server returned Err");
    }

    #[test]
    fn tag_for_uri_derives_kebab_with_sail_prefix_when_needed() {
        assert_eq!(tag_for_uri(&Url::parse("file:///x/MyCounter.set").unwrap()), "my-counter");
        assert_eq!(tag_for_uri(&Url::parse("file:///x/counter.set").unwrap()), "sail-counter");
    }

    #[test]
    fn capabilities_advertise_full_sync_and_v0_stubs() {
        let caps = server_capabilities();
        assert!(matches!(
            caps.text_document_sync,
            Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL))
        ));
        assert!(caps.hover_provider.is_some());
        assert!(caps.completion_provider.is_some());
        assert!(caps.code_action_provider.is_some());
        assert!(caps.definition_provider.is_some());
    }
}
