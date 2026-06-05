//! diagnostics.snapshot — aggregated in-process observability op.
//!
//! Returns a JSON tree with `process_rss_bytes` and one subtree per
//! registered subsystem. Each subsystem implements
//! [`ship_substrate::diagnostics::DiagnosticSnapshot`]. The registry is
//! constructed in [`build_registry`] and embedded in `ApiState` via
//! [`DiagRegistry`] which callers pass to the handler.
//!
//! Subsystems instrumented here (9 total, see spec):
//!   1. kernel_router       — actors, event rate (impl in ship-substrate)
//!   2. supervisor          — dispatched agents + per-turn actor sessions
//!   3. session_registry    — per-turn PTY sessions
//!   4. actor_registry      — actor-backed sessions (shells, agents)
//!   5. mesh_registry       — registered mesh agents
//!   6. health_tasks        — supervised background tasks
//!   7. pipelines_service   — pipeline definitions
//!   8. config_watcher      — file-watcher state (stub; no public count API)
//!   9. rpc_connections     — active TCP connection count (stub; axum gives no hook)
//!
//! The spec lists CollectionRegistry and ProjectionCache as targets.
//! CollectionRegistry count is embedded in the `pipelines_service` subtree.
//! ProjectionCache has no in-process size hook (all queries hit SQLite); it
//! is registered as a named stub so the diff can show "no change" cleanly.

use std::sync::Arc;

use serde_json::{Value, json};
use ship_substrate::diagnostics::{DiagnosticRegistry, DiagnosticSnapshot};

use crate::ship_runtime::ApiState;

// ── Subsystem snapshot types ───────────────────────────────────────────────

/// Supervisor / process-registry snapshot: live dispatched agents.
///
/// Uses a boxed closure to read the count so the type `ProcessRegistry`
/// (pub(crate) in ship-supervisor) doesn't need to be named here.
pub struct SupervisorDiag {
    count_fn: Arc<dyn Fn() -> usize + Send + Sync>,
}

impl SupervisorDiag {
    /// Build from the dispatch handle if present, or return a zero-count stub.
    pub fn from_dispatch_handle(
        handle: Option<&crate::supervisor::run_dispatch::DispatchHandle>,
    ) -> Self {
        match handle {
            Some(dh) => {
                let registry = dh.registry.clone();
                Self {
                    count_fn: Arc::new(move || {
                        tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current()
                                .block_on(async { registry.lock().await.len() })
                        })
                    }),
                }
            }
            None => Self { count_fn: Arc::new(|| 0) },
        }
    }
}

impl DiagnosticSnapshot for SupervisorDiag {
    fn name(&self) -> &'static str { "supervisor" }

    fn snapshot(&self) -> Value {
        let count = (self.count_fn)();
        let bytes_estimated = count * 512;
        json!({
            "bytes_estimated": bytes_estimated,
            "count_dispatched_agents": count,
        })
    }
}

/// Per-turn PTY session registry snapshot.
pub struct PtySessionsDiag {
    registry: crate::sessions::SessionRegistry,
}

impl PtySessionsDiag {
    pub fn new(registry: crate::sessions::SessionRegistry) -> Self {
        Self { registry }
    }
}

impl DiagnosticSnapshot for PtySessionsDiag {
    fn name(&self) -> &'static str { "session_registry" }

    fn snapshot(&self) -> Value {
        let sessions = self.registry.list();
        let count = sessions.len();
        let bytes_estimated = count * 1024;
        json!({
            "bytes_estimated": bytes_estimated,
            "count_sessions": count,
        })
    }
}

/// Actor-backed session registry snapshot.
pub struct ActorSessionsDiag {
    registry: crate::sessions::ActorRegistry,
}

impl ActorSessionsDiag {
    pub fn new(registry: crate::sessions::ActorRegistry) -> Self {
        Self { registry }
    }
}

impl DiagnosticSnapshot for ActorSessionsDiag {
    fn name(&self) -> &'static str { "actor_registry" }

    fn snapshot(&self) -> Value {
        let entries = self.registry.list();
        let count = entries.len();
        let bytes_estimated = count * 512;
        json!({
            "bytes_estimated": bytes_estimated,
            "count_actor_sessions": count,
        })
    }
}

