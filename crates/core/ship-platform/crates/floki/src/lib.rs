//! Floki (L1) — Ship's first-party, embedded, code-mode agent runtime.
//!
//! Floki is the runtime where **we own the loop**. That ownership is the whole
//! point: because the loop is ours, every effect an agent causes enters at the
//! one door (`gateway::Gateway`) where the gate runs; code-mode JS executes in a
//! locked-down V8 isolate ([`ship_js`]) whose only effect is the same gated wire;
//! and a run is journaled so it can suspend and resume by replay
//! ([`durable_execution`]) without re-running effects or re-burning tokens. A
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
pub mod auth;
pub mod dispatch;
pub mod events;
pub mod gate;
pub mod gateway;
pub mod registry;
pub mod sdkgen;
pub mod spine;

pub use agent_def::Agent;
pub use agent_loop::{AgentOutput, Approver, Cancel, HeadlessApprover, Steer, ToolCtx, run_step};
pub use auth::AuthContext;
pub use dispatch::GatewayDispatch;
pub use events::{AgentEvent, EventSink, UserInput};
pub use gateway::{Gateway, GatewayCallKind, GatewayError, InProcessGateway};
pub use spine::SpineError;
