//! Ship-free in-memory data port for the deck-demo live-data + mutation showcases.
//!
//! Provides the three `/_sui/` endpoints the SUI wire protocol requires:
//! - `GET /_sui/views/{name}/sse`         — per-view live-data SSE channel
//! - `POST /_sui/mutation/{name}?form=`   — progressive-enhancement mutation
//! - `GET /_sui/data?expr=`               — single-expression SSE stream
//!
//! The host contract matches `apps/shipd/src/views/sui_*.rs` verbatim, with
//! the kernel and registry removed. `resolve_demo` is the local registry.
//! All SSE encoding comes from `deck_core::live_wire` — no copies.
//!
//! The vocabulary is `ship.demo.*`. The SUI compiler bakes `ship.<ns>.<method>`
//! into every form action and SSE subscription; using `ship.demo.*` here proves
//! that vocabulary is a contract, not a Ship coupling.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;

use axum::{
    Router,
    body::{Body, Bytes},
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::Response,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::{Mutex, broadcast};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

use deck_core::live_wire::{format_keepalive, format_patch_set, format_patch_signals};
use deck_core::render_helpers::is_valid_view_name;
use futures_util::StreamExt as _;
use sui_compiler::{bindings, keys::BindingKind, parse};

// ── store ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TodoItem {
    pub id: u64,
    pub text: String,
    pub done: bool,
}

/// Shared in-memory state for the demo views.
/// The background ticker advances `tick` every second and sends a `bump`
/// so every open SSE connection re-resolves and emits a fresh patch.
pub struct DemoStore {
    pub todos: Mutex<Vec<TodoItem>>,
    pub tick: AtomicU64,
    /// Broadcast channel: any send wakes all SSE loops.
    /// Capacity 16 — bursts are coalesced before re-resolve anyway.
    bump: broadcast::Sender<()>,
    /// Token cancelled on shutdown; the background ticker exits cleanly.
    cancel: CancellationToken,
}

impl DemoStore {
    pub fn new() -> Arc<Self> {
        let (bump, _) = broadcast::channel(16);
        let cancel = CancellationToken::new();
        let store = Arc::new(Self {
            todos: Mutex::new(Vec::new()),
            tick: AtomicU64::new(0),
            bump,
            cancel,
        });
        // Spawn the background ticker; it holds a weak reference so the store
        // can be dropped (and the cancel token fired) without leaking the task.
        let weak = Arc::downgrade(&store);
        let cancel_child = store.cancel.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            interval.tick().await; // skip immediate tick
            loop {
                tokio::select! {
                    _ = cancel_child.cancelled() => break,
                    _ = interval.tick() => {
                        let Some(s) = weak.upgrade() else { break };
                        s.tick.fetch_add(1, Ordering::Relaxed);
                        let _ = s.bump.send(());
                    }
                }
            }
        });
        store
    }

    /// Shut down the background ticker.
    pub fn shutdown(&self) {
        self.cancel.cancel();
    }

    /// Subscribe to bump notifications.
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.bump.subscribe()
    }
}

// ── resolver ─────────────────────────────────────────────────────────────────

