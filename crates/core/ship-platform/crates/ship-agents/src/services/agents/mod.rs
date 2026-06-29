//! Agents domain — method-oriented service trait + state + view discovery.
//!
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/agents/mod.rs`.

pub mod agents_jsonc;
pub mod annotation;
pub mod create;
pub mod dispatch_core;
pub mod dispatch_request;
pub mod dispatch_resume;
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
pub mod host;
pub mod wire_types;

pub use annotation::{CollectionPersistPort, CollectionError, post_annotation};
pub use dispatch_core::{AgentDispatchPort, DispatchCoreError, DispatchOutcome, dispatch_agent_session_core};
pub use dispatch_request::{DispatchRequest, billing_warning_for_axes, default_invocation, default_keepalive, default_open_tab, validate_axes};
pub use dispatch_resume::{ResumeOutcome, ResumeResolveError, load_and_validate_session_file, resolve_resume};
pub use feedback::{FeedbackGuard, FeedbackError, ViewBroadcastPort, ViewBroadcastError, shared_guard, post_feedback};
pub use rpc_wire::AgentsWire;
pub use host::{AgentsGrantHost, AgentsHost, AgentsWatchHost, DispatchHandlePort};
