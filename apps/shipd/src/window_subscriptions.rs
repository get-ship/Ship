//! Per-window live-data subscription registry — the server side of the
//! live-data multiplex (ADR 0001). One webview opens ONE `Stream` and adds/
//! removes logical reads via `Subscribe`/`Unsubscribe`; this registry keys the
//! per-window state on `window_id` so the single stream fans out to N reads.
//!
//! This module owns the registry data structures + lifecycle (open / subscribe
//! / unsubscribe / idle-reap). The `Stream` server-stream handler and the
//! Connect RPC impls live in `window_subscription_rpc.rs`; they drive ONE kernel
//! actor per window and call each `Subscription.resolve` on each kernel event,
//! emitting `MultiplexEvent.data` tagged by `subscription_id` — the same
//! resolve-on-event loop as `live_read::live_read_inner`, generalized from
//! one-actor-per-op to one-actor-per-window (reusing the per-op `services::*`
//! resolvers — see the resolver dispatch in the rpc module).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::ship_runtime::ApiState;

pub type WindowId = String;
pub type SubscriptionId = String;

/// Resolve a subscription's current full value from runtime state. The same
/// closure shape `live_read::live_read_with` takes; the multiplex reuses the
/// per-op `services::*` builders to construct these (rpc module).
pub type ResolveFn = Arc<dyn Fn(&ApiState) -> Value + Send + Sync>;

/// One logical live read inside a window.
pub struct Subscription {
    pub id: SubscriptionId,
    pub domain: String,
    pub op: String,
    pub args: Value,
    pub resolve: ResolveFn,
}

impl Subscription {
    pub fn key(&self) -> String {
        format!("{}|{}|{}", self.domain, self.op, self.args)
    }
}

/// Per-window state: the live subscriptions + liveness for the idle reaper.
pub struct WindowBucket {
    pub window_id: WindowId,
    pub session_id: String,
    subs: Mutex<HashMap<SubscriptionId, Subscription>>,
    /// ms epoch of the last activity (open / subscribe / stream send); the
    /// reaper drops a bucket whose last activity is older than the idle timeout.
    last_active_ms: AtomicU64,
}

impl WindowBucket {
    fn new(window_id: WindowId, session_id: String) -> Self {
        Self {
            window_id,
            session_id,
            subs: Mutex::new(HashMap::new()),
            last_active_ms: AtomicU64::new(now_ms()),
        }
    }

    pub fn touch(&self) {
        self.last_active_ms.store(now_ms(), Ordering::Relaxed);
    }

    pub fn last_active_ms(&self) -> u64 {
        self.last_active_ms.load(Ordering::Relaxed)
    }

    pub fn subscription_count(&self) -> usize {
        self.subs.lock().unwrap_or_else(|e| e.into_inner()).len()
    }

    pub fn insert_sub(&self, sub: Subscription) {
        self.subs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(sub.id.clone(), sub);
        self.touch();
    }

    pub fn remove_sub(&self, sub_id: &str) -> bool {
        let removed = self
            .subs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(sub_id)
            .is_some();
        if removed {
            self.touch();
        }
        removed
    }

    /// Snapshot the current subscriptions' (id, resolve) pairs for the stream
    /// bridge to re-resolve on a kernel event. Clones the `ResolveFn` Arcs.
    pub fn resolvers(&self) -> Vec<(SubscriptionId, ResolveFn)> {
        self.subs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .map(|s| (s.id.clone(), s.resolve.clone()))
            .collect()
    }
}

/// Registry of all open windows, keyed on `window_id`.
pub struct WindowSubscriptionRegistry {
    windows: Mutex<HashMap<WindowId, Arc<WindowBucket>>>,
    idle_timeout_ms: u64,
}

impl Default for WindowSubscriptionRegistry {
    fn default() -> Self {
        // 35s = the default 15s heartbeat × 2 + 5s slack (ADR §reaper).
        Self::with_idle_timeout(35_000)
    }
}

impl WindowSubscriptionRegistry {
    pub fn with_idle_timeout(idle_timeout_ms: u64) -> Self {
        Self { windows: Mutex::new(HashMap::new()), idle_timeout_ms }
    }

    /// Mint a fresh `win.<ulid>` and create its bucket bound to `session_id`.
    pub fn open(&self, session_id: String) -> WindowId {
        let window_id = format!("win.{}", ship_substrate::gen_ulid());
        let bucket = Arc::new(WindowBucket::new(window_id.clone(), session_id));
        self.windows
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(window_id.clone(), bucket);
        window_id
    }

    pub fn bucket(&self, window_id: &str) -> Option<Arc<WindowBucket>> {
        self.windows
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(window_id)
            .cloned()
    }

    /// Add a subscription to `window_id`; returns the new `sub.<ulid>` id, or
    /// `None` if the window is unknown (client must `Open` first).
    pub fn subscribe(
        &self,
        window_id: &str,
        domain: String,
        op: String,
        args: Value,
        resolve: ResolveFn,
    ) -> Option<SubscriptionId> {
        let bucket = self.bucket(window_id)?;
        let id = format!("sub.{}", ship_substrate::gen_ulid());
        bucket.insert_sub(Subscription { id: id.clone(), domain, op, args, resolve });
        Some(id)
    }

    pub fn unsubscribe(&self, window_id: &str, sub_id: &str) -> bool {
        self.bucket(window_id).map(|b| b.remove_sub(sub_id)).unwrap_or(false)
    }