/// Resolve a `demo.*` chain (already stripped of the `ship.` prefix).
/// Returns `Ok(Value)` or an error string. Mirrors `services::registry::call`.
pub async fn resolve_demo(chain: &str, args: &Value, store: &DemoStore) -> Result<Value, String> {
    match chain {
        "demo.clock" => {
            let now = store.tick.load(Ordering::Relaxed);
            Ok(json!({ "now": now }))
        }
        "demo.todos" => {
            let items = store.todos.lock().await;
            let list: Vec<Value> = items
                .iter()
                .map(|t| json!({ "id": t.id, "text": t.text, "done": t.done }))
                .collect();
            Ok(json!({ "items": list }))
        }
        "demo.addTodo" => {
            let text = args["text"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "addTodo: missing `text`".to_string())?;
            let mut items = store.todos.lock().await;
            let id = items.last().map(|t| t.id + 1).unwrap_or(1);
            items.push(TodoItem {
                id,
                text: text.to_string(),
                done: false,
            });
            Ok(json!({ "id": id }))
        }
        "demo.toggleTodo" => {
            let id = args["id"]
                .as_u64()
                .ok_or_else(|| "toggleTodo: missing numeric `id`".to_string())?;
            let mut items = store.todos.lock().await;
            let item = items
                .iter_mut()
                .find(|t| t.id == id)
                .ok_or_else(|| format!("toggleTodo: no item with id {id}"))?;
            item.done = !item.done;
            Ok(json!({ "done": item.done }))
        }
        // `demo.boom` always returns an error — used by views/boom to exercise
        // the {#while-error} error-recovery branch without white-screening.
        "demo.boom" => Err("demo.boom: intentional error for the error-boundary showcase".to_string()),
        _ => Err(format!("unknown_service: demo resolver has no op `{chain}`")),
    }
}

/// Strip the `ship.` prefix and parse args from a bare `demo.*()` expression.
/// Returns `(chain_without_ship_prefix, args_json)`.
fn parse_demo_expr(expr: &str) -> Option<(String, Value)> {
    let (head, tail) = expr.split_once('(')?;
    let raw_args = tail.strip_suffix(')')?;
    let chain = head.strip_prefix("ship.")?;
    if !chain.starts_with("demo.") {
        return None;
    }
    let args = if raw_args.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(raw_args.trim()).ok()?
    };
    Some((chain.to_string(), args))
}

// ── keepalive ─────────────────────────────────────────────────────────────────

const KEEPALIVE_SECS: u64 = 20;

async fn keepalive_loop(tx: mpsc::Sender<Result<String, std::io::Error>>, cancel: CancellationToken) {
    let mut ticker = tokio::time::interval(Duration::from_secs(KEEPALIVE_SECS));
    ticker.tick().await; // skip immediate
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = ticker.tick() => {
                if tx.send(Ok(format_keepalive().to_string())).await.is_err() {
                    break;
                }
            }
        }
    }
}

// ── SSE per-view handler ──────────────────────────────────────────────────────

/// `GET /_sui/views/{name}/sse`
/// Mirror: `apps/shipd/src/views/sui_sse.rs` (kernel removed).
async fn serve_sse(
    Path(name): Path<String>,
    State(store): State<Arc<DemoStore>>,
) -> Response {
    serve_sse_inner(name, store).await
}

async fn serve_sse_inner(name: String, store: Arc<DemoStore>) -> Response {
    if !is_valid_view_name(&name) {
        return sse_error(StatusCode::BAD_REQUEST, "invalid view name");
    }
    let source = match deck_core::render_helpers::render_view_source(&name) {
        Some(s) => s,
        None => return sse_error(StatusCode::NOT_FOUND, "view not found"),
    };

    let program = match sui_compiler::parse::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            let msg = e.render(&source);
            tracing::warn!(view = %name, error = %msg, "data_port: parse error");
            return sse_error(StatusCode::INTERNAL_SERVER_ERROR, "view parse error");
        }
    };

    // Collect live <data> declarations.
    let live_data: Vec<(String, String)> = program
        .data
        .iter()
        .filter(|d| d.live)
        .map(|d| (d.name.clone(), d.source.source.clone()))
        .collect();

    let (tx, rx) = mpsc::channel::<Result<String, std::io::Error>>(64);
    let cancel = CancellationToken::new();

    if live_data.is_empty() {
        // No live data — just keepalive.
        let tx2 = tx.clone();
        let cancel2 = cancel.clone();
        tokio::spawn(async move { keepalive_loop(tx2, cancel2).await });
        drop(tx);
    } else {
        for (data_name, expr_source) in live_data {
            let tx = tx.clone();
            let store = store.clone();
            let cancel = cancel.clone();
            tokio::spawn(async move {
                run_live_source(data_name, expr_source, store, tx, cancel).await;
            });
        }
        // Keepalive.
        let tx_hb = tx.clone();
        let cancel_hb = cancel.clone();
        tokio::spawn(async move { keepalive_loop(tx_hb, cancel_hb).await });
        drop(tx);
    }

    let stream = ReceiverStream::new(rx);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| Response::new(Body::from("sse stream init failed")))
}

