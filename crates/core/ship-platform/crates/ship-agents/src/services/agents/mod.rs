//! Agents domain — method-oriented service trait + state + view discovery.
//!
//! HTTP handlers binding axum to this trait live in
//! `apps/shipd/src/services/agents/mod.rs`.
//!
//! The coding-class counterpart (`ship.code.v1.CodeService`) lives
//! alongside the in-process surface (`ship.ops.v1.AgentsService`) in
//! `code_dispatch_core` / `code_dispatch_request` / `code_wire` —
//! mount split, 2026-07-10.

pub mod annotation;
pub mod code_dispatch_core;
pub mod code_dispatch_request;
pub mod code_wire;
pub mod create;
pub mod dispatch_core;
pub mod dispatch_request;
#[cfg(feature = "ship-code")]
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

pub use code_dispatch_core::dispatch_code_session_core;
pub use code_dispatch_request::{CodeDispatchRequest, billing_warning_for_axes, validate_code_dispatch_request, validate_axes as validate_code_axes};
pub use code_wire::{CodeWire, CodeWireConfig};
pub use dispatch_core::{AgentDispatchPort, DispatchCoreError, DispatchOutcome, dispatch_agent_session_core};
#[cfg(feature = "ship-code")]
pub use dispatch_resume::{ResumeOutcome, ResumeResolveError, load_and_validate_session_file, resolve_resume};
pub use feedback::{FeedbackGuard, FeedbackError, ViewBroadcastPort, ViewBroadcastError, shared_guard, post_feedback};
pub use rpc_wire::AgentsWire;
pub use host::{AgentsGrantHost, AgentsHost, AgentsWatchHost, DispatchHandlePort};
