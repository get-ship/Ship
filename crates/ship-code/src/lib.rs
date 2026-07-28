//! Third-party harness run-assembly — Bridge/fleet product surface.
//!
//! Turns a `SessionContext` into a running harness process (Process / Tmux /
//! Server / Routing). Hosts (shipd, helmd) inject `LifecycleSink`, isolation,
//! and auth; those crates stay in Ship — this crate path-deps them only.
//!
//! Home: `bridge/crates/ship-code`. Not Ship monorepo.

pub mod agent_lifecycle;
pub mod dispatch_ports;
pub mod executors;
pub mod helpers;
pub mod process_registry;
pub mod process_tree_probe;
pub mod lifecycle;

pub use executors::{pty_actor_handle, pty_executor_handle_for_test};
pub use helpers::worktrees_dir;
pub use lifecycle::{LifecycleSink, NoopLifecycleSink};
