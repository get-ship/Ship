//! Per-source drivers for the `/_sui/views/{name}/sse` channel.
//!
//! The handler in `sui_sse.rs` parses a view's `<data live />` declarations and
//! spawns one task per source; the loops that actually push frames live here.
//! Two kinds:
//! - `run_data_source` — namespace-event-driven: spawns a kernel actor, sends
//!   the initial resolve, then re-resolves on each subscribed event.
//! - `run_signal_data_source` — signal-backed: subscribes to a shared
//!   `Signal<T>` updated by one background task.
//!
//! Every loop `select!`s against a `CancellationToken` so a SIGTERM closes the
//! stream promptly (#256) instead of holding the axum graceful-shutdown drain
//! open until the client disconnects.

use std::time::Duration;

use ship_substrate::events::ActorConfig;
use ship_substrate::gen_ulid;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::ship_runtime::ApiState;
use super::sui_data_resolvers::{parse_expr, resolve as resolve_value, subscribe_namespaces};
use super::sui_patches::format_patch_signals;
use super::git_signal;

/// Keepalive comment interval so proxies don't close idle connections.
pub(super) const KEEPALIVE_SECS: u64 = 20;

/// Emit a `: ship-keepalive` comment every `KEEPALIVE_SECS` so proxies don't
/// close the idle connection. Exits on client disconnect OR shutdown (#256) —
/// without the latter this loop kept the SSE stream (and the graceful-shutdown
/// drain) open until SIGKILL.
pub(super) async fn keepalive_loop(
    tx: mpsc::Sender<Result<String, std::io::Error>>,
    cancel: CancellationToken,
) {
    let mut ticker = tokio::time::interval(Duration::from_secs(KEEPALIVE_SECS));
    ticker.tick().await; // skip immediate tick
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = ticker.tick() => {
                if tx.send(Ok(": ship-keepalive\n\n".to_string())).await.is_err() {
                    break;
                }
            }
        }
    }
}

/// Drive one live data source: send initial value then re-resolve on
/// every kernel event in the subscribed namespaces.
pub(super) async fn run_data_source(
    data_name: String,
    expr_source: String,
    state: ApiState,
    tx: mpsc::Sender<Result<String, std::io::Error>>,
    cancel: CancellationToken,
) {
    let parsed = match parse_expr(&expr_source) {
        Some(p) => p,
        None => {
            // Unparseable expression — send a loading envelope and exit.
            tracing::warn!(expr = %expr_source, "sui_sse: unparseable data source expression");
            let frame = format_patch_signals(&data_name, &serde_json::json!({"kind": "loading"}));
            let _ = tx.send(Ok(frame)).await;
            return;
        }
    };

    // Resolve and send the initial value.
    let initial = resolve_initial(&parsed, &state, &expr_source);
    let frame = format_patch_signals(&data_name, &initial);
    if tx.send(Ok(frame)).await.is_err() {
        return;
    }

    let namespaces = subscribe_namespaces(&parsed);
    if namespaces.is_empty() {
        // No live updates available — initial value only, keep alive via hb task.
        return;
    }

    let actor_id = format!("sui-sse-{}-{}", data_name, gen_ulid());
    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: namespaces,
        see_directed_events: false,
    };
    let mailbox = match state.kernel().lock().await.spawn_actor(&actor_id, config) {
        Ok((_store, mb)) => Some(mb),
        Err(e) => {
            tracing::error!(error = %e, actor = %actor_id, "sui_sse spawn_actor failed");
            None
        }
    };
    let _guard = crate::ship_runtime::ActorGuard {
        actor_id,
        kernel: state.kernel().clone(),
    };
    let Some(mut mb) = mailbox else { return };

    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            ev = mb.recv() => {
                let Some(_ev) = ev else { break };
                // Coalesce burst events before re-resolving.
                while mb.try_recv().is_ok() {}

                let value = resolve_value(&parsed, &state).unwrap_or(serde_json::Value::Null);
                let live_value = serde_json::json!({"kind": "ok", "value": value});
                let frame = format_patch_signals(&data_name, &live_value);
                if tx.send(Ok(frame)).await.is_err() {
                    break;
                }
            }
        }
    }
}

