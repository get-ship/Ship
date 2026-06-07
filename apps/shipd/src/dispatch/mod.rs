//! Job + agent-session dispatch composition root (daemon-side).
//!
//! These modules were lifted out of the L0 `ship-supervisor` crate as part of
//! the supervisor split. They are the host-side wiring that the L1 dispatch
//! ports inject into the pipeline orchestrator (`ship-pipeline-runtime`) and
//! the agent-session actor (`ship-agents`):
//!
//! - `run_dispatch` — `subscribe_job_events`: builds the concrete executor /
//!   isolation / platform impls, the `OrchestratorContext`, and the
//!   `AgentPorts`, then starts both actors.
//! - `platform` — `ShipdPlatform` (the `AgentPlatform` impl) + `MockPlatform`.
//! - `adapters` — `GitWorktreeIsolation` (the `IsolationStrategy` impl).
//! - `orphan_cleanup` — startup worktree reaping for terminal jobs.
//!
//! They live here (not in `ship-agents`) because they depend on
//! `ship-pipeline-runtime`, which depends on `ship-agents` — placing them in
//! `ship-agents` would close a dependency cycle. shipd already depends on both,
//! so it is the natural composition layer.

pub mod adapters;
pub mod orphan_cleanup;
pub mod platform;
pub mod run_dispatch;
