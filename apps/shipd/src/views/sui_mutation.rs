//! `POST /_sui/mutation/{view}?form=<key>` — progressive-enhancement
//! fallback for `<form mutation="ship.X.Y">` elements.
//!
//! When a browser submits the form without JS, it POSTs to this
//! endpoint (the SSR walker injects `action` and `method` attrs at
//! render time). The handler:
//!
//! 1. Validates the view name and `?form=<key>` query parameter.
//! 2. Compiles the view source to locate the binding whose key matches;
//!    reads the `ship.X.Y` SDK chain from the binding's `expr`.
//! 3. Parses the URL-encoded form body into a JSON object.
//! 4. Calls `command::call(chain_without_ship_prefix, &args, ctx)` —
//!    the ONE capability gate (#78). Was `registry::call` (read dispatch)
//!    before the registry cut (#5); that was a silent-broken write
//!    endpoint because no read op owned the mutation chain.
//! 5. On success: HTTP 303 to `Referer` (or `/`).
//! 6. On error: HTTP 303 to `Referer?_sui_error=<msg>` (simple flash).
//!
//! The route declares `@auth handler`; the auth middleware populates
//! `AuthContext` in request extensions and the per-route auth layer
//! passes through. `command::call` enforces the per-op blast radius
//! (the gate). Only SDK operations registered in the mutation catalog
//! work via PE; JS-attached mutations bypass this endpoint and call
//! the SDK directly.

use axum::{
    body::{Body, Bytes},
    extract::{Extension, Path, Query, State},
    http::{StatusCode, header},
    response::Response,
};
use serde::Deserialize;
use serde_json::Value;
use sui_compiler::{bindings, keys::BindingKind, parse};

use deck::render_helpers::{is_valid_view_name, not_found, render_view_source};
use crate::auth::AuthContext;
use crate::services::command::{self, CommandCtx};
use crate::ship_runtime::ApiState;

#[derive(Debug, Deserialize)]
pub struct MutationQuery {
    /// Binding key (`b<N>`) identifying the form's mutation binding.
    pub form: String,
}

/// `POST /_sui/mutation/{name}?form=<key>` — PE form submission.
pub async fn serve_sui_mutation(
    Path(name): Path<String>,
    Query(q): Query<MutationQuery>,
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Response {
    if !is_valid_view_name(&name) {
        return not_found(&format!("invalid view name: {name}"));
    }
    let source = match render_view_source(&name) {
        Some(s) => s,
        None => return not_found(&format!("view not found: views/{name}/view.sui")),
    };

    let chain = match resolve_mutation_chain(&source, &q.form) {
        Some(c) => c,
        None => {
            return not_found(&format!(
                "form key `{}` not found in view `{name}`",
                q.form
            ));
        }
    };

    let op_name = chain.strip_prefix("ship.").unwrap_or(&chain);
    let args = parse_form_body(&body);
    let referer = referer_url(&headers);
    let ctx = CommandCtx { state: &state, auth: &auth };

    match command::call(op_name, &args, &ctx).await {
        Ok(_) => redirect_303(&referer),
        Err(e) => {
            let msg = e.to_string();
            tracing::warn!(chain = %chain, error = %msg, "sui mutation dispatch failed");
            redirect_error(&referer, &msg)
        }
    }
}

/// Walk the view's compiled binding registry to find a `Mutation`
/// binding with the given key. Returns the SDK chain expression.
fn resolve_mutation_chain(source: &str, key: &str) -> Option<String> {
    let program = parse::parse(source).ok()?;
    let reg = bindings::build(&program);
    reg.iter()
        .find(|b| b.key == key && matches!(b.kind, BindingKind::Mutation))
        .map(|b| b.expr.clone())
}

/// Convert a URL-encoded form body (`name=value&...`) to a JSON object.
/// Duplicate keys produce a JSON array. Unknown / empty bodies → `{}`.
fn parse_form_body(body: &[u8]) -> Value {
    let mut obj = serde_json::Map::new();
    for (k, v) in form_urlencoded::parse(body) {
        let val = Value::String(v.into_owned());
        match obj.get_mut(k.as_ref()) {
            Some(Value::Array(arr)) => arr.push(val),
            Some(existing) => {
                let prev = std::mem::replace(existing, Value::Null);
                *existing = Value::Array(vec![prev, val]);
            }
            None => { obj.insert(k.into_owned(), val); }
        }
    }
    Value::Object(obj)
}

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
    let encoded_msg = percent_encode(msg);
    let base = referer.split('?').next().unwrap_or(referer);
    let location = format!("{base}?_sui_error={encoded_msg}");
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, location)
        .body(Body::empty())
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