/// Drive one live data source: send initial value, then re-resolve on each bump.
async fn run_live_source(
    data_name: String,
    expr_source: String,
    store: Arc<DemoStore>,
    tx: mpsc::Sender<Result<String, std::io::Error>>,
    cancel: CancellationToken,
) {
    // Parse the expression.
    let (chain, args) = match parse_demo_expr(&expr_source) {
        Some(p) => p,
        None => {
            let frame = format_patch_signals(&data_name, &json!({"kind": "loading"}));
            let _ = tx.send(Ok(frame)).await;
            return;
        }
    };

    // Send initial value.
    let initial_value = match resolve_demo(&chain, &args, &store).await {
        Ok(v) => json!({"kind": "ok", "value": v}),
        Err(e) => {
            tracing::warn!(chain = %chain, error = %e, "data_port: initial resolve failed");
            json!({"kind": "error", "message": e})
        }
    };
    let frame = format_patch_signals(&data_name, &initial_value);
    if tx.send(Ok(frame)).await.is_err() {
        return;
    }

    // Subscribe and loop.
    let mut bump_rx = store.subscribe();
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            result = bump_rx.recv() => {
                match result {
                    Ok(()) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        // Coalesce bursts.
                        while bump_rx.try_recv().is_ok() {}
                        let value = match resolve_demo(&chain, &args, &store).await {
                            Ok(v) => json!({"kind": "ok", "value": v}),
                            Err(e) => json!({"kind": "error", "message": e}),
                        };
                        let frame = format_patch_signals(&data_name, &value);
                        if tx.send(Ok(frame)).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}

// ── mutation handler ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct MutationQuery {
    form: String,
}

/// Maximum body size for a PE form submission (mirrors shipd's per-route limit).
/// Without this guard, axum buffers arbitrarily large request bodies before the
/// handler is invoked. 64 KB is well above any real HTML form.
const MAX_MUTATION_BODY_BYTES: usize = 64 * 1024;

/// `POST /_sui/mutation/{name}?form=<key>` — PE form submission.
/// Mirror: `apps/shipd/src/views/sui_mutation.rs` (kernel removed).
async fn serve_mutation(
    Path(name): Path<String>,
    Query(q): Query<MutationQuery>,
    State(store): State<Arc<DemoStore>>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Response {
    if body.len() > MAX_MUTATION_BODY_BYTES {
        return Response::builder()
            .status(StatusCode::PAYLOAD_TOO_LARGE)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from("request body too large"))
            .unwrap_or_else(|_| Response::new(Body::empty()));
    }
    if !is_valid_view_name(&name) {
        return not_found(&format!("invalid view name: {name}"));
    }
    let source = match deck_core::render_helpers::render_view_source(&name) {
        Some(s) => s,
        None => return not_found(&format!("view not found: views/{name}/page.sui")),
    };

    // Resolve the mutation chain via the compiled binding registry.
    let chain = match resolve_mutation_chain(&source, &q.form) {
        Some(c) => c,
        None => {
            return not_found(&format!(
                "form key `{}` not found in view `{name}`",
                q.form
            ));
        }
    };

    // Strip `ship.` prefix, parse form body.
    let registry_name = chain.strip_prefix("ship.").unwrap_or(&chain).to_string();
    let args = parse_form_body(&body);
    let referer = referer_url(&headers);

    // Dispatch to local resolver and bump subscribers on success.
    match resolve_demo(&registry_name, &args, &store).await {
        Ok(_) => {
            let _ = store.bump.send(());
            redirect_303(&referer)
        }
        Err(e) => {
            tracing::warn!(chain = %chain, error = %e, "data_port: mutation failed");
            redirect_error(&referer, &e)
        }
    }
}

/// Walk the compiled binding registry for a `Mutation` with the given key.
fn resolve_mutation_chain(source: &str, key: &str) -> Option<String> {
    let program = parse::parse(source).ok()?;
    let reg = bindings::build(&program);
    reg.iter()
        .find(|b| b.key == key && matches!(b.kind, BindingKind::Mutation))
        .map(|b| b.expr.clone())
}

/// Convert URL-encoded form body to a JSON object.
pub fn parse_form_body(body: &[u8]) -> Value {
    let mut obj = serde_json::Map::new();
    for (k, v) in form_urlencoded::parse(body) {
        let val = Value::String(v.into_owned());
        match obj.get_mut(k.as_ref()) {
            Some(Value::Array(arr)) => arr.push(val),
            Some(existing) => {
                let prev = std::mem::replace(existing, Value::Null);
                *existing = Value::Array(vec![prev, val]);
            }
            None => {
                obj.insert(k.into_owned(), val);
            }
        }
    }
    Value::Object(obj)
}

// ── data endpoint ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct DataQuery {
    expr: String,
}

