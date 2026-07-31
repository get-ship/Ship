//! Job + agent-session dispatch composition root (daemon-side).
//!
//! Host-side wiring for the L1 dispatch ports injected into the agent-session
//! actor (`ship-agents`):
//!
//! - `run_dispatch` — `subscribe_job_events`: builds the concrete executor /
//!   isolation / platform impls and the `AgentPorts`, then starts the actor.
//! - `ports` — `ConfigSpawnPolicy` + `ProjectionsReader` (composition-only).
//! - `platform` — `ShipdPlatform` (the `AgentPlatform` impl) + `MockPlatform`.
//! - `adapters` — `GitWorktreeIsolation` (the `IsolationStrategy` impl).
//! - `orphan_cleanup` — startup worktree reaping for terminal jobs.
//!
//! Lives here (not in `ship-agents`) to avoid a dependency cycle: these modules
//! wire bridge/executors that `ship-agents` only names as ports.

pub mod adapters;
pub mod orphan_cleanup;
pub mod platform;
pub mod ports;
pub mod run_dispatch;
