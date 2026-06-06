//! Agents domain — method-oriented service trait + state + view discovery.
//!
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/agents/mod.rs`.

pub mod annotation;
pub mod create;
pub mod feedback;
pub mod rpc_wire;
pub mod service;
pub mod settings_view;
pub mod state;
pub mod view_state;
pub mod watch;
#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;
pub mod wire;
pub mod wire_types;

pub use annotation::{CollectionPersistPort, CollectionError, post_annotation};
pub use feedback::{FeedbackGuard, FeedbackError, ViewBroadcastPort, ViewBroadcastError, shared_guard, post_feedback};
pub use rpc_wire::AgentsWire;
