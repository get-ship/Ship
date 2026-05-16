//! Unified entity search — fans out across adapters (views, collections,
//! collection items, agents, skills, sessions, artifacts), merges and ranks
//! the hits, and returns a paginated mixed-kind result.
//!
//! The HTTP surface lives at `/api/v1/search` in `apps/shipd/src/routes/search_api.rs`.
//! Cmdk's "find" mode consumes it directly.

pub mod adapter;
pub mod adapters;
pub mod aggregator;
pub mod ranking;
pub mod types;

pub use adapter::{AdapterResult, EntitySearchAdapter};
pub use aggregator::{SearchResponse, run_search};
pub use types::{Cursor, EntityKind, EntityKindFilter, SearchHit, SearchOpts};
