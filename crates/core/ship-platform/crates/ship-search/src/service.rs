//! `SearchService` — composes adapters from a `SearchContext` and runs the
//! aggregator. Ports + adapters: the service depends on the trait (port);
//! production wiring (shipd) and tests inject their own impls (adapters).
//!
//! ## Scope + auth
//!
//! `SearchContext` is **per-request**. Production callers (shipd) construct
//! a fresh context per Connect call with the caller's auth + space scope
//! baked in — snapshots returned by the trait MUST already be filtered to
//! what the caller is allowed to see. Search service treats snapshots as
//! truth and does not re-check permissions; that's the adapter's job.
//!
//! This shape means the same `SearchService::run()` produces different
//! results inside a space (scoped snapshots) vs from the hub (broader
//! snapshots) without the service knowing about scope at all.
//!
//! ## Query sanitization
//!
//! Sanitization (control-char strip, length cap, etc.) lives at the wire
//! boundary, NOT here. Search service trusts its input. Wire-side caller
//! must call `sanitize_query` (defined here for reuse) before passing the
//! query in.

use std::sync::Arc;

use async_trait::async_trait;

use crate::adapter::EntitySearchAdapter;
use crate::adapters::{
    agents::{AgentSnapshot, AgentsAdapter},
    artifacts::{ArtifactSnapshot, ArtifactsAdapter},
    collection_items::{CollectionItemSnapshot, CollectionItemsAdapter},
    collections::{CollectionSnapshot, CollectionsAdapter},
    sessions::{SessionSnapshot, SessionsAdapter},
    skills::{SkillSnapshot, SkillsAdapter},
    views::{ViewSnapshot, ViewsAdapter},
};
use crate::aggregator::{run_search, SearchResponse};
use crate::types::{EntityKindFilter, SearchOpts};

/// Per-request dependency port. Implementations MUST return snapshots
/// already filtered to the caller's authorization scope (e.g. the space
/// they're acting in). The search service does no further permission
/// checks — what the context returns IS what the caller may search.
#[async_trait]
pub trait SearchContext: Send + Sync {
    /// Sync sources — in-memory state (sessions, agent registry, etc.).
    fn views(&self) -> Vec<ViewSnapshot>;
    fn collections(&self) -> Vec<CollectionSnapshot>;
    fn agents(&self) -> Vec<AgentSnapshot>;
    fn skills(&self) -> Vec<SkillSnapshot>;
    fn sessions(&self) -> Vec<SessionSnapshot>;

    /// Async sources — backed by DB queries.
    async fn collection_items(&self) -> Vec<CollectionItemSnapshot>;
    async fn artifacts(&self) -> Vec<ArtifactSnapshot>;
}

/// Stateless search service. Construct once per workspace/process; pass
/// a fresh `SearchContext` per `run()` call.
pub struct SearchService;

impl SearchService {
    pub fn new() -> Self {
        Self
    }

    /// Run a search across every enabled adapter source.
    ///
    /// `opts.types` is the kind allowlist — when set, only those sources
    /// are loaded from the context (avoids unnecessary snapshot work).
    /// `query` is assumed pre-sanitized; see [`sanitize_query`].
    pub async fn run<C: SearchContext>(
        &self,
        ctx: &C,
        query: &str,
        opts: SearchOpts,
    ) -> SearchResponse {
        let enabled = |k: EntityKindFilter| -> bool {
            opts.types.as_deref().is_none_or(|list| list.contains(&k))
        };

        let mut adapters: Vec<Arc<dyn EntitySearchAdapter>> = Vec::new();

        if enabled(EntityKindFilter::View) {
            adapters.push(Arc::new(ViewsAdapter::new(ctx.views())));
        }
        if enabled(EntityKindFilter::Collection) {
            adapters.push(Arc::new(CollectionsAdapter::new(ctx.collections())));
        }
        if enabled(EntityKindFilter::CollectionItem) {
            adapters.push(Arc::new(CollectionItemsAdapter::new(ctx.collection_items().await)));
        }
        if enabled(EntityKindFilter::Agent) {
            adapters.push(Arc::new(AgentsAdapter::new(ctx.agents())));
        }
        if enabled(EntityKindFilter::Skill) {
            adapters.push(Arc::new(SkillsAdapter::new(ctx.skills())));
        }
        if enabled(EntityKindFilter::Session) {
            adapters.push(Arc::new(SessionsAdapter::new(ctx.sessions())));
        }
        if enabled(EntityKindFilter::Artifact) {
            adapters.push(Arc::new(ArtifactsAdapter::new(ctx.artifacts().await)));
        }

        run_search(&adapters, query, opts).await
    }
}

impl Default for SearchService {
    fn default() -> Self {
        Self::new()
    }
}

/// Sanitize a wire-side query string before passing to [`SearchService::run`].
///
/// - Strips control characters (0x00-0x1F + 0x7F)
/// - Caps at 256 chars
/// - Trims leading/trailing whitespace
pub fn sanitize_query(raw: &str) -> String {
    let mut s: String = raw
        .chars()
        .filter(|c| !c.is_control())
        .take(256)
        .collect();
    s = s.trim().to_string();
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_strips_control_chars() {
        assert_eq!(sanitize_query("hello\x00world\x1f"), "helloworld");
        assert_eq!(sanitize_query("\x7fnoise"), "noise");
    }

    #[test]
    fn sanitize_caps_length() {
        let long = "a".repeat(300);
        assert_eq!(sanitize_query(&long).len(), 256);
    }

    #[test]
    fn sanitize_trims_whitespace() {
        assert_eq!(sanitize_query("  hello  "), "hello");
    }
}