/// Mesh registry snapshot.
pub struct MeshRegistryDiag {
    registry: ship_substrate::services::mesh::SharedMeshRegistry,
}

impl MeshRegistryDiag {
    pub fn new(registry: ship_substrate::services::mesh::SharedMeshRegistry) -> Self {
        Self { registry }
    }
}

impl DiagnosticSnapshot for MeshRegistryDiag {
    fn name(&self) -> &'static str { "mesh_registry" }

    fn snapshot(&self) -> Value {
        let count = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.registry.read().await.len()
            })
        });
        let bytes_estimated = count * 256;
        json!({
            "bytes_estimated": bytes_estimated,
            "count_mesh_agents": count,
        })
    }
}

/// Health task registry snapshot.
pub struct HealthTasksDiag {
    registry: ship_substrate::task_health::HealthRegistry,
}

impl HealthTasksDiag {
    pub fn new(registry: ship_substrate::task_health::HealthRegistry) -> Self {
        Self { registry }
    }
}

impl DiagnosticSnapshot for HealthTasksDiag {
    fn name(&self) -> &'static str { "health_tasks" }

    fn snapshot(&self) -> Value {
        let tasks = self.registry.snapshot();
        let total = tasks.len();
        let alive = tasks.iter().filter(|t| t.alive).count();
        let bytes_estimated = total * 128;
        json!({
            "bytes_estimated": bytes_estimated,
            "count_tasks": total,
            "count_alive": alive,
            "tasks": tasks,
        })
    }
}

/// Pipelines service snapshot — counts registered pipeline definitions.
pub struct PipelinesDiag {
    service: Arc<dyn ship_pipeline_runtime::pipelines::service::PipelinesService>,
}

impl PipelinesDiag {
    pub fn new(
        service: Arc<dyn ship_pipeline_runtime::pipelines::service::PipelinesService>,
    ) -> Self {
        Self { service }
    }
}

impl DiagnosticSnapshot for PipelinesDiag {
    fn name(&self) -> &'static str { "pipelines_service" }

    fn snapshot(&self) -> Value {
        let count = self.service.list().len();
        let bytes_estimated = count * 512;
        json!({
            "bytes_estimated": bytes_estimated,
            "count_pipeline_definitions": count,
        })
    }
}

/// Config watcher stub — the underlying notify watcher has no public count
/// of pending events. Registered so diffs show "no change" for this slot
/// rather than omitting it.
pub struct ConfigWatcherDiag;

impl DiagnosticSnapshot for ConfigWatcherDiag {
    fn name(&self) -> &'static str { "config_watcher" }
    fn snapshot(&self) -> Value {
        // NOTE: ship_platform::config::WatcherService (ConfigWatcher alias) holds
        // a notify::RecommendedWatcher internally. That watcher's pending-event
        // queue is not exposed via a public API; a future change to the watcher
        // struct can add a counter and wire it here.
        json!({
            "bytes_estimated": 0,
            "note": "pending event count not available from notify watcher API",
        })
    }
}

/// RPC / TCP connection count stub — axum's tower layers do not expose a
/// live connection counter without an explicit middleware. Registered as a
/// named stub so the slot is present for diffing.
pub struct RpcServerDiag;

impl DiagnosticSnapshot for RpcServerDiag {
    fn name(&self) -> &'static str { "rpc_server" }
    fn snapshot(&self) -> Value {
        // NOTE: axum::serve does not expose a live connection count. Adding
        // ConcurrencyLimitLayer or a custom middleware that increments an
        // AtomicUsize on each accept would fix this. Filed for follow-up.
        json!({
            "bytes_estimated": 0,
            "note": "live connection count not instrumented; requires accept middleware",
        })
    }
}

/// ProjectionCache stub — all projection reads hit SQLite; there is no
/// in-process cache with a measurable byte footprint today.
pub struct ProjectionCacheDiag;

