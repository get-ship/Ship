//! `EntitySearchAdapter` — the per-kind contract the aggregator fans out to.
//!
//! Implementations live in `super::adapters`. Each adapter owns its own
//! data source (a snapshot of view registry, a service handle, …) and is
//! responsible for case-insensitive substring matching + ranking.

use async_trait::async_trait;

use super::types::{EntityKindFilter, SearchHit, SearchOpts};

/// Result returned by a single adapter for one query. `next_offset` is
/// `Some(n)` when there are more results past the returned window;
/// `None` means the adapter exhausted its matches.
pub struct AdapterResult {
    pub hits: Vec<SearchHit>,
    pub next_offset: Option<usize>,
}

#[async_trait]
pub trait EntitySearchAdapter: Send + Sync {
    /// Stable token used as the per-adapter cursor key and as the
    /// `types=` query parameter for the wire-level filter.
    fn name(&self) -> &'static str;

    /// Which kind filter token enables this adapter. Cmdk's "find" mode
    /// either passes no filter (all adapters run) or passes a whitelist
    /// (only matching adapters run).
    fn kind_filter(&self) -> EntityKindFilter;

    /// Search for `q`. Implementations should:
    ///  - read `opts.cursor.get(self.name())` for their starting offset
    ///  - return at most `opts.limit_or_default()` hits
    ///  - set `next_offset` when more matches exist past the window
    async fn query(&self, q: &str, opts: &SearchOpts) -> AdapterResult;
}
