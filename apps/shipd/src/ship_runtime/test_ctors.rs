//! Test constructors for ShipRuntime. Not compiled in production builds.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use ship_substrate::task_health::HealthRegistry;

use super::{ShipRuntime, RuntimeConfig, SharedKernel};

impl ShipRuntime {
    /// Build a ShipRuntime for tests — synchronous, takes a pre-initialized
    /// kernel and ship_dir. Does NOT run integrity checks or DB init.
    pub fn new_test(ship_dir: std::path::PathBuf, kernel: SharedKernel) -> Self {
        let workspace_dir = ship_dir.parent().map(|p| p.to_path_buf());
        let daemon_dir = ship_substrate::ShipHome::at(ship_dir.clone()).daemon_dir();
        // `with_roots(.., Some(ship_dir))` pins BOTH scopes to the harness's own
        // temp dir. `::arc()` leaves project scope unset, so `FilesService`
        // falls back to `get_project_dir`'s real ancestor walk — which, run
        // from inside a git worktree, redirects to the MAIN checkout's
        // `.ship/`. A test that fully dispatches a project-scope write (not
        // gate-stubbed) then lands real files on the operator's checkout.
        let rules = std::sync::Arc::new(
            ship_agents::services::rules::service::RulesServiceImpl::with_roots(
                ship_dir.clone(),
                Some(ship_dir.clone()),
            ),
        );
        let mcp = std::sync::Arc::new(
            ship_agents::services::mcp::service::McpServiceImpl::with_roots(
                ship_dir.clone(),
                Some(ship_dir.clone()),
            ),
        );
        let skills = std::sync::Arc::new(
            ship_agents::services::skills::service::SkillsServiceImpl::with_roots(
                ship_dir.clone(),
                Some(ship_dir.clone()),
            ),
        );
        let agents = std::sync::Arc::new(
            ship_agents::services::agents::service::AgentsServiceImpl::with_roots(
                ship_dir.clone(),
                Some(ship_dir.clone()),
            ),
        );
        let pipelines =
            ship_pipeline_runtime::pipelines::service::PipelinesServiceImpl::arc(ship_dir.clone());
        // Test fixture binds a concrete space (`test-space`) — there is no daemon
        // home space / `global` partition to fall back onto (S2). Production boot
        // leaves these `None` in space-less mode.
        // home space / `global` partition to fall back onto (S2). Production boot
        // leaves these `None` in space-less mode. Both collection + artifact wire
        // mounts share the SAME ctx (single per-scope db).
        // Test fixture binds a concrete space (`test-space`). The ctor is only
        // reachable from test paths that always pass a writable tmpdir, so the
        // open can't fail in practice; an open failure here would be a
        // genuine test-env bug, surface it loudly rather than silently booting
        // space-less and writing a test that masks the failure.
        let space_ctx = Some(
            super::init::build_space_db_context_arc(kernel.clone(), &ship_dir, "test-space")
                .expect("test space.db open must succeed"),
        );
        let schemas = Arc::new(ship_collection::projection::SchemaRegistry::new());
        let global_blob_dir = ship_dir.clone();
        Self {
            config: RuntimeConfig::default(),
            kernel,
            ship_dir,
            daemon_dir,
            #[cfg(feature = "bridge")]
            dispatch_handle: std::sync::OnceLock::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            space_ctx,
            schemas,
            global_blob_dir,
            live_sessions: ship_agents::services::sessions::Registry::new(),

            rules,
            // Slice 3 close-out: the runtime permissions CRUD service
            // was removed — no live UI consumer invokes it, and the
            // named-preset layer is now sourced from
            // `defineConfig.agents.permissions.presets` in
            // `stack.config.ts`.
            mcp,
            skills,
            agents,
            pipelines,
            served_space_id: std::sync::OnceLock::from(Some("test-space".to_string())),
            workspace_dir,
            started_at: std::time::Instant::now(),
            hook_engine: std::sync::OnceLock::new(),
            config_watcher: std::sync::OnceLock::new(),
            agent_def_kernel: std::sync::OnceLock::new(),
            agent_def_index: std::sync::OnceLock::new(),
            agent_reactive: std::sync::OnceLock::new(),
            bundler: std::sync::OnceLock::new(),
            shutdown_token: std::sync::OnceLock::new(),
            live_resolvers: Arc::new(crate::ship_runtime::live_resolvers::build()),
        }
    }

}
