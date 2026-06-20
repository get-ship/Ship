//! Thin transport for entity search (`search.proto`).
//!
//! The search composition + ranking + the global merge all live in the
//! `ship-search` crate (scope-blind by design). shipd owns ONLY what the crate
//! cannot: resolving the per-request scope and, for a GLOBAL search, the set of
//! reachable spaces + their per-space `SearchContext` + the opt-out gate.
//!
//! - `scope = THIS_SPACE` (or `UNSPECIFIED`): run the daemon's active-space
//!   context — byte-identical to the historical single-context path.
//! - `scope = GLOBAL`: enumerate live spaces via `SpaceService.List`, skip any
//!   that opted out (`search.global = false` on the unified
//!   `settings_overlay`), build a `SearchContext` bound to each remaining
//!   space (`SpaceContext::for_space`), and hand them to
//!   `ship_search::run_global`, which runs the SAME scope-blind service per
//!   space, stamps each hit's `space_id`, merges, and re-ranks.
//!
//! B1: the opt-out is read from the unified `settings_overlay` per space
//! (the legacy `SpaceService.info`/`Value::Null` path that read the
//! `space_overlay` table is gone). `search.global` is the only key; unset
//! (or non-bool) is included, only an explicit `false` opts out.

use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceResult};
use serde_json::Value;
use ship_ops_rpc::{OwnedSearchRequestView, SearchResponse as ProtoSearchResponse};
use ship_search::{
    hit_to_proto, parse_request, run_global, SearchScope, SearchService,
    SpaceTarget,
};
use ship_substrate::ports::{DataStore, KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

use crate::rpc::search_context::ShipdSearchContext;
use crate::ship_runtime::ApiState;

/// The fixed partition key for the Runtime-scoped space registry store
/// (`space_registry` / `space_active` live in runtime.db, machine-wide).
const RUNTIME_PARTITION: &str = "runtime";

/// shipd's search wire. Branches on the request scope; delegates all
/// composition + ranking to `ship-search`.
pub struct ShipdSearchWire {
    service: SearchService,
    state: ApiState,
}

impl ShipdSearchWire {
    pub fn new(state: ApiState) -> Self {
        Self { service: SearchService::new(), state }
    }
}

#[allow(refining_impl_trait)]
impl ship_ops_rpc::SearchService for ShipdSearchWire {
    async fn search(
        &self,
        _ctx: RequestContext,
        request: OwnedSearchRequestView,
    ) -> ServiceResult<ProtoSearchResponse> {
        let parsed = parse_request(&request);

        // B6 — forward the aggregator's cursor output. The proto request cursor
        // already decodes into `parsed.opts.cursor`; this branch was dropping
        // the response cursor. ThisSpace has access to the full
        // `SearchResponse { hits, cursor }` and can forward it directly. The
        // GLOBAL path delegates to `run_global_search`, whose surface still
        // returns only `Vec<SearchHit>` today — see TODO at the call site.
        let (hits, next_cursor) = match parsed.scope {
            SearchScope::ThisSpace => {
                let ctx = ShipdSearchContext::new(self.state.clone());
                let resp = self.service.run(&ctx, &parsed.query, parsed.opts).await;
                (resp.hits, resp.cursor.unwrap_or_default())
            }
            SearchScope::Global => {
                // TODO: `run_global_search` discards the per-space aggregator
                // cursor — its return type is `Vec<SearchHit>`. Once the
                // global fan-out exposes a merged `SearchResponse` (i.e.
                // `run_global` surfaces `resp.cursor`), forward it here in
                // place of the empty string. Do not fabricate a cursor.
                let global_hits = self.run_global_search(&parsed.query, parsed.opts).await;
                (global_hits, String::new())
            }
        };

        let proto_hits = hits.into_iter().map(hit_to_proto).collect();
        Ok(Response::new(ProtoSearchResponse {
            hits: proto_hits,
            next_cursor,
            ..Default::default()
        }))
    }
}

impl ShipdSearchWire {
    /// Fan out across reachable, opted-in spaces and merge.
    async fn run_global_search(
        &self,
        query: &str,
        opts: ship_search::SearchOpts,
    ) -> Vec<ship_search::SearchHit> {
        let spaces = self.list_spaces().await;
        let opted_out = self.opted_out_spaces(&spaces).await;

        let ship_dir = self.state.ship_dir().to_path_buf();
        let targets: Vec<SpaceTarget<ShipdSearchContext>> = spaces
            .into_iter()
            .map(|space_id| {
                let state = self.state.clone();
                let dir = ship_dir.clone();
                let sid = space_id.clone();
                SpaceTarget::new(space_id, move || {
                    // Bind this space's own physical db. If for_space fails the
                    // context degrades to empty snapshots (no fabricated rows),
                    // so a broken space contributes nothing rather than poison
                    // the merge.
                    match SpaceContext::for_space(&dir, SpaceId::new(sid.clone())) {
                        Ok(space_ctx) => {
                            ShipdSearchContext::for_space(state.clone(), Arc::new(space_ctx))
                        }
                        Err(_) => ShipdSearchContext::new(state.clone()),
                    }
                })
            })
            .collect();

        run_global(&self.service, targets, query, opts, |space_id| {
            !opted_out.contains(space_id)
        })
        .await
    }

    /// Live space ids from the Runtime-scoped registry (`SpaceService.List`).
    async fn list_spaces(&self) -> Vec<String> {
        use ship_space::service::SpaceService;
        let svc = SpaceService::new(self.runtime_ctx());
        svc.list()
            .await
            .map(|records| records.into_iter().map(|r| r.id).collect())
            .unwrap_or_default()
    }

    /// Subset of `spaces` whose effective `search.global` setting is `false`.
    /// Default (unset, non-bool) is included; only an explicit `false` opts out.
    /// Reads from the unified `settings_overlay` table (B1: replaced the
    /// `SpaceService.info`/`Value::Null`/`space_overlay` path).
    async fn opted_out_spaces(&self, spaces: &[String]) -> std::collections::HashSet<String> {
        let mut out = std::collections::HashSet::new();
        for space_id in spaces {
            let store = self.space_store(space_id);
            // B1: read `search.global` from the unified `settings_overlay`
            // (B1: replaced the `SpaceService.info`/`Value::Null`/`space_overlay`
            // path). Unset / non-bool ⇒ included; only an explicit `false` opts
            // out. A read error is conservative: include the space (don't opt
            // it out) — the fan-out gate's default is "include".
            let value = ship_config::overlay::get(&*store, "search.global").await.ok().flatten();
            let included = match value {
                Some(Value::Bool(b)) => b,
                _ => true,
            };
            if !included {
                out.insert(space_id.clone());
            }
        }
        out
    }

    /// Runtime-scoped `SpaceContext` (registry reads + kernel handle).
    fn runtime_ctx(&self) -> Arc<SpaceContext> {
        let store = self.runtime_store();
        let kernel = Arc::new(KernelLog::new(self.state.kernel().clone()));
        Arc::new(SpaceContext::new(SpaceId::new(RUNTIME_PARTITION), store, kernel))
    }

    fn runtime_store(&self) -> Arc<dyn DataStore> {
        use ship_substrate::data::DataService;
        let runtime_db = ship_substrate::db::runtime_db_path(self.state.ship_dir());
        let svc = DataService::open_at(&runtime_db).expect("data service over runtime.db");
        Arc::new(SingleBackendDataStore::new(svc, RUNTIME_PARTITION))
    }

    fn space_store(&self, space_id: &str) -> Arc<dyn DataStore> {
        use ship_substrate::data::DataService;
        let space_db = ship_substrate::db::space_db_path(self.state.ship_dir(), space_id);
        let svc = DataService::open_at(&space_db).expect("data service over space.db");
        Arc::new(SingleBackendDataStore::new(svc, space_id))
    }
}