/// `GET /_sui/data?expr=` — single-expression SSE stream.
/// Mirror: `apps/shipd/src/views/sui_data.rs` (kernel removed).
async fn serve_data(
    Query(q): Query<DataQuery>,
    State(store): State<Arc<DemoStore>>,
) -> Response {
    let expr = q.expr.trim().to_string();
    if !is_valid_expr(&expr) {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from("invalid expr"))
            .unwrap_or_else(|_| Response::new(Body::from("bad request")));
    }

    let (chain, args) = match parse_demo_expr(&expr) {
        Some(p) => p,
        None => {
            // Fallback: emit null patch and hold open.
            let frame = format_patch_set(&Value::Null, &expr);
            let stream = futures_util::stream::once(async move { Ok::<String, std::io::Error>(frame) })
                .chain(futures_util::stream::pending());
            return sse_stream(stream);
        }
    };

    let (tx, rx) = mpsc::channel::<Result<String, std::io::Error>>(64);
    let cancel = CancellationToken::new();

    tokio::spawn(async move {
        // Send initial.
        let initial = match resolve_demo(&chain, &args, &store).await {
            Ok(v) => v,
            Err(_) => Value::Null,
        };
        if tx.send(Ok(format_patch_set(&initial, &expr))).await.is_err() {
            return;
        }

        // Subscribe and loop.
        let mut bump_rx = store.subscribe();
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                result = bump_rx.recv() => {
                    match result {
                        Ok(()) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            while bump_rx.try_recv().is_ok() {}
                            let value = resolve_demo(&chain, &args, &store).await.unwrap_or(Value::Null);
                            if tx.send(Ok(format_patch_set(&value, &expr))).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            }
        }
    });

    let stream = ReceiverStream::new(rx);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| Response::new(Body::from("sse stream init failed")))
}

fn is_valid_expr(expr: &str) -> bool {
    if expr.is_empty() || expr.len() > 1024 {
        return false;
    }
    if expr.contains('\n') || expr.contains(';') || expr.contains('`') {
        return false;
    }
    let first = expr.chars().next().unwrap();
    first.is_ascii_alphabetic() || first == '_' || first == '$'
}

// ── router ────────────────────────────────────────────────────────────────────

/// Build the data-port router. Merge into the host's main router via
/// `Router::merge(data_port::router(store))`.
pub fn router(store: Arc<DemoStore>) -> Router {
    Router::new()
        .route("/_sui/views/{name}/sse", get(serve_sse))
        .route("/_sui/mutation/{name}", post(serve_mutation))
        .route("/_sui/data", get(serve_data))
        .with_state(store)
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn referer_url(headers: &axum::http::HeaderMap) -> String {
    headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("/")
        .to_string()
}

fn redirect_303(location: &str) -> Response {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, location)
        .body(Body::empty())
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

fn redirect_error(referer: &str, msg: &str) -> Response {
    let encoded = form_urlencoded::byte_serialize(msg.as_bytes()).collect::<String>();
    let base = referer.split('?').next().unwrap_or(referer);
    let location = format!("{base}?_sui_error={encoded}");
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, location)
        .body(Body::empty())
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

fn not_found(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("not found")))
}

