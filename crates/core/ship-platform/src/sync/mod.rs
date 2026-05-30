//! Sync client — pushes local events to cloud, pulls remote events down.
//!
//! Push/pull cursors live in `ship_substrate::db::cursors` (the `cursors`
//! table in platform.db, alongside per-projection rebuild cursors).

pub mod client;
pub mod types;

pub use client::SyncClient;
pub use types::{PullResponse, PushRequest, PushResponse};

#[cfg(test)]
mod tests;