    /// Drop `window_id` entirely (window closed / stream gone / reaped).
    pub fn drop_window(&self, window_id: &str) -> bool {
        self.windows
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(window_id)
            .is_some()
    }

    pub fn window_count(&self) -> usize {
        self.windows.lock().unwrap_or_else(|e| e.into_inner()).len()
    }

    /// Drop every bucket idle longer than the idle timeout. Returns the count
    /// dropped. Called on a periodic sweep + as a safety net behind the
    /// stream-drop GC.
    pub fn reap(&self) -> usize {
        let cutoff = now_ms().saturating_sub(self.idle_timeout_ms);
        let mut windows = self.windows.lock().unwrap_or_else(|e| e.into_inner());
        let before = windows.len();
        windows.retain(|_, b| b.last_active_ms() >= cutoff);
        before - windows.len()
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Build the resolver for a `(domain, op)` live read, reusing the **same**
/// builders the per-op `<Op>Live` Connect streams call — never a re-implemented
/// copy. `git.overview` reuses the `git_signal` resolver (GitWire); `pipelines.*`
/// reuse the shipd pipeline view builders (the same ones `pipelines_live_fn`
/// drives). Returns `None` for an op the multiplex doesn't yet route — the
/// `Subscribe` handler surfaces that as an explicit error rather than emit an
/// empty value, so the client keeps that op on its per-op stream (no silent
/// gap, no fabricated data).
///
/// Coverage grows by adding a match arm that calls the existing builder. The
/// follow-up that makes this exhaustive is unifying the per-op live resolvers
/// into one registry both the `<Op>Live` streams and this dispatch consume —
/// recorded against the live-data-multiplex plan.
pub fn resolver_for(domain: &str, op: &str, args: &Value) -> Option<ResolveFn> {
    match (domain, op) {
        ("git", "overview") => Some(Arc::new(|state: &ApiState| {
            crate::views::git_signal::refresh(state);
            crate::views::git_signal::git_overview_signal().peek()
        })),
        ("pipelines", "summaries") => Some(Arc::new(|state: &ApiState| {
            use crate::services::pipelines::summaries;
            use crate::views::state::Filter;
            let ctx = summaries::build(state.ship_dir(), Filter::default(), None);
            serde_json::to_value(&ctx).unwrap_or(Value::Null)
        })),
        ("pipelines", "run") => {
            let slug = args
                .get("slug")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Some(Arc::new(move |state: &ApiState| {
                use crate::services::pipelines::run_detail;
                let ctx = run_detail::build(&slug, state.ship_dir());
                serde_json::to_value(&ctx).unwrap_or(Value::Null)
            }))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noop_resolve() -> ResolveFn {
        Arc::new(|_: &ApiState| Value::Null)
    }

    #[test]
    fn open_mints_unique_window_ids() {
        let reg = WindowSubscriptionRegistry::default();
        let a = reg.open("sess-a".into());
        let b = reg.open("sess-b".into());
        assert_ne!(a, b, "two opens must mint distinct window ids");
        assert!(a.starts_with("win."), "window id is win.<ulid>; got {a}");
        assert_eq!(reg.window_count(), 2);
        assert_eq!(reg.bucket(&a).unwrap().session_id, "sess-a");
    }

    #[test]
    fn subscribe_then_unsubscribe_mutates_the_bucket() {
        let reg = WindowSubscriptionRegistry::default();
        let w = reg.open("s".into());
        let sub = reg
            .subscribe(&w, "agents".into(), "summaries".into(), Value::Null, noop_resolve())
            .expect("subscribe to a known window");
        assert!(sub.starts_with("sub."), "sub id is sub.<ulid>; got {sub}");
        assert_eq!(reg.bucket(&w).unwrap().subscription_count(), 1);
        assert!(reg.unsubscribe(&w, &sub), "unsubscribe an existing sub");
        assert_eq!(reg.bucket(&w).unwrap().subscription_count(), 0);
        assert!(!reg.unsubscribe(&w, &sub), "second unsubscribe is a no-op");
    }

    #[test]
    fn subscribe_to_unknown_window_is_none() {
        let reg = WindowSubscriptionRegistry::default();
        assert!(
            reg.subscribe("win.nope", "x".into(), "y".into(), Value::Null, noop_resolve())
                .is_none(),
            "subscribe to an unopened window must return None"
        );
    }

    #[test]
    fn reap_drops_idle_buckets_keeps_active() {
        // 0ms idle timeout → any bucket not touched "now" is reaped; a freshly
        // touched bucket survives. Asserts the behavior (idle dropped, active
        // kept), not a wall-clock — no flakiness.
        let reg = WindowSubscriptionRegistry::with_idle_timeout(0);
        let stale = reg.open("stale".into());
        // Force the stale bucket's last-active into the past.
        reg.bucket(&stale)
            .unwrap()
            .last_active_ms
            .store(now_ms().saturating_sub(10_000), Ordering::Relaxed);
        let fresh = reg.open("fresh".into());
        reg.bucket(&fresh).unwrap().touch();

        let dropped = reg.reap();
        assert_eq!(dropped, 1, "exactly the stale bucket is reaped");
        assert!(reg.bucket(&stale).is_none(), "stale window gone");
        assert!(reg.bucket(&fresh).is_some(), "fresh window kept");
    }
}
