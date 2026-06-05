//! ShipRuntime — central manager for the shipd daemon.
//!
//! Replaces the ad-hoc `ApiState` struct with a structured runtime that
//! owns the kernel, config, and lazily-initialized managers.
//!
//! Submodules:
//! - `types`     — config types, type aliases, and small utility types
//! - `init`      — construction (`new`), shutdown, and test helpers
//! - `accessors` — accessor methods, background task starters, event routing

pub mod types;
pub(crate) mod init;
mod accessors;
mod tasks;
#[cfg(test)]
mod test_ctors;

// Re-export all public types from this module level so callers that use
// `crate::ship_runtime::*` continue to resolve without path changes.
pub use types::{
    AuthMode, RuntimeConfig, AppMetadata,
    AppState, ApiState,
    SharedKernel, AgentMailboxes, PtyConnections,
    WorktreeResolver, ActorGuard,
};
pub use init::default_worktree_resolver;

use std::path::PathBuf;
use std::sync::Arc;

use ship_platform::services::mesh::SharedMeshRegistry;
use crate::auth::SessionRegistry;
use ship_substrate::task_health::HealthRegistry;
use crate::supervisor::run_dispatch::DispatchHandle;

/// Central runtime for shipd. Owns the kernel, mesh registry, and config.
/// Managers (mesh, jobs) are lazily initialized on first access.
pub struct ShipRuntime {
    pub(super) config: RuntimeConfig,
    pub(super) kernel: SharedKernel,
    pub(super) mesh_registry: SharedMeshRegistry,
    pub(super) agent_mailboxes: AgentMailboxes,
    pub(super) pty_connections: PtyConnections,
    pub(super) ship_dir: PathBuf,
    /// Daemon IPC directory — `<root>/machine/daemon/` or
    /// `<root>/machine/daemons/<hash>/` per devx-04.
    pub(super) daemon_dir: PathBuf,
    pub(super) mesh: tokio::sync::OnceCell<crate::managers::MeshManager>,
    pub(super) jobs: tokio::sync::OnceCell<crate::managers::RunManager>,
    pub(super) dispatch_handle: std::sync::OnceLock<DispatchHandle>,
    pub(super) session_scanner: tokio::sync::OnceCell<crate::managers::SessionScanner>,
    /// Scoped session registry — maps session tokens to DaemonSession.
    pub(super) session_registry: SessionRegistry,
    /// Health registry for supervised background tasks.
    pub(super) health_registry: HealthRegistry,
    /// Diagnostic registry — aggregates per-subsystem `DiagnosticSnapshot`
    /// impls. Built lazily via `diag_registry()` / `set_diag_registry()`
    /// after the dispatch handle is set (supervisor snapshot depends on it).
    pub(super) diag_registry: std::sync::OnceLock<ship_substrate::diagnostics::DiagnosticRegistry>,
    /// Active-space `SpaceContext` for collections, bound to the active space's
    /// per-scope `spaces/<id>/space.db`. `None` when the daemon booted space-less
    /// (no discoverable project); the `collections()` accessor then errors cleanly
    /// rather than fabricate a `global` partition (S2 home-space kill).
    pub(super) collection_ctx: Option<Arc<ship_substrate::ports::SpaceContext>>,
    /// Active-space `SpaceContext` for artifacts. `None` in space-less mode — the
    /// `artifacts()` accessor errors cleanly.
    pub(super) artifact_ctx: Option<Arc<ship_substrate::ports::SpaceContext>>,
    /// The ONE shared collection schema registry — hydrated from the file
    /// overlay at startup and shared across the collection wire, triggers,
    /// and the internal `collections()` accessor so a wire-created schema is
    /// visible everywhere (C-2 UD-7).
    pub(super) schemas: Arc<ship_collection::projection::SchemaRegistry>,
    /// Blob root for artifacts (global ship dir).
    pub(super) global_blob_dir: std::path::PathBuf,
    /// Agent-owned view sessions (Phase 1 canvas framework).
    pub(super) view_sessions: crate::views::session::SharedViewSessionRegistry,
    /// PTY session registry — chat, pipelines, dock terminal all go
    /// through here. See `crate::sessions`.
    pub(super) pty_sessions: crate::sessions::SessionRegistry,
    /// Actor-backed session registry — destination of the supervisor-
    /// primitive migration. Shells live here today; agents and pipelines
    /// follow as 1.1.c / 1.1.d land.
    pub(super) actor_sessions: crate::sessions::ActorRegistry,
    /// Project workspace dir (e.g. `/path/to/repo`) when the daemon was
    /// started with `--workspace`. Drives project-scoped artifact storage.
    pub(super) workspace_dir: Option<PathBuf>,
    /// Wall-clock instant the daemon process started — used for uptime.
    pub(super) started_at: std::time::Instant,
    /// Hook engine (external source adapters + hot-reload watcher).
    pub(super) hook_engine: std::sync::OnceLock<std::sync::Arc<ship_platform::triggers::engine::HookEngine>>,
    /// Declarative-tier config watcher (Phase 4) — emits domain.changed events.
    pub(super) config_watcher: std::sync::OnceLock<ship_platform::config::ConfigWatcher>,
    /// Shared event-log handle for the live agent-def index (agents arc S5).
    /// Set by `start_agent_def_index`: the boot rebuild + the change consumer +
    /// the gated-write NOTIFY all ride THIS one `KernelLog` so the op-emit and
    /// the consumer's `subscribe()` share a broadcast (each `KernelLog::new`
    /// owns its own channel). `None` until the index is mounted (space-less boot
    /// never mounts it).
    pub(super) agent_def_kernel:
        std::sync::OnceLock<std::sync::Arc<ship_substrate::ports::KernelLog>>,
    /// Reactive registry for the unified agent `Watch` (agents-arc E5). Built
    /// from the SAME shared `agent_def_kernel` the live def-index rides — N
    /// subscribers on one `WatchKey` share one resolver + one kernel
    /// subscription. `None` until `start_agent_def_index` mounts the shared
    /// kernel (space-less boot never mounts it; the Watch RPC then errors
    /// cleanly). Reuses the existing shared log rather than standing up a
    /// parallel registry (one-way-to-do-things; mirrors collection/space).
    pub(super) agent_reactive:
        std::sync::OnceLock<std::sync::Arc<ship_reactive::ReactiveRegistry>>,
    /// In-process Rolldown bundler (#163) — owned here, lazily started on
    /// first access. `Bundler::start` is synchronous (it locates the
    /// `sui-compiler` binary and sets up channels, no child process), so
    /// `std::sync::OnceLock` is the right primitive — using
    /// `tokio::sync::OnceCell` would require `get_or_init` to take an
    /// `async` initializer.
    pub bundler: std::sync::OnceLock<deck::bundler::Bundler>,
    /// `RulesService` impl — pilot for the modular-runtime spec 02
    /// service-layer pattern. HTTP handlers in `services::rules` go
    /// through this trait so the cloud worker / browser embed / unit
    /// tests can swap implementations without standing up an HTTP server.
    pub(super) rules: std::sync::Arc<dyn ship_platform::services::rules::service::RulesService>,
    /// `PermissionsService` impl — second mr-02 pilot, includes the
    /// cross-domain `tool_patterns` composition.
    pub(super) permissions: std::sync::Arc<dyn ship_platform::services::permissions::service::PermissionsService>,
    /// `McpService` impl — third mr-02 pilot.
    pub(super) mcp: std::sync::Arc<dyn ship_platform::services::mcp::service::McpService>,
    /// `SkillsService` impl — fourth mr-02 migration.
    pub(super) skills: std::sync::Arc<dyn ship_platform::services::skills::service::SkillsService>,
    /// `AgentsService` impl — fifth mr-02 migration.
    pub(super) agents: std::sync::Arc<dyn ship_platform::services::agents::service::AgentsService>,
    /// `PipelinesService` impl — sixth mr-02 migration (definition CRUD only;
    /// dispatch / cancel / runs still flow through `runtime_api`).
    pub(super) pipelines: std::sync::Arc<dyn ship_pipeline_runtime::pipelines::service::PipelinesService>,
    /// `GitService` impl — seventh mr-02 migration. Project-bound; the
    /// SSE handler in `git/mod.rs` stays separate.
    pub(super) git: std::sync::Arc<dyn ship_platform::services::git::service::GitService>,
    /// Worktree resolver — production uses `git worktree list`; tests
    /// override with an in-memory map. See `resolve_worktree` method and
    /// `new_test_with_worktree_resolver` test ctor.
    pub(super) worktree_resolver: WorktreeResolver,
}
