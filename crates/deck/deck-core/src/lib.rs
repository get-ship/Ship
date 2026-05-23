//! deck-core — transport-agnostic SUI hosting logic.
//!
//! This crate owns *what* deck serves (compiled views, bundles, the catalog,
//! pack assets) and produces a [`DeckResponse`] for each. It never touches
//! axum or any HTTP framework. Server adapters (`deck-axum`, a future
//! `deck-bun`) translate `DeckResponse` onto their own response type — the
//! ports-and-adapters seam.

pub mod response;

pub use response::{CachePolicy, DeckResponse, compute_etag};

/// Daemon operating mode — controls cache headers, HMR availability, SSR cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DaemonMode {
    Development,
    Production,
}

impl DaemonMode {
    pub fn parse(s: &str) -> Self {
        match s {
            "production" => Self::Production,
            _ => Self::Development,
        }
    }
}
