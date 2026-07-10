//! Agent-run assembly (L1) — the harness/isolation/token-aware machinery
//! that turns a `SessionContext` into a running harness process.
//!
//! Lifted out of the L0 `ship-supervisor` crate (the supervisor split): the
//! OS-level process primitive (`process_supervisor::spawn_process`,
//! PTY/signals/wait) stays in L0, while everything that *assembles* an agent
//! run — harness selection (`ship-harnesses`), isolation decision + sandbox
//! dispatch (`ship-isolation` / `ship-device`), per-process token minting
//! (`ship-auth`), spawn-env + stream parsing (`ship-files`) — lives here.
//!
//! `executors` holds the `SessionExecutor` implementations
//! (Process / TmuxWindow / Server / Routing). `helpers` holds the git /
//! tmux / worktree subprocess utilities and the worktrees-dir resolver.
//! `agent_lifecycle` holds graceful agent exit for pipeline step transitions.
//!
//! The composition root (`run_dispatch`) and the `AgentPlatform` /
//! `IsolationStrategy` host impls live in `apps/shipd` because they depend on
//! `ship-pipeline-runtime`, which depends on this crate — keeping them here
//! would close a dependency cycle.

pub mod agent_lifecycle;
pub mod dispatch_ports;
pub mod executors;
pub mod helpers;
pub mod process_registry;
pub mod process_tree_probe;
pub mod lifecycle;

pub use executors::{pty_actor_handle, pty_executor_handle_for_test};
pub use helpers::worktrees_dir;