fn sse_error(status: StatusCode, msg: &str) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("error")))
}

fn sse_stream<S>(stream: S) -> Response
where
    S: futures_util::Stream<Item = Result<String, std::io::Error>> + Send + 'static,
{
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| Response::new(Body::from("sse stream failed")))
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_demo_expr_clock() {
        let (chain, args) = parse_demo_expr("ship.demo.clock()").unwrap();
        assert_eq!(chain, "demo.clock");
        assert!(args.is_object());
    }

    #[test]
    fn parse_demo_expr_todos() {
        let (chain, _) = parse_demo_expr("ship.demo.todos()").unwrap();
        assert_eq!(chain, "demo.todos");
    }

    #[test]
    fn parse_demo_expr_rejects_non_demo() {
        assert!(parse_demo_expr("ship.git.overview()").is_none());
    }

    #[test]
    fn parse_demo_expr_rejects_missing_ship_prefix() {
        assert!(parse_demo_expr("demo.clock()").is_none());
    }

    #[tokio::test]
    async fn resolve_demo_clock_returns_now() {
        let store = DemoStore::new();
        store.tick.store(42, Ordering::Relaxed);
        let v = resolve_demo("demo.clock", &json!({}), &store).await.unwrap();
        assert_eq!(v["now"], 42u64);
    }

    #[tokio::test]
    async fn resolve_demo_todos_empty() {
        let store = DemoStore::new();
        let v = resolve_demo("demo.todos", &json!({}), &store).await.unwrap();
        assert_eq!(v["items"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn resolve_demo_add_and_list() {
        let store = DemoStore::new();
        resolve_demo("demo.addTodo", &json!({"text": "hello"}), &store)
            .await
            .unwrap();
        let v = resolve_demo("demo.todos", &json!({}), &store).await.unwrap();
        let items = v["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["text"], "hello");
        assert_eq!(items[0]["done"], false);
    }

    #[tokio::test]
    async fn resolve_demo_toggle_todo() {
        let store = DemoStore::new();
        resolve_demo("demo.addTodo", &json!({"text": "flip me"}), &store)
            .await
            .unwrap();
        let id = {
            let items = store.todos.lock().await;
            items[0].id
        };
        let v = resolve_demo("demo.toggleTodo", &json!({"id": id}), &store)
            .await
            .unwrap();
        assert_eq!(v["done"], true);
    }

    #[tokio::test]
    async fn resolve_demo_unknown_op_errors() {
        let store = DemoStore::new();
        let r = resolve_demo("demo.notReal", &json!({}), &store).await;
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("unknown_service"));
    }

    #[test]
    fn parse_form_body_basic() {
        let v = parse_form_body(b"text=hello+world");
        assert_eq!(v["text"], "hello world");
    }

    #[test]
    fn parse_form_body_empty() {
        let v = parse_form_body(b"");
        assert!(v.as_object().unwrap().is_empty());
    }

    #[test]
    fn resolve_mutation_chain_finds_key() {
        let src = r#"<form mutation="ship.demo.addTodo"><input name="text"/></form>"#;
        let program = parse::parse(src).unwrap();
        let reg = bindings::build(&program);
        let key = reg
            .iter()
            .find(|b| matches!(b.kind, BindingKind::Mutation))
            .expect("mutation binding")
            .key
            .clone();
        let chain = resolve_mutation_chain(src, &key).unwrap();
        assert_eq!(chain, "ship.demo.addTodo");
    }

    #[test]
    fn resolve_mutation_chain_missing_key_is_none() {
        let src = r#"<form mutation="ship.demo.addTodo"></form>"#;
        assert!(resolve_mutation_chain(src, "b9999").is_none());
    }
}
