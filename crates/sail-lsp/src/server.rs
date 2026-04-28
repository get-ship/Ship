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
    CodeActionParams, CodeActionProviderCapability, CompletionOptions, CompletionParams,
    CompletionResponse, Diagnostic, HoverParams, HoverProviderCapability, InitializeParams,
    OneOf, PublishDiagnosticsParams, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url,
};

use crate::code_actions::code_actions;
use crate::completion::completions;
use crate::diagnostics::compile_diagnostics;
use crate::document::Documents;
use crate::hover::hover;

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
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(
                ["<", "{", ".", ":", "="].iter().map(|s| (*s).to_string()).collect(),
            ),
            ..Default::default()
        }),
        // codeAction + definition stay stubs in v1 — the README marks
        // them as such. The capabilities are advertised so editors
        // wire keybindings; handlers return empty results.
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
                handle_request(conn, &docs, req)?;
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

fn handle_request(conn: &Connection, docs: &Documents, req: Request) -> Result<()> {
    match req.method.as_str() {
        Completion::METHOD => {
            let params: CompletionParams = req.extract_params(Completion::METHOD)?;
            let items = docs
                .get(&params.text_document_position.text_document.uri)
                .map(|d| completions(&d.text, params.text_document_position.position))
                .unwrap_or_default();
            send_result(conn, req.id, CompletionResponse::Array(items))
        }
        HoverRequest::METHOD => {
            let params: HoverParams = req.extract_params(HoverRequest::METHOD)?;
            let result = docs
                .get(&params.text_document_position_params.text_document.uri)
                .and_then(|d| hover(&d.text, params.text_document_position_params.position));
            send_result(conn, req.id, result)
        }
        CodeActionRequest::METHOD => {
            let params: CodeActionParams = req.extract_params(CodeActionRequest::METHOD)?;
            let actions = code_actions(&params);
            send_result(conn, req.id, actions)
        }
        // definition stays stubbed in v1.
        GotoDefinition::METHOD => empty_response(conn, req.id),
        other => {
            tracing::debug!(method = %other, "unhandled LSP request");
            empty_response(conn, req.id)
        }
    }
}

trait RequestExt {
    fn extract_params<P: serde::de::DeserializeOwned>(&self, method: &str) -> Result<P>;
}

impl RequestExt for Request {
    fn extract_params<P: serde::de::DeserializeOwned>(&self, method: &str) -> Result<P> {
        serde_json::from_value(self.params.clone())
            .with_context(|| format!("decode params for {method}"))
    }
}

fn send_result<T: serde::Serialize>(
    conn: &Connection,
    id: lsp_server::RequestId,
    value: T,
) -> Result<()> {
    let resp = lsp_server::Response {
        id,
        result: Some(serde_json::to_value(value).context("encode response")?),
        error: None,
    };
    conn.sender
        .send(Message::Response(resp))
        .context("send response")
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
    fn capabilities_advertise_full_sync_and_trigger_chars() {
        let caps = server_capabilities();
        assert!(matches!(
            caps.text_document_sync,
            Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL))
        ));
        assert!(caps.hover_provider.is_some());
        let cp = caps.completion_provider.expect("completion provider");
        let triggers = cp.trigger_characters.expect("trigger chars registered");
        for c in ["<", "{", ".", ":", "="] {
            assert!(triggers.iter().any(|t| t == c), "missing trigger {c}: {triggers:?}");
        }
        assert!(caps.code_action_provider.is_some());
        assert!(caps.definition_provider.is_some());
    }

    #[test]
    fn completion_request_returns_at_least_one_item() {
        use lsp_types::request::Completion as CompletionReq;
        use lsp_types::{
            CompletionParams, PartialResultParams, Position, TextDocumentIdentifier,
            TextDocumentItem, TextDocumentPositionParams, VersionedTextDocumentIdentifier,
            WorkDoneProgressParams,
        };
        use lsp_types::notification::DidOpenTextDocument;
        use lsp_types::DidOpenTextDocumentParams;

        let (server_conn, client) = Connection::memory();
        let server = thread::spawn(move || run(server_conn));

        // initialize
        let init = serde_json::to_value(InitializeParams {
            capabilities: ClientCapabilities::default(),
            ..Default::default()
        }).unwrap();
        client.sender.send(Message::Request(req(1, Initialize::METHOD, init))).unwrap();
        let _ = recv_response(&client);
        let initialized = serde_json::to_value(InitializedParams {}).unwrap();
        client.sender.send(Message::Notification(note(Initialized::METHOD, initialized))).unwrap();

        // open a doc with a known signal so completion has something to surface
        let uri = Url::parse("file:///x/widget.set").unwrap();
        let text = "<script>\nstate count = 0\n</script>\n<template>{</template>".to_string();
        let open = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "set".into(),
                version: 1,
                text,
            },
        };
        client.sender.send(Message::Notification(note(
            DidOpenTextDocument::METHOD,
            serde_json::to_value(open).unwrap(),
        ))).unwrap();
        // drain the diagnostics notification published on open
        let _ = client.receiver.recv().unwrap();

        // completion at the `{` (line 3, col 11) — interpolation context
        let completion = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 3, character: 11 },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        };
        client.sender.send(Message::Request(req(
            10, CompletionReq::METHOD,
            serde_json::to_value(completion).unwrap(),
        ))).unwrap();
        let resp = recv_response(&client);
        assert_eq!(resp.id, RequestId::from(10));
        let val = resp.result.expect("result present");
        let items: Vec<lsp_types::CompletionItem> = match val {
            serde_json::Value::Array(_) => serde_json::from_value(val).unwrap(),
            // CompletionResponse can also serialise as { items, isIncomplete }
            other => serde_json::from_value::<lsp_types::CompletionList>(other)
                .map(|l| l.items)
                .unwrap_or_default(),
        };
        assert!(!items.is_empty(), "expected ≥1 completion item, got {items:?}");
        assert!(items.iter().any(|i| i.label == "count"), "{items:?}");

        // shutdown
        client.sender.send(Message::Request(req(99, Shutdown::METHOD, serde_json::Value::Null))).unwrap();
        let _ = recv_response(&client);
        client.sender.send(Message::Notification(note(Exit::METHOD, serde_json::Value::Null))).unwrap();
        // Suppress diagnostics that drain into the version we don't care about.
        let _ = VersionedTextDocumentIdentifier { uri, version: 1 };
        server.join().expect("server panicked").expect("server returned Err");
    }
}