impl DiagnosticSnapshot for ProjectionCacheDiag {
    fn name(&self) -> &'static str { "projection_cache" }
    fn snapshot(&self) -> Value {
        // NOTE: ship_substrate projections are fully SQLite-backed.
        // There is no in-process cache today. If a Bloom-filter or LRU
        // projection cache is added, wire its byte_size() here.
        json!({
            "bytes_estimated": 0,
            "note": "no in-process cache; queries go directly to platform.db",
        })
    }
}

// ── Registry builder ───────────────────────────────────────────────────────

/// Build and populate the diagnostic registry from the live runtime.
///
/// Called once at daemon startup. The registry is then embedded in the
/// handler closure so `diagnostics.snapshot` can call `snapshot_all()`.
pub fn build_registry(state: &ApiState) -> DiagnosticRegistry {
    let reg = DiagnosticRegistry::new();

    // 1. KernelRouter — Arc-wraps the SharedKernel (already an Arc<Mutex<KernelRouter>>).
    use ship_substrate::events::kernel_router::KernelRouterDiag;
    reg.register(Arc::new(KernelRouterDiag::new(state.kernel().clone())));

    // 2. Supervisor / dispatch handle process registry.
    reg.register(Arc::new(SupervisorDiag::from_dispatch_handle(state.dispatch_handle())));

    // 3. Per-turn PTY session registry.
    reg.register(Arc::new(PtySessionsDiag::new(state.pty_sessions().clone())));

    // 4. Actor-backed session registry.
    reg.register(Arc::new(ActorSessionsDiag::new(state.actor_sessions().clone())));

    // 5. Mesh registry.
    reg.register(Arc::new(MeshRegistryDiag::new(state.mesh_registry().clone())));

    // 6. Supervised health tasks.
    reg.register(Arc::new(HealthTasksDiag::new(state.health_registry().clone())));

    // 7. Pipeline definitions.
    {
        let pipelines_arc: Arc<dyn ship_pipeline_runtime::pipelines::service::PipelinesService> =
            state.pipelines_arc();
        reg.register(Arc::new(PipelinesDiag::new(pipelines_arc)));
    }

    // 8. Config watcher stub.
    reg.register(Arc::new(ConfigWatcherDiag));

    // 9. RPC server stub.
    reg.register(Arc::new(RpcServerDiag));

    // 10. Projection cache stub.
    reg.register(Arc::new(ProjectionCacheDiag));

    reg
}

// ── Op handler ────────────────────────────────────────────────────────────

/// Handler for the `diagnostics.snapshot` op.
///
/// Aggregates all registered subsystem snapshots, prepends `ts_utc` and
/// `process_rss_bytes`, and returns the JSON tree.
pub fn handle_snapshot(registry: &DiagnosticRegistry) -> Value {
    let ts = chrono::Utc::now().to_rfc3339();
    let rss = ship_substrate::diagnostics::process_rss_bytes();

    let mut result = registry.snapshot_all();
    let obj = result.as_object_mut().expect("snapshot_all returns object");
    obj.insert("ts_utc".to_string(), json!(ts));
    if let Some(rss_bytes) = rss {
        obj.insert("process_rss_bytes".to_string(), json!(rss_bytes));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_watcher_diag_snapshot_is_valid_json() {
        let snap = ConfigWatcherDiag.snapshot();
        assert!(snap.is_object());
        assert!(snap["note"].is_string());
    }

    #[test]
    fn rpc_server_diag_snapshot_is_valid_json() {
        let snap = RpcServerDiag.snapshot();
        assert!(snap.is_object());
    }

    #[test]
    fn projection_cache_diag_snapshot_is_valid_json() {
        let snap = ProjectionCacheDiag.snapshot();
        assert!(snap.is_object());
    }

    #[test]
    fn handle_snapshot_adds_ts_and_rss_fields() {
        let reg = DiagnosticRegistry::new();
        reg.register(Arc::new(ConfigWatcherDiag));
        let snap = handle_snapshot(&reg);
        assert!(snap["ts_utc"].is_string());
        // process_rss_bytes is present on Linux, absent elsewhere — just check the key
        // is either a number or absent (not null / string).
        if let Some(rss) = snap.get("process_rss_bytes") {
            assert!(rss.is_number(), "process_rss_bytes must be a number");
        }
        assert!(snap["subsystems"]["config_watcher"].is_object());
    }
}
