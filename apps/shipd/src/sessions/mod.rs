//! PTY session registry — the one primitive that backs chat, pipelines,
//! and the dock's terminal tab. Each session is a child process running
//! under a real pseudo-terminal so providers (Claude, Codex, …) behave
//! as if they're in a real shell, plus arbitrary shells for the
//! terminal panel.
//!
//! # Layering
//!
//! * `types` — `SessionId`, `SessionKind`, `SessionStatus`, `PtySession`.
//! * `registry` — in-memory store. Thread-safe. One `Arc<SessionRegistry>`
//!   lives on `ApiState`; handlers and executors all share it.
//! * `lifecycle` — spawn + stop + stdin-write. Provider adapters compose
//!   the launch command; we own the PTY and the reader thread.
//!
//! # Why not reuse `ProcessExecutor`?
//!
//! `ProcessExecutor` is built for pipeline jobs — headless, one-shot,
//! autostart prompt injected. Chat/agent sessions are interactive, long-
//! lived, user-driven; the shape is different enough that sharing the
//! spawn path by copying the ~40 lines of portable-pty boilerplate is
//! cheaper than generalising the executor. The reader-to-event-parser
//! adapter trait is shared via `runtime::services::provider_adapter`.

pub mod types;
pub mod registry;
pub mod actor_registry;
pub mod shell_actor;
pub mod agent_actor;
pub mod lifecycle;
pub mod api;

pub use registry::SessionRegistry;
pub use actor_registry::{ActorEntry, ActorRegistry};
pub use types::{PtySession, SessionId, SessionKind, SessionStatus, SpawnRequest};
