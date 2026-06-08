//! The live-read resolver registry — the single source of truth for "what does
//! `(domain, op, args)` resolve to" in the snapshot-emission family.
//!
//! ONE `(domain, op) -> LiveResolver` table, consumed by BOTH live transports:
//! - the per-op `<Op>Live` Connect streams, via `OpsImpl::live_read_with`;
//! - the live-data multiplex (`WindowSubscriptionService`), via `resolve_fn`.
//!
//! This replaces the scattered per-domain dispatch (git_signal, the inlined
//! `pipelines_live_fn` loop, the dead `sui_data_resolvers::resolve`). An op is
//! declared live in exactly one place; both transports derive their coverage
//! from it, so the multiplex never needs a hand-kept second copy.
//!
//! Scope: the **snapshot** family — each resolver returns the FULL current value
//! for `(state, args)`, re-run on each `config` kernel event. Patch-diff `Watch`
//! domains (large list diffs via `ReactiveRegistry`) are a different stream
//! contract and keep their own registry; they are not folded in here.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use crate::ship_runtime::ApiState;
use crate::window_subscriptions::ResolveFn;

/// Resolve `(state, args) -> full current value`. Args-aware so a single entry
/// serves parameterized reads (e.g. `pipelines.run({slug})`).
pub type LiveResolver = Arc<dyn Fn(&ApiState, &Value) -> Value + Send + Sync>;

/// The `(domain, op) -> resolver` table. Built once at startup, held in
/// `ApiState`. Keyed by `"domain.op"`.
#[derive(Default)]
pub struct LiveResolverRegistry {
    map: HashMap<String, LiveResolver>,
}

impl LiveResolverRegistry {
    fn register(&mut self, domain: &str, op: &str, resolver: LiveResolver) {
        self.map.insert(format!("{domain}.{op}"), resolver);
    }

    /// The raw args-aware resolver for `(domain, op)`, if registered.
    pub fn lookup(&self, domain: &str, op: &str) -> Option<&LiveResolver> {
        self.map.get(&format!("{domain}.{op}"))
    }

    /// Bind `args` and return the args-free `ResolveFn` both transports want
    /// (the multiplex's `Subscription.resolve` and `live_read_with`). `None`
    /// when the op is not in the snapshot registry — callers surface that as an
    /// explicit error rather than a silent empty value.
    pub fn resolve_fn(&self, domain: &str, op: &str, args: &Value) -> Option<ResolveFn> {
        let resolver = self.lookup(domain, op)?.clone();
        let args = args.clone();
        Some(Arc::new(move |state: &ApiState| resolver(state, &args)))
    }

    /// Whether `(domain, op)` has a snapshot resolver.
    pub fn has(&self, domain: &str, op: &str) -> bool {
        self.map.contains_key(&format!("{domain}.{op}"))
    }
}