/// Resolve the initial value, wrapping in `{"kind":"ok","value":...}`.
/// Falls back to `{"kind":"loading"}` on error so the runtime never sees
/// a transport-level failure for an authoring mistake.
fn resolve_initial(
    parsed: &super::sui_data_resolvers::ParsedExpr,
    state: &ApiState,
    expr: &str,
) -> serde_json::Value {
    match resolve_value(parsed, state) {
        Ok(v) => serde_json::json!({"kind": "ok", "value": v}),
        Err(e) => {
            tracing::warn!(error = %e, expr = %expr, "sui_sse: initial resolve failed");
            serde_json::json!({"kind": "loading"})
        }
    }
}

/// Returns true when the expression maps to a signal-backed source.
/// Signal-backed sources maintain a shared `Signal<T>` singleton updated by a
/// single background task; SSE connections subscribe to the signal rather than
/// spawning their own kernel actors.
pub(super) fn is_signal_backed_expr(expr: &str) -> bool {
    // Strip "ship." prefix and "()" suffix to get the registry name.
    expr.trim_start_matches("ship.")
        .trim_end_matches("()")
        .trim_end_matches(')')
        .starts_with(git_signal::REGISTRY_NAME)
}

/// Drive a signal-backed live data source. Sends the initial value, then
/// pushes a `patch-signals` frame whenever the underlying `Signal` changes.
///
/// Runs alongside `run_data_source` for non-converted sources; both emit
/// identical `patch-signals` frames and the consumer cannot distinguish them.
pub(super) async fn run_signal_data_source(
    data_name: String,
    _expr_source: String,
    state: ApiState,
    tx: mpsc::Sender<Result<String, std::io::Error>>,
    cancel: CancellationToken,
) {
    // Ensure the singleton background updater is alive.
    git_signal::ensure_updater(state.clone()).await;

    // Resolve the initial value and seed the signal.
    git_signal::refresh(&state);

    let sig = git_signal::git_overview_signal();
    let initial = sig.peek();
    let frame = format_patch_signals(&data_name, &serde_json::json!({"kind": "ok", "value": initial}));
    if tx.send(Ok(frame)).await.is_err() {
        return;
    }

    // Bridge signal updates to this async task via mpsc.
    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::channel::<()>(4);
    let _disposer = sui_signals::effect({
        let sig = sig.clone();
        let notify_tx = notify_tx.clone();
        move || {
            let _ = sig.get(); // establishes tracking
            let _ = notify_tx.try_send(());
        }
    });
    // Drain the initial notification fired during effect construction.
    let _ = notify_rx.try_recv();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            recv = notify_rx.recv() => {
                let Some(()) = recv else { break };
                // Coalesce rapid updates.
                while notify_rx.try_recv().is_ok() {}

                let value = sig.peek();
                let frame = format_patch_signals(&data_name, &serde_json::json!({"kind": "ok", "value": value}));
                if tx.send(Ok(frame)).await.is_err() {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_signal_backed_matches_git_overview() {
        assert!(is_signal_backed_expr("ship.git.overview()"));
        assert!(!is_signal_backed_expr("ship.git.log()"));
        assert!(!is_signal_backed_expr("ship.agents.summaries()"));
    }

    /// Verify that writing to the git.overview signal propagates through the
    /// effect → mpsc bridge, which is the mechanism run_signal_data_source
    /// uses to push SSE frames.
    #[tokio::test]
    async fn signal_update_notifies_bridge() {
        use sui_signals::{Signal, effect};
        use tokio::sync::mpsc;

        let sig: Signal<serde_json::Value> = Signal::new(serde_json::Value::Null);
        let (notify_tx, mut notify_rx) = mpsc::channel::<()>(4);

        let _disposer = effect({
            let sig = sig.clone();
            let tx = notify_tx.clone();
            move || {
                let _ = sig.get();
                let _ = tx.try_send(());
            }
        });
        // Drain initial notification from effect construction.
        let _ = notify_rx.try_recv();

        // Writing to the signal should trigger the bridge.
        sig.set_unchecked(serde_json::json!({"branch": "main", "clean": true}));

        let result = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            notify_rx.recv(),
        )
        .await;

        assert!(result.is_ok(), "signal update did not reach bridge within 100ms");
    }
}
