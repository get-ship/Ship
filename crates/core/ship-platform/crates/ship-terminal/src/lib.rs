//! `ship-terminal` — Terminal domain (L1) — PTY session registry + shell spawn.
//!
//! Carved from `ship-agents::services::sessions` (Phase 2 of the
//! launch-unified-HEAD unification). Owns the PTY session domain so
//! the agent-domain crate is no longer overloaded with terminal work.
//!
//! The PTY session registry is a parallel system being retired
//! kind-by-kind (per `actor_registry`'s comment). Today it still holds
//! per-turn agent sessions (codex / opencode) and shells (via
//! `shell_actor::spawn_shell`). The destination registry
//! (`ship-agents::services::sessions::actor_registry::ActorRegistry`)
//! is where long-lived agent sessions live; that's the path Phase 3
//! routes the wire through.
//!
//! ## Modules
//! - `types`        — `SessionId`, `SessionKind`, `SessionStatus`,
//!                    `SessionMeta`, `SpawnRequest`, `Inner`,
//!                    `SessionHandle`, `PtySession`.
//! - `registry`     — in-memory `SessionRegistry` (HashMap of
//!                    `SessionId -> SessionHandle`).
//! - `shell_actor`  — `spawn_shell` / `stop` — shell spawn through
//!                    the supervisor primitive, inserted into the
//!                    registry above.

pub mod registry;
pub mod shell_actor;
pub mod types;

pub use registry::SessionRegistry;
pub use shell_actor::{spawn_shell, stop as stop_shell};
pub use types::{
    Inner, PtySession, SessionHandle, SessionId, SessionKind, SessionMeta, SessionStatus,
    SpawnRequest,
};