/// Build the registry with every snapshot resolver reachable from shipd today.
/// Each entry calls the SAME builder its per-op `<Op>Live` stream calls — reuse,
/// not a re-implemented copy.
///
/// `account.state` is intentionally NOT registered: the wire's
/// `AccountService::state` is async (a future `StateLive` rpc needs a sync
/// facade first), and inventing a sync read here would be new resolution
/// logic. `dashboard.state` / `mesh.snapshot` are ghost ops with no builder
/// at all. See `.artifacts/live-data-coverage.md` for the full coverage
/// table + GAP list.
pub fn build() -> LiveResolverRegistry {
    let mut reg = LiveResolverRegistry::default();

    // git.overview — reuses the git_signal resolver (GitWire under the hood).
    reg.register(
        "git",
        "overview",
        Arc::new(|state: &ApiState, _args: &Value| {
            crate::views::git_signal::refresh(state);
            crate::views::git_signal::git_overview_signal().peek()
        }),
    );

    // pipelines.* — reuses the shipd pipeline view builders (the same ones the
    // per-op pipelines live streams drive).
    reg.register(
        "pipelines",
        "summaries",
        Arc::new(|state: &ApiState, _args: &Value| {
            use crate::services::pipelines::summaries;
            use crate::views::state::Filter;
            let ctx = summaries::build(state.ship_dir(), Filter::default(), None);
            serde_json::to_value(&ctx).unwrap_or(Value::Null)
        }),
    );
    reg.register(
        "pipelines",
        "run",
        Arc::new(|state: &ApiState, args: &Value| {
            use crate::services::pipelines::run_detail;
            let slug = args.get("slug").and_then(|v| v.as_str()).unwrap_or("");
            let ctx = run_detail::build(slug, state.ship_dir());
            serde_json::to_value(&ctx).unwrap_or(Value::Null)
        }),
    );
    reg.register(
        "pipelines",
        "focus",
        Arc::new(|state: &ApiState, args: &Value| {
            use crate::services::pipelines::focus;
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            // The unary read returns `null` when no pipeline file resolves
            // to `name` — the wire's `live_read_with` translates that into
            // a `not_found` ConnectError. The live stream then yields a
            // single null frame, which the SDK surfaces as an error
            // envelope to the controller.
            match focus::build(name, state.ship_dir()) {
                Some(ctx) => serde_json::to_value(&ctx).unwrap_or(Value::Null),
                None => Value::Null,
            }
        }),
    );
    reg.register(
        "pipelines",
        "history",
        Arc::new(|state: &ApiState, args: &Value| {
            use crate::services::pipelines::history;
            let q = args.get("q").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string);
            let pipeline = args.get("pipeline").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string);
            let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
            let ctx = history::build(state.ship_dir(), q, pipeline, offset, limit);
            serde_json::to_value(&ctx).unwrap_or(Value::Null)
        }),
    );
    // agents.summaries — reuses the agents-wire `Summaries` builder. The
    // wire's `AgentsService::Summaries` rpc calls the same `list_agents`
    // helper; the multiplex's re-resolution loop runs the same call. The
    // view consumes `AgentsSummariesResponse { items: [...] }` (the proto
    // wire shape), so we wrap the list under `items`.
    reg.register(
        "agents",
        "summaries",
        Arc::new(|state: &ApiState, _args: &Value| {
            let items = crate::views::state::list_agents(state.ship_dir());
            serde_json::to_value(serde_json::json!({ "items": items })).unwrap_or(Value::Null)
        }),
    );

    // collections.list — reuses the collections-wire `List` builder. The
    // wire's `CollectionsService::List` rpc calls the same `SchemaRegistry::list`
    // and maps each `SchemaDoc` to a proto `CollectionSummary`; the multiplex
    // re-resolution loop runs the same map. `state.collections()` errors
    // cleanly when the daemon is space-less (S2 home-space kill) — surface
    // that as `null` so the SDK renders an `error` envelope rather than a
    // fabricated `global` partition. The view consumes `{ items: [...] }`
    // per the SDK's `ShipSDK.collections.list` contract.
    reg.register(
        "collections",
        "list",
        Arc::new(|state: &ApiState, _args: &Value| {
            let Ok(svc) = state.collections() else { return Value::Null };
            let summaries: Vec<Value> = svc
                .list()
                .into_iter()
                .map(|s| {
                    serde_json::json!({
                        "id": s.id,
                        "title": s.title.unwrap_or_default(),
                        // The wire's `List` handler leaves this at 0; counts
                        // are surfaced on the per-collection detail read.
                        "item_count": 0u32,
                        "system": s.system,
                    })
                })
                .collect();
            serde_json::json!({ "items": summaries })
        }),
    );

    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_ops_resolve_and_unknown_is_none() {
        let reg = build();
        assert!(reg.has("git", "overview"));
        assert!(reg.has("pipelines", "summaries"));
        assert!(reg.has("pipelines", "run"));
        assert!(reg.has("pipelines", "focus"));
        assert!(reg.has("agents", "summaries"), "agents.summaries registered (job/livedata-coverage)");
        assert!(reg.has("collections", "list"), "collections.list registered (job/livedata-coverage)");
        assert!(!reg.has("nonsense", "void"), "unregistered op has no resolver");
        assert!(
            reg.resolve_fn("nonsense", "void", &Value::Null).is_none(),
            "resolve_fn returns None for an unregistered op (caller errors, not silent)"
        );
        assert!(
            reg.resolve_fn("pipelines", "run", &serde_json::json!({"slug": "x"})).is_some(),
            "a registered parameterized op binds its args"
        );
        assert!(
            reg.resolve_fn("pipelines", "focus", &serde_json::json!({"name": "smoke"})).is_some(),
            "pipelines.focus binds its name arg"
        );
    }
}
