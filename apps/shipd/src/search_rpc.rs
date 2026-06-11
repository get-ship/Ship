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
//!   that opted out (`search.global = false`), build a `SearchContext` bound to
//!   each remaining space (`SpaceContext::for_space`), and hand them to
//!   `ship_search::run_global`, which runs the SAME scope-blind service per
//!   space, stamps each hit's `space_id`, merges, and re-ranks.

use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceResult};
use ship_ops_rpc::{OwnedSearchRequestView, SearchResponse as ProtoSearchResponse};
use ship_search::{
    hit_to_proto, parse_request, run_global, SearchScope, SearchService,
    SpaceTarget,
};
use ship_space::service::SpaceService;
use ship_substrate::ports::{DataStore, KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

use crate::search_context::ShipdSearchContext;
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

        let hits = match parsed.scope {
            SearchScope::ThisSpace => {
                let ctx = ShipdSearchContext::new(self.state.clone());
                self.service.run(&ctx, &parsed.query, parsed.opts).await.hits
            }
            SearchScope::Global => {
                self.run_global_search(&parsed.query, parsed.opts).await?
            }
        };

        let proto_hits = hits.into_iter().map(hit_to_proto).collect();
        Ok(Response::new(ProtoSearchResponse {
            hits: proto_hits,
            next_cursor: String::new(),
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
    ) -> Result<Vec<ship_search::SearchHit>, connectrpc::ConnectError> {
        let spaces = self.list_spaces().await?;
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

        let hits = run_global(&self.service, targets, query, opts, |space_id| {
            !opted_out.contains(&space_id.to_string())
        })
        .await;
        Ok(hits)
    }

    /// Live space ids from the Runtime-scoped registry (`SpaceService.List`).
    async fn list_spaces(
        &self,
    ) -> Result<Vec<String>, connectrpc::ConnectError> {
        let svc = SpaceService::new(self.runtime_ctx()?);
        Ok(svc
            .list()
            .await
            .map(|records| records.into_iter().map(|r| r.id).collect())
            .unwrap_or_default())
    }

    /// Subset of `spaces` whose effective `search.global` setting is `false`.
    /// Default (unset) is included, so only an explicit `false` opts out.
    /// Per-space db failures are non-fatal — a broken space is skipped (it
    /// contributes nothing to the opt-out set rather than poisoning the merge).
    async fn opted_out_spaces(&self, spaces: &[String]) -> std::collections::HashSet<String> {
        let svc = match self.runtime_ctx() {
            Ok(ctx) => SpaceService::new(ctx),
            Err(_) => return std::collections::HashSet::new(),
        };
        // settings.state op removed with the string-dispatch apparatus; the
        // settings overlay is read directly by the crate-owned SettingsWire.
        let global_settings = serde_json::Value::Null;

        let mut out = std::collections::HashSet::new();
        for space_id in spaces {
            let store = match self.space_store(space_id) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let Ok(info) = svc
                .info(&*store, Some(space_id), global_settings.clone())
                .await
            else {
                continue;
            };
            let included = info
                .settings
                .get("search.global")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            if !included {
                out.insert(space_id.clone());
            }
        }
        out
    }

    /// Runtime-scoped `SpaceContext` (registry reads + kernel handle).
    fn runtime_ctx(&self) -> Result<Arc<SpaceContext>, connectrpc::ConnectError> {
        let store = self.runtime_store()?;
        let kernel = Arc::new(KernelLog::new(self.state.kernel().clone()));
        Ok(Arc::new(SpaceContext::new(SpaceId::new(RUNTIME_PARTITION), store, kernel)))
    }

    fn runtime_store(&self) -> Result<Arc<dyn DataStore>, connectrpc::ConnectError> {
        use ship_substrate::data::DataService;
        let runtime_db = ship_substrate::db::runtime_db_path(self.state.ship_dir());
        let svc = DataService::open_at(&runtime_db)
            .map_err(|e| connectrpc::ConnectError::internal(format!("data service over runtime.db: {e}")))?;
        Ok(Arc::new(SingleBackendDataStore::new(svc, RUNTIME_PARTITION)))
    }

    fn space_store(&self, space_id: &str) -> Result<Arc<dyn DataStore>, connectrpc::ConnectError> {
        use ship_substrate::data::DataService;
        let space_db = ship_substrate::db::space_db_path(self.state.ship_dir(), space_id);
        let svc = DataService::open_at(&space_db)
            .map_err(|e| connectrpc::ConnectError::internal(format!("data service over space.db: {e}")))?;
        Ok(Arc::new(SingleBackendDataStore::new(svc, space_id)))
    }
}
