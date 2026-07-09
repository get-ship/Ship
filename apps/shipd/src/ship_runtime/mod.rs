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
// live_resolvers: the `(domain, op) -> live value` resolver table — built in
// `init`, held on `ShipRuntime`, exposed via `accessors::live_resolvers`.
pub mod live_resolvers;
mod tasks;
#[cfg(test)]
mod test_ctors;

// Re-export all public types from this module level so callers that use
// `crate::ship_runtime::*` continue to resolve without path changes.
pub use types::{
    RuntimeConfig, AppMetadata,
    ApiState,
    SharedKernel,
    ActorGuard,
};

use std::path::PathBuf;
use std::sync::Arc;

use crate::auth::SessionRegistry;
use ship_substrate::task_health::HealthRegistry;
#[cfg(feature = "ship-code")]
use crate::dispatch::run_dispatch::DispatchHandle;

/// Central runtime for shipd. Owns the kernel, registry, and config.
pub struct ShipRuntime {
    pub(super) config: RuntimeConfig,
    pub(super) kernel: SharedKernel,
    pub(super) ship_dir: PathBuf,
    /// `<root>/machine/daemons/<hash>/` per devx-04.
    pub(super) daemon_dir: PathBuf,
    #[cfg(feature = "ship-code")]
    pub(super) dispatch_handle: std::sync::OnceLock<DispatchHandle>,
    /// Scoped session registry — maps session tokens to DaemonSession.
    pub(super) session_registry: SessionRegistry,
    /// Health registry for supervised background tasks.
    pub(super) health_registry: HealthRegistry,
    /// Active-space `SpaceContext`, bound to the active space's per-scope
    /// `spaces/<id>/space.db`. Both `collections()` and `artifacts()` ride the
    /// SAME ctx — they share the same per-scope db, the same `KernelLog`, and
    /// the same broadcast channel (one-way-to-do-things; collapsing the
    /// previous dual `DataService` connections). `None` when the daemon
    /// booted space-less; both accessors then error cleanly rather than
    /// fabricate a `global` partition (S2 home-space kill).
    pub(super) space_ctx: Option<Arc<ship_substrate::ports::SpaceContext>>,
    /// The ONE shared collection schema registry — hydrated from the file
    /// overlay at startup and shared across the collection wire, triggers,
    /// and the internal `collections()` accessor so a wire-created schema is
    /// visible everywhere (C-2 UD-7).
    pub(super) schemas: Arc<ship_collection::projection::SchemaRegistry>,
    /// Blob root for artifacts (global ship dir).
    pub(super) global_blob_dir: std::path::PathBuf,
    // (The agent-owned `view_sessions` canvas registry was deleted: the
    // UI purge removed every surface that could create (`view.open`) or
    // watch (SSE subscribe) a session, leaving a permanently-empty map.
    // The ViewService Patch/Append/Signal ops answer "not found" directly.)
    /// PTY session registry — chat, pipelines, dock terminal all go
    /// through here. Backed by the `ship_terminal` L1 crate.
    pub(super) pty_sessions: ship_terminal::SessionRegistry,
    /// Actor-backed session registry — destination of the supervisor-
    /// primitive migration. Shells live here today; agents and pipelines
    /// follow as 1.1.c / 1.1.d land.
    pub(super) actor_sessions: ship_agents::services::sessions::ActorRegistry,
    /// The space this daemon is currently serving — its mint-once
    /// `space_id` (an owned string, NOT a `SpaceId` newtype). Cached at boot
    /// from `ship_space::identity::ensure_identity_for_root`; never re-minted,
    /// never re-registered, never `resolve_active_space`-recomputed. `None`
    /// when the daemon is space-less (no `--workspace`, or no project root
    /// it can attach to). Every reader that needs "the served space"
    /// — `active_space_id`, `SpaceService.Info`, the dispatch chain's
    /// `spaces/<id>/space.db` partition — reaches for this.
    pub(super) served_space_id: std::sync::OnceLock<Option<String>>,
    /// Project workspace dir (e.g. `/path/to/repo`) when the daemon was
    /// started with `--workspace`. Drives project-scoped artifact storage.
    pub(super) workspace_dir: Option<PathBuf>,
    /// Wall-clock instant the daemon process started — used for uptime.
    pub(super) started_at: std::time::Instant,
    /// descriptor at boot — no dedicated file-watcher post format-migration
    /// Wave 4 phase A).
    pub(super) hook_engine: std::sync::OnceLock<std::sync::Arc<ship_triggers::engine::HookEngine>>,
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
    /// The live agent-def index (project scope), retained so the profile READ
    /// handlers serve list/detail/spawn from the DataStore snapshot instead of
    /// re-reading `.ship/agents/*.jsonc`. Set by `start_agent_def_index` (a
    /// clone of the Arc the change consumer owns); `None` until the index is
    /// mounted (space-less boot never mounts it).
    pub(super) agent_def_index:
        std::sync::OnceLock<std::sync::Arc<ship_platform::config::AgentDefIndex>>,
    /// Reactive registry for the unified agent `Watch` (agents-arc E5). Built
    /// from the SAME shared `agent_def_kernel` the live def-index rides — N
    /// subscribers on one `WatchKey` share one resolver + one kernel
    /// subscription. `None` until `start_agent_def_index` mounts the shared
    /// kernel (space-less boot never mounts it; the Watch RPC then errors
    /// cleanly). Reuses the existing shared log rather than standing up a
    /// parallel registry (one-way-to-do-things; mirrors collection/space).
    pub(super) agent_reactive:
        std::sync::OnceLock<std::sync::Arc<ship_reactive::ReactiveRegistry>>,
    /// Reactive registry for the typed pipeline `Watch` (pipelines P0.4). Built
    /// from a dedicated `KernelLog` fed by a bridge actor that taps the kernel
    /// router's `pipeline.*` + `session.*` mailbox stream (the orchestrator emits
    /// through the raw `KernelRouter`, not a shared log) — so the Watch resolvers
    /// wake on run/step lifecycle. `None` until `start_pipeline_reactive` runs.
    pub(super) pipeline_reactive:
        std::sync::OnceLock<std::sync::Arc<ship_reactive::ReactiveRegistry>>,
    /// The shared `KernelLog` the pipeline Watch registry rides — held so the
    /// bridge task can `broadcast()` tapped router events into it.
    pub(super) pipeline_watch_kernel:
        std::sync::OnceLock<std::sync::Arc<ship_substrate::ports::KernelLog>>,
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
    pub(super) rules: std::sync::Arc<dyn ship_agents::services::rules::service::RulesService>,
    /// `PermissionsService` impl — second mr-02 pilot, includes the
    /// cross-domain `tool_patterns` composition.
    pub(super) permissions: std::sync::Arc<dyn ship_agents::services::permissions::service::PermissionsService>,
    /// `McpService` impl — third mr-02 pilot.
    pub(super) mcp: std::sync::Arc<dyn ship_agents::services::mcp::service::McpService>,
    /// `SkillsService` impl — fourth mr-02 migration.
    pub(super) skills: std::sync::Arc<dyn ship_agents::services::skills::service::SkillsService>,
    /// `AgentsService` impl — fifth mr-02 migration.
    pub(super) agents: std::sync::Arc<dyn ship_agents::services::agents::service::AgentsService>,
    /// `PipelinesService` impl — sixth mr-02 migration (definition CRUD only;
    /// dispatch / cancel / runs still flow through `runtime_api`).
    pub(super) pipelines: std::sync::Arc<dyn ship_pipeline_runtime::pipelines::service::PipelinesService>,
    // The dedicated `git` service field is gone — git is an L0 git2 wrapper
    // (`ship_git::{read,write,diff,state}`), not an RPC service. Views read
    // git through the abstract registry op (`live_resolvers` `git.overview`).
    /// Shutdown cancellation token (#256). A child of the daemon's signal
    /// token, set once in `run_network` right after that token is created.
    /// Long-lived SSE keepalive/data loops `select!` against this so a SIGTERM
    /// closes them promptly instead of holding the axum graceful-shutdown drain
    /// open until the client happens to disconnect (the ~10s SIGTERM→SIGKILL
    /// hang). Cleared/cancelled again in `shutdown()` as a belt-and-suspenders.
    pub(super) shutdown_token: std::sync::OnceLock<tokio_util::sync::CancellationToken>,
    /// The live-read resolver registry — SSOT for `(domain, op) -> value` in the
    /// snapshot family. Both the per-op `<Op>Live` streams (via `live_read_with`)
    /// and the multiplex resolve through this one table.
    /// (The per-window multiplex registry was deleted in the transport move to
    /// deck-axum (#276); it now lives in `deck_core::window_registry` and is
    /// constructed at router-mount time in `apps/shipd/src/router.rs`.)
    pub(super) live_resolvers: std::sync::Arc<crate::ship_runtime::live_resolvers::LiveResolverRegistry>,
}
