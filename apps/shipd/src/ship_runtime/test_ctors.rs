//! Test constructors for ShipRuntime. Not compiled in production builds.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use ship_substrate::services::mesh::SharedMeshRegistry;
use ship_substrate::task_health::HealthRegistry;

use super::{ShipRuntime, RuntimeConfig, SharedKernel};

impl ShipRuntime {
    /// Build a ShipRuntime for tests — synchronous, takes a pre-initialized
    /// kernel and ship_dir. Does NOT run integrity checks or DB init.
    pub fn new_test(ship_dir: std::path::PathBuf, kernel: SharedKernel) -> Self {
        let workspace_dir = ship_dir.parent().map(|p| p.to_path_buf());
        let daemon_dir = ship_substrate::ShipHome::at(ship_dir.clone()).daemon_dir();
        let rules = ship_agents::services::rules::service::RulesServiceImpl::arc(ship_dir.clone());
        let mcp = ship_agents::services::mcp::service::McpServiceImpl::arc(ship_dir.clone());
        let skills = ship_agents::services::skills::service::SkillsServiceImpl::arc(ship_dir.clone());
        let agents = ship_agents::services::agents::service::AgentsServiceImpl::arc(ship_dir.clone());
        let pipelines =
            ship_pipeline_runtime::pipelines::service::PipelinesServiceImpl::arc(ship_dir.clone());
        // Test fixture binds a concrete space (`test-space`) — there is no daemon
        // home space / `global` partition to fall back onto (S2). Production boot
        // leaves these `None` in space-less mode.
        let collection_ctx =
            Some(super::init::build_space_db_context_arc(kernel.clone(), &ship_dir, "test-space"));
        let artifact_ctx =
            Some(super::init::build_space_db_context_arc(kernel.clone(), &ship_dir, "test-space"));
        let schemas = Arc::new(ship_collection::projection::SchemaRegistry::new());
        let global_blob_dir = ship_dir.clone();
        // Permissions reaches into Mcp via the trait for tool_patterns.
        // Wire the same arc both places so HTTP and trait-to-trait paths
        // observe the same catalog.
        let permissions: std::sync::Arc<dyn ship_agents::services::permissions::service::PermissionsService> =
            std::sync::Arc::new(
                ship_agents::services::permissions::service::PermissionsServiceImpl::with_roots(
                    ship_dir.clone(),
                    None,
                    mcp.clone(),
                ),
            );
        Self {
            config: RuntimeConfig::default(),
            kernel,
            mesh_registry: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            agent_mailboxes: Arc::new(Mutex::new(HashMap::new())),
            pty_connections: Arc::new(Mutex::new(HashMap::new())),
            ship_dir,
            daemon_dir,
            dispatch_handle: std::sync::OnceLock::new(),
            session_scanner: tokio::sync::OnceCell::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            collection_ctx,
            artifact_ctx,
            schemas,
            global_blob_dir,
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
            pty_sessions: crate::sessions::SessionRegistry::new(),
            actor_sessions: crate::sessions::ActorRegistry::new(),
            rules,
            permissions,
            mcp,
            skills,
            agents,
            pipelines,
            workspace_dir,
            started_at: std::time::Instant::now(),
            hook_engine: std::sync::OnceLock::new(),
            config_watcher: std::sync::OnceLock::new(),
            agent_def_kernel: std::sync::OnceLock::new(),
            agent_reactive: std::sync::OnceLock::new(),
            bundler: std::sync::OnceLock::new(),
        }
    }

    /// Test constructor that also accepts a mesh registry.
    pub fn new_test_with_registry(
        ship_dir: std::path::PathBuf,
        kernel: SharedKernel,
        mesh_registry: SharedMeshRegistry,
    ) -> Self {
        let workspace_dir = ship_dir.parent().map(|p| p.to_path_buf());
        let daemon_dir = ship_substrate::ShipHome::at(ship_dir.clone()).daemon_dir();
        let rules = ship_agents::services::rules::service::RulesServiceImpl::arc(ship_dir.clone());
        let mcp = ship_agents::services::mcp::service::McpServiceImpl::arc(ship_dir.clone());
        let skills = ship_agents::services::skills::service::SkillsServiceImpl::arc(ship_dir.clone());
        let agents = ship_agents::services::agents::service::AgentsServiceImpl::arc(ship_dir.clone());
        let pipelines =
            ship_pipeline_runtime::pipelines::service::PipelinesServiceImpl::arc(ship_dir.clone());
        // Test fixture binds a concrete space (`test-space`) — there is no daemon
        // home space / `global` partition to fall back onto (S2). Production boot
        // leaves these `None` in space-less mode.
        let collection_ctx =
            Some(super::init::build_space_db_context_arc(kernel.clone(), &ship_dir, "test-space"));
        let artifact_ctx =
            Some(super::init::build_space_db_context_arc(kernel.clone(), &ship_dir, "test-space"));
        let schemas = Arc::new(ship_collection::projection::SchemaRegistry::new());
        let global_blob_dir = ship_dir.clone();
        // Permissions reaches into Mcp via the trait for tool_patterns.
        // Wire the same arc both places so HTTP and trait-to-trait paths
        // observe the same catalog.
        let permissions: std::sync::Arc<dyn ship_agents::services::permissions::service::PermissionsService> =
            std::sync::Arc::new(
                ship_agents::services::permissions::service::PermissionsServiceImpl::with_roots(
                    ship_dir.clone(),
                    None,
                    mcp.clone(),
                ),
            );
        Self {
            config: RuntimeConfig::default(),
            kernel,
            mesh_registry,
            agent_mailboxes: Arc::new(Mutex::new(HashMap::new())),
            pty_connections: Arc::new(Mutex::new(HashMap::new())),
            ship_dir,
            daemon_dir,
            dispatch_handle: std::sync::OnceLock::new(),
            session_scanner: tokio::sync::OnceCell::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            collection_ctx,
            artifact_ctx,
            schemas,
            global_blob_dir,
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
            pty_sessions: crate::sessions::SessionRegistry::new(),
            actor_sessions: crate::sessions::ActorRegistry::new(),
            rules,
            permissions,
            mcp,
            skills,
            agents,
            pipelines,
            workspace_dir,
            started_at: std::time::Instant::now(),
            hook_engine: std::sync::OnceLock::new(),
            config_watcher: std::sync::OnceLock::new(),
            agent_def_kernel: std::sync::OnceLock::new(),
            agent_reactive: std::sync::OnceLock::new(),
            bundler: std::sync::OnceLock::new(),
        }
    }
}
