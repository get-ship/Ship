//! The live-read resolver registry — the single source of truth for "what does
//! `(domain, op, args)` resolve to" in the snapshot-emission family.
//!
//! ONE `(domain, op) -> LiveResolver` table, consumed by BOTH live transports:
//! - the per-op `<Op>Live` Connect streams, via `OpsImpl::live_read_with`;
//! - the live-data multiplex (`WindowSubscriptionService`), via `resolve_fn`.
//!
//! This replaces the scattered per-domain dispatch (git_signal, the
//! now-deleted `pipelines_live_fn` loop, the long-gone
//! `sui_data_resolvers::resolve`). An op is
//!
//! Scope: the **snapshot** family — each resolver returns the FULL current value
//! for `(state, args)`, re-run on each `config` kernel event. Patch-diff `Watch`
//! domains (large list diffs via `ReactiveRegistry`) are a different stream
//! contract and keep their own registry; they are not folded in here.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use ship_account::types::AccountToken;
use ship_substrate::config_doc;

use crate::ship_runtime::ApiState;

/// Resolve a subscription's current full value from runtime state. The same
/// closure shape `live_read::live_read_with` takes; both transports (per-op
/// `<Op>Live` streams AND the live-data multiplex) feed off this one alias.
/// Lives here — the producer of `resolve_fn` — so the multiplex's registry
/// in `deck_core` can depend on the SHAPE only (`Arc<dyn Fn(&ApiState) -> Value>`)
/// without owning the alias itself.
pub type ResolveFn = Arc<dyn Fn(&ApiState) -> Value + Send + Sync>;
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
/// `account.state` was unregistered: the wire's `AccountService::state` is
/// async (a future `StateLive` rpc needs a sync facade first), and inventing
/// a sync read here would be new resolution logic. The sync facade landed
/// in this pass — the resolver below reads the daemon-owned token file
/// (`<daemon_dir>/account.jsonc`) the same way the wire does, projects the
/// tagged `AccountState` enum, and lets the existing
/// `sse('account', 'state')` TS call resolve. `dashboard.state` /
/// `mesh.snapshot` remain ghost ops with no builder at all. See
/// `.artifacts/live-data-coverage.md` for the full coverage table.
pub fn build() -> LiveResolverRegistry {
    let mut reg = LiveResolverRegistry::default();

    // git.overview — the prototype for "domain data behind the abstract
    // registry op". Git is an L0 git2 wrapper (`ship_git`), not an RPC
    // service; the registry composes the `GitOverview` payload from its read
    // fns directly. The view subscribes `git.overview` through the multiplex;
    // there is no GitService on the wire.
    reg.register(
        "git",
        "overview",
        Arc::new(|state: &ApiState, _args: &Value| {
            let Some(root) = state.workspace_dir() else { return Value::Null };
            let buckets = ship_git::read::load_status_buckets(root);
            let branch = ship_git::read::load_branch(root);
            let commits = ship_git::read::load_log(root, None, 20);
            let branches = ship_git::read::load_branches(root);
            let (ahead, behind) = ship_git::read::load_ahead_behind(root).unwrap_or((0, 0));
            let overview = ship_git::state::GitOverview {
                workspace_id: None,
                branch,
                head_short: ship_git::read::load_head_short(root),
                clean: buckets.clean,
                total_changed: buckets.total,
                staged: buckets.staged,
                modified: buckets.modified,
                untracked: buckets.untracked,
                commits,
                branches,
                ahead,
                behind,
            };
            serde_json::to_value(&overview).unwrap_or(Value::Null)
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
            let items =
                ship_agents::services::agents::view_state::list_agents(state.ship_dir());
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

    // account.state — sync snapshot of the typed `AccountState` enum
    // (mirrors `crates/core/ship-platform/crates/ship-account/src/types.rs`).
    // The wire's `AccountService::state` is async; the live-data
    // multiplex re-resolution loop runs sync, so we re-read the
    // daemon-owned token file (`<daemon_dir>/account.jsonc`) directly
    // via the substrate secret-doc seam. Same value the wire returns
    // to Connect callers, just derived from the on-disk token. Pending
    // is intentionally NOT tracked here — the in-memory pending-login
    // registry lives only inside the wire's `AccountService` and
    // disappears on daemon restart (per the service's contract); a
    // restarted daemon reads logged_out until the CLI's `ship login`
    // callback completes and a fresh token lands on disk.
    reg.register(
        "account",
        "state",
        Arc::new(|state: &ApiState, _args: &Value| {
            let token_path = state.daemon_dir().join("account.jsonc");
            let token: Option<AccountToken> = config_doc::read_doc(&token_path)
                .ok()
                .flatten();
            match token {
                None => serde_json::json!({ "state": "logged_out" }),
                Some(t) if t.is_expired() => serde_json::json!({ "state": "logged_out" }),
                Some(t) => serde_json::json!({
                    "state": "logged_in",
                    "user_id": t.user_id,
                }),
            }
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
        assert!(reg.has("agents", "summaries"), "agents.summaries registered (job/livedata-coverage)");
        assert!(reg.has("collections", "list"), "collections.list registered (job/livedata-coverage)");
        assert!(reg.has("account", "state"), "account.state registered (job/phase3b-rewires)");
        assert!(!reg.has("pipelines", "summaries"), "pipelines live resolvers retired (job/livedata-collapse)");
        assert!(!reg.has("nonsense", "void"), "unregistered op has no resolver");
        assert!(
            reg.resolve_fn("nonsense", "void", &Value::Null).is_none(),
            "resolve_fn returns None for an unregistered op (caller errors, not silent)"
        );
    }
}
