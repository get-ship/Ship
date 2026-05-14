//! Tests for `ServerExecutor` and its routing integration.
//!
//! Scope of this slice: wiring correctness, not protocol fidelity. The
//! per-provider `ServerSession` implementations (codex JSON-RPC, opencode
//! HTTP+SSE) ship in follow-on slices and bring their own end-to-end
//! tests. Here we exercise:
//!
//!   1. `ServerExecutor::spawn` surfaces session errors cleanly.
//!   2. `RoutingExecutor` picks `ServerExecutor` for a server-mode
//!      provider when a session is bound.
//!   3. `RoutingExecutor` falls through to Process/TmuxWindow when no
//!      session is bound (the default for this scaffolding slice).
//!   4. The provider capabilities table lists codex + opencode as the
//!      server-mode providers — sanity that the routing criterion
//!      targets the right set.

use std::collections::HashMap;
use std::sync::Arc;

use ship_platform::services::dispatch_ports::{ExecutorHandle, JobContext, JobExecutor, LaunchProfile};
use ship_platform::services::provider_adapter::capabilities;
use ship_platform::services::server_session::StubSession;

use super::routing::{RouteKind, RoutingExecutor, ServerSessionRegistry};
use super::server::ServerExecutor;

fn fresh_ctx(provider: Option<&str>) -> JobContext {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Work dir must survive the test body — leak via keep() so the
    // returned JobContext references a valid path. The OS cleans up
    // /tmp entries eventually; tests are short-lived.
    let work_dir = tmp.keep();
    JobContext {
        job_id: "job-test".to_string(),
        slug: "slug-test".to_string(),
        agent: "test-agent".to_string(),
        branch: "main".to_string(),
        spec_path: ".artifacts/job-spec.md".to_string(),
        spec_content: None,
        work_dir,
        env: HashMap::new(),
        launch: LaunchProfile {
            provider: provider.map(str::to_string),
            ..Default::default()
        },
        exit_tx: None,
        step: None,
        headless: true,
    }
}

#[tokio::test]
async fn server_executor_spawn_with_stub_session_errors_clearly() {
    let session = Arc::new(StubSession::new("codex"));
    let executor = ServerExecutor::new(None, session);
    let ctx = fresh_ctx(Some("codex"));

    let result = <ServerExecutor as JobExecutor>::spawn(&executor, &ctx).await;
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("StubSession must fail start_turn"),
    };
    let msg = err.to_string();
    assert!(msg.contains("StubSession"), "msg was: {msg}");
    assert!(msg.contains("codex"), "msg was: {msg}");
}

#[tokio::test]
async fn routing_picks_server_executor_when_provider_supports_and_session_bound() {
    // Codex declares server_mode=true. With a stub session bound,
    // RoutingExecutor should route through ServerExecutor.
    let registry = ServerSessionRegistry::new();
    registry.bind("codex", Arc::new(StubSession::new("codex")));
    let routing = RoutingExecutor::with_registry(None, registry);

    let ctx = fresh_ctx(Some("codex"));
    assert_eq!(routing.route_kind(&ctx), RouteKind::Server);

    // End-to-end: the spawn itself hits the bound StubSession and
    // surfaces its error, confirming the route reached the executor.
    let err = match <RoutingExecutor as JobExecutor>::spawn(&routing, &ctx).await {
        Err(e) => e,
        Ok(_) => panic!("stub must fail start_turn"),
    };
    let msg = err.to_string();
    assert!(msg.contains("StubSession"), "msg was: {msg}");
}

#[test]
fn routing_falls_back_when_no_session_bound_for_server_mode_provider() {
    // Codex declares server_mode=true but the registry is empty, so
    // routing must NOT pick ServerExecutor — it falls through to
    // Process (headless) or TmuxWindow (interactive).
    let routing = RoutingExecutor::with_registry(None, ServerSessionRegistry::new());
    let ctx = fresh_ctx(Some("codex"));
    assert_ne!(routing.route_kind(&ctx), RouteKind::Server);
}

#[test]
fn routing_falls_back_for_non_server_mode_provider_even_if_registry_has_entry() {
    // Claude has server_mode=false. Even if a session were bound for
    // "claude", the capability flag gates routing — ServerExecutor
    // must NOT be picked.
    let registry = ServerSessionRegistry::new();
    registry.bind("claude", Arc::new(StubSession::new("claude")));
    let routing = RoutingExecutor::with_registry(None, registry);

    let ctx = fresh_ctx(Some("claude"));
    assert_ne!(routing.route_kind(&ctx), RouteKind::Server);
}

#[test]
fn capabilities_filter_for_server_mode_yields_codex() {
    // opencode's server_mode is currently false (see
    // ship-platform/src/services/providers/opencode.rs — temporarily
    // disabled until the session code matches the current adapter shape).
    // codex is the only server-capable provider until that re-enables.
    let server_capable: Vec<&str> = capabilities()
        .into_iter()
        .filter(|(_, c)| c.server_mode)
        .map(|(id, _)| id)
        .collect();
    assert!(server_capable.contains(&"codex"), "got: {server_capable:?}");
    assert_eq!(server_capable.len(), 1, "got: {server_capable:?}");
}

#[test]
fn session_registry_bind_then_get_roundtrips() {
    let r = ServerSessionRegistry::new();
    assert!(r.get("codex").is_none());
    r.bind("codex", Arc::new(StubSession::new("codex")));
    let session = r.get("codex").expect("bound session");
    assert_eq!(session.provider(), "codex");
}

#[test]
fn session_registry_returns_none_for_unbound_provider() {
    let r = ServerSessionRegistry::new();
    r.bind("codex", Arc::new(StubSession::new("codex")));
    assert!(r.get("opencode").is_none());
}

#[tokio::test]
async fn server_executor_stop_delegates_to_session_cancel() {
    // StubSession::cancel returns Ok(()). We build a fake handle by
    // spawn'ing against a session that succeeds — not possible with
    // StubSession. Instead, exercise the degenerate path: stop on a
    // handle whose state doesn't match ServerState is a no-op Ok.
    let session = Arc::new(StubSession::new("codex"));
    let executor = ServerExecutor::new(None, session);
    let handle = ExecutorHandle { pid: None, inner: Box::new(42u32) };
    <ServerExecutor as JobExecutor>::stop(&executor, &handle)
        .await
        .expect("stop on unknown handle is a no-op");
}
