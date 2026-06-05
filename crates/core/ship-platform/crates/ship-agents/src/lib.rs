//! `ship-agents` — agent-session orchestration (L1).
//!
//! Owns the bare `agent.session` dispatch primitive carved out of the historic
//! orchestrator (agents-arc S3b). The pipeline-DAG half (advance / sweep /
//! replay / fanout / gate / promote / merge) was carved out of `ship-platform`
//! along with `ship_platform::services::orchestrator` into the
//! `ship-pipeline-runtime` crate (P3 pipeline cut).
//!
//! ## What lives here
//!
//! - [`dispatch`] — `handle_agent_session_started`: prep / spawn / register /
//!   exit-handler-attach for bare single-agent sessions (`ship use`,
//!   `ship dispatch`, MCP `dispatch_agent`). The SOLE handler of
//!   `agent.session.started`.
//! - [`cancel`] — the agent-cancel decision: stop the harness via the L0
//!   teardown core, clean up, emit `process.killed`. The SOLE handler of
//!   `agent.session.cancelled`.
//! - [`actor`] — `start_agents`: the `service.agents` actor that runs the L0
//!   router gate then routes the two events above.
//! - [`ports`] — `AgentPorts`, the injected L0-port bundle.
//! - [`sessions`] — RUNTIME-scoped session records (#283): the runtime-session
//!   types + query/progress reads + hydration + on-disk artifacts, folded from
//!   the historic ship-platform sessions tree (agents-arc E4). L0-clean (names
//!   only `ship-substrate`); emits the `session.*` namespace, NOT
//!   `agent.session.*`. Distinct from agent DEFS (space/account config).
//!
//! `build_spawn_env` (the pure `SpawnPolicy`-port call) is L0 mechanism — it
//! names no agent-domain type, only the substrate principal + the L0 port — so
//! it lives in `ship_runner::build_spawn_env`, the one home both this crate's
//! dispatch primitive and the pipeline orchestrator call.
//!
//! ## Layering
//!
//! Depends on L0 ONLY: `ship-substrate` (kernel router, event types, the
//! domain-free dispatch ports), `ship-runner` (router gate, dedup, teardown,
//! exit handler, kernel-emit), `ship-types` (the `Authority` principal). It
//! drives the L0 runner mechanism and never reaches into `ship-platform`'s
//! `crate::config`, `crate::auth`, or `crate::projections`.
//!
//! Two concrete impls the agent path needs stay in L1 (ship-platform) and are
//! injected here through their L0 ports, exactly as the runner-ports keystone
//! intends:
//!   - `ConfigSpawnPolicy` (the [`SpawnPolicy`] impl) reads project config →
//!     stays in L1; agents sees only the `SpawnPolicy` port.
//!   - `ProjectionsReader` (the [`ProjectionReader`] impl) reads the SQL
//!     projections → stays in L1; agents sees only the `ProjectionReader` port.
//!
//! [`SpawnPolicy`]: ship_substrate::services::dispatch_ports::SpawnPolicy
//! [`ProjectionReader`]: ship_substrate::services::dispatch_ports::ProjectionReader

pub mod actor;
pub mod cancel;
pub mod dispatch;
pub mod ports;
// Runtime-scoped session records (agents-arc E4 fold) — the L0-clean analytics
// half of the historic ship-platform sessions tree. RUNTIME-scoped (#283),
// distinct from agent defs and from the agent.session.* dispatch primitive above.
pub mod sessions;

pub use actor::{handle_agents_event, start_agents};
pub use cancel::handle_agent_cancelled;
pub use dispatch::handle_agent_session_started;
pub use ports::AgentPorts;
