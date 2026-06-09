//! OS-level process lifecycle primitives (L0).
//!
//! After the supervisor split this crate owns only the OS-level pieces that
//! carry no L1 dependency: terminal-tab launching and the process-event emit
//! re-exports. The agent-run *assembly* (harness selection, isolation/sandbox
//! dispatch, token minting, executors, git/tmux worktree helpers) moved up into
//! `ship-agents` (`ship_agents::runtime`), and the daemon-side composition root
//! (`run_dispatch`, `ShipdPlatform`, `GitWorktreeIsolation`, orphan cleanup)
//! moved into `apps/shipd` (`shipd::dispatch`) — both because that assembly
//! depends on L1 crates this L0 primitive must not name.
//!
//! The fork/exec/PTY/signal/wait mechanics themselves live one level down in
//! `ship_substrate::supervisor` (`spawn_actor`, `ActorHandle`); this crate is
//! the thin OS-facing surface above it.

pub mod process_events;
pub mod terminal_launcher;
