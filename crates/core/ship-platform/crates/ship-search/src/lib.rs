//! Unified entity search — fans out across adapters (views, collections,
//! collection items, agents, skills, sessions, artifacts), merges and ranks
//! the hits, and returns a paginated mixed-kind result.
//!
//! Public surface:
//! - [`SearchService`] — entry point. Stateless; construct once per process.
//! - [`SearchContext`] — port for snapshot sources. Implement once per
//!   caller (production: shipd builds a fresh one per Connect request with
//!   the caller's scope baked in; tests: in-memory fixtures).
//! - [`sanitize_query`] — call before [`SearchService::run`].
//!
//! Scope + auth are the adapter's responsibility — snapshots returned by
//! `SearchContext` MUST already be filtered to what the caller may see.

pub mod adapter;
pub mod adapters;
pub mod aggregator;
pub mod ranking;
pub mod service;
pub mod types;

pub use adapter::{AdapterResult, EntitySearchAdapter};
pub use aggregator::{SearchResponse, run_search};
pub use service::{sanitize_query, SearchContext, SearchService};
pub use types::{Cursor, EntityKind, EntityKindFilter, SearchHit, SearchOpts};
