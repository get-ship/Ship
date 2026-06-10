//! Floki (L1) — Ship's first-party, embedded, code-mode agent runtime.
//!
//! Floki is the runtime where **we own the loop**. That ownership is the whole
//! point: because the loop is ours, every effect an agent causes enters at the
//! one door (`gateway::Gateway`) where the gate runs; code-mode JS executes in a
//! locked-down V8 isolate ([`ship_js`]) whose only effect is the same gated wire;
//! and a code-mode op stream CAN be journaled and replayed without re-running
//! effects ([`durable::run_durable_script`], Stage 1) — but the live loop is
//! not yet journal-driven and the model turn is not durable (tokens re-burn on
//! resume); that is Stage 2, tracked in floki-Stage-2 (#30). A
//! third-party harness (claude, codex — `ship-code`'s territory) cannot offer
//! any of that, because we don't own its loop.
//!
//! The agent definition is read from [`agent_def`] — the ONE keystone both
//! runtimes share. Floki projects an [`agent_def::Agent`]'s `capabilities` into
//! the gate's grant list and runs the model turn against it.
//!
//! ## The shape of a run
//! [`agent_loop::run_step`] drives one model turn (rig is the commodity loop; we
//! do not reinvent it). Our two tools make every effect land gated:
//! - `ship_op` — a single atomic op through [`gateway::Gateway`] (the gate +
//!   the spine's post-exec verify + the `Ask` approval handshake).
//! - `run_script` — **code-mode**: agent-authored JS, executed on ship-js's
//!   `!Send` VM-thread via the [`dispatch::GatewayDispatch`] adapter, importing
//!   only the scoped SDK ([`sdkgen`]) for granted services. The gate still
//!   enforces — the SDK is the soft layer, the gate is the hard one.
//!
//! ## The seam to the L0 engines
//! ship-js and durable-execution both speak one contract:
//! [`ship_js::Dispatch`] — `call(op, args) -> Result<Value, DispatchError>`,
//! carrying no auth. Floki's rich op door ([`gateway::Gateway`], with
//! `Denied`/`Ask`/call-kinds) is adapted to it by [`dispatch::GatewayDispatch`],
//! which captures `(gateway, auth)` so the credential never crosses the seam.

pub mod agent_loop;
pub mod approver;
pub mod auth;
pub mod dispatch;
pub mod durable;
pub mod dtsgen;
pub mod gate;
pub mod gateway;
pub mod lsp;
pub mod provider;
pub mod registry;
pub mod resilience;
pub mod sdkgen;
pub mod session;
pub mod spine;

// NOTE: orchestration (the recursive spawn-agent seam) is intentionally NOT
// graduated here. The spike's `OrchestrationDispatch` trait is shaped around a
// mock-turn `ChildBehavior` test fake; the real seam is "spawn-agent = recursive
// Run", which belongs to the control-plane verb design (D5). Graduating the
// test-shaped trait now would bake in an API D5 must immediately break.

pub use agent_def::{
    Agent, SessionCancelled, SessionCompleted, SessionEventSink, SessionFailed,
    SessionStarted, SessionSuspended, TurnEnded, TurnStarted,
};
pub use agent_loop::{AgentOutput, Approver, Cancel, HeadlessApprover, Steer, ToolCtx, run_step};
pub use ship_js::deno::ScriptExecutor;
pub use approver::UserInput;
pub use auth::AuthContext;
pub use dispatch::GatewayDispatch;
pub use durable::run_durable_script;
pub use gateway::{Gateway, GatewayCallKind, GatewayError, InProcessGateway};
pub use provider::{BuiltModel, Credential, Provider, build_model};
pub use resilience::{RetryPolicy, run_resilient};
pub use session::{HistoryStore, InMemoryHistoryStore, Session, Turn};
pub use spine::SpineError;