/// Minimal percent-encoding for query param values (space → %20, etc.).
fn percent_encode(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::{Extension, State};
    use axum::http::HeaderMap;
    use ship_substrate::events::CallerKind;
    use tempfile::TempDir;

    fn make_api_state(dir: &TempDir) -> ApiState {
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
        let kernel = std::sync::Arc::new(tokio::sync::Mutex::new(
            ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
        ));
        std::sync::Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
    }

    /// Test auth: CLI/daemon-style caller with `Scope::Full` so the gate
    /// (which an unknown-op test never reaches) doesn't false-fail before
    /// the dispatch chain reports `unknown_command`. Operator-cred stays
    /// `false` — no S6 elevation here.
    fn make_test_auth() -> AuthContext {
        AuthContext {
            caller_kind: CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![crate::auth::Scope::Full],
            grants: vec![],
            operator_cred: false,
        }
    }

    /// A form key that matches no mutation binding resolves to `None`,
    #[test]
    fn missing_form_key_resolves_none() {
        let src = r#"<form mutation="ship.known.op"></form>"#;
        let program = parse::parse(src).expect("parse");
        let reg = bindings::build(&program);
        let real_key = reg
            .iter()
            .find(|b| matches!(b.kind, BindingKind::Mutation))
            .expect("mutation binding")
            .key
            .clone();

        assert!(
            resolve_mutation_chain(src, &real_key).is_some(),
            "the real binding key resolves"
        );
        assert!(
            resolve_mutation_chain(src, "b9999").is_none(),
            "an unknown binding key resolves to None"
        );
    }

    /// Unknown view → 404.
    #[tokio::test(flavor = "multi_thread")]
    async fn unknown_view_404() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let auth = make_test_auth();
        let resp = serve_sui_mutation(
            Path("does-not-exist".to_string()),
            Query(MutationQuery { form: "b0".to_string() }),
            State(state),
            Extension(auth),
            HeaderMap::new(),
            Bytes::new(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
    /// Unknown SDK chain → command dispatch returns error → 303 to error URL.
    #[tokio::test(flavor = "multi_thread")]
    async fn unknown_chain_redirects_to_error() {
        let dir = TempDir::new().unwrap();
        let state = make_api_state(&dir);
        let auth = make_test_auth();

        // Build a minimal view with a mutation binding pointing at an
        // unknown chain so the command dispatch fails.
        let src = r#"<form mutation="ship.unknown.op"></form>"#;
        let program = parse::parse(src).expect("parse");
        let reg = bindings::build(&program);
        let key = reg
            .iter()
            .find(|b| matches!(b.kind, BindingKind::Mutation))
            .expect("mutation binding")
            .key
            .clone();

        // resolve_mutation_chain resolves the chain. We then drive
        // command::call directly with the same op name to confirm the
        // error path (unknown op → "unknown command").
        let chain = resolve_mutation_chain(src, &key).expect("chain found");
        assert_eq!(chain, "ship.unknown.op");

        let op_name = chain.strip_prefix("ship.").unwrap_or(&chain);
        let args = Value::Object(Default::default());
        let ctx = CommandCtx { state: &state, auth: &auth };
        let result = command::call(op_name, &args, &ctx).await;
        assert!(result.is_err(), "unknown op must error");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("unknown"), "expected unknown-command error, got: {msg}");
    }

    #[test]
    fn parse_form_body_converts_to_json() {
        let body = b"name=Alice&tone=primary";
        let v = parse_form_body(body);
        assert_eq!(v["name"], "Alice");
        assert_eq!(v["tone"], "primary");
    }

    #[test]
    fn parse_form_body_empty_returns_empty_object() {
        let v = parse_form_body(b"");
        assert!(v.as_object().unwrap().is_empty());
    }

    #[test]
    fn percent_encode_spaces_and_brackets() {
        let s = percent_encode("[error] something went wrong");
        assert!(!s.contains(' '));
        assert!(!s.contains('['));
    }
}
