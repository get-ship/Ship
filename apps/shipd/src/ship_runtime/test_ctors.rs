//! Test constructors for ShipRuntime. Not compiled in production builds.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use ship_platform::services::mesh::SharedMeshRegistry;
use ship_substrate::task_health::HealthRegistry;

use super::{ShipRuntime, RuntimeConfig, SharedKernel, WorktreeResolver, default_worktree_resolver};

impl ShipRuntime {
    /// Build a ShipRuntime for tests — synchronous, takes a pre-initialized
    /// kernel and ship_dir. Does NOT run integrity checks or DB init.
    pub fn new_test(ship_dir: std::path::PathBuf, kernel: SharedKernel) -> Self {
        let workspace_dir = ship_dir.parent().map(|p| p.to_path_buf());
        let daemon_dir = ship_substrate::ShipHome::at(ship_dir.clone()).daemon_dir();
        let rules = ship_platform::services::rules::service::RulesServiceImpl::arc(ship_dir.clone());
        let mcp = ship_platform::services::mcp::service::McpServiceImpl::arc(ship_dir.clone());
        let skills = ship_platform::services::skills::service::SkillsServiceImpl::arc(ship_dir.clone());
        let agents = ship_platform::services::agents::service::AgentsServiceImpl::arc(ship_dir.clone());
        let pipelines =
            ship_platform::services::pipelines::service::PipelinesServiceImpl::arc(ship_dir.clone());
        let git = ship_platform::services::git::service::GitServiceImpl::arc(ship_dir.clone());
        let data_service: Arc<tokio::sync::Mutex<Option<ship_substrate::data::DataService>>> =
            Arc::new(tokio::sync::Mutex::new(None));
        let collections =
            ship_platform::services::collections::service::CollectionsServiceImpl::arc(
                ship_dir.clone(),
                kernel.clone(),
                data_service.clone(),
            );
        let artifacts = ship_platform::services::artifacts::service::ArtifactsServiceImpl::arc(
            ship_dir.clone(),
            kernel.clone(),
            data_service.clone(),
        );
        // Permissions reaches into Mcp via the trait for tool_patterns.
        // Wire the same arc both places so HTTP and trait-to-trait paths
        // observe the same catalog.
        let permissions: std::sync::Arc<dyn ship_platform::services::permissions::service::PermissionsService> =
            std::sync::Arc::new(
                ship_platform::services::permissions::service::PermissionsServiceImpl::with_roots(
                    ship_dir.clone(),
                    None,
                    mcp.clone(),
                ),
            );
        let account = test_account_service(&ship_dir);
        Self {
            config: RuntimeConfig::default(),
            kernel,
            mesh_registry: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            agent_mailboxes: Arc::new(Mutex::new(HashMap::new())),
            pty_connections: Arc::new(Mutex::new(HashMap::new())),
            ship_dir,
            daemon_dir,
            mesh: tokio::sync::OnceCell::new(),
            jobs: tokio::sync::OnceCell::new(),
            dispatch_handle: std::sync::OnceLock::new(),
            session_scanner: tokio::sync::OnceCell::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            diag_registry: std::sync::OnceLock::new(),
            data_service,
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
            pty_sessions: crate::sessions::SessionRegistry::new(),
            actor_sessions: crate::sessions::ActorRegistry::new(),
            rules,
            permissions,
            mcp,
            skills,
            agents,
            pipelines,
            git,
            collections,
            artifacts,
            account: account.clone(),
            workspace_dir,
            started_at: std::time::Instant::now(),
            hook_engine: std::sync::OnceLock::new(),
            config_watcher: std::sync::OnceLock::new(),
            bundler: std::sync::OnceLock::new(),
            worktree_resolver: default_worktree_resolver(),
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
        let rules = ship_platform::services::rules::service::RulesServiceImpl::arc(ship_dir.clone());
        let mcp = ship_platform::services::mcp::service::McpServiceImpl::arc(ship_dir.clone());
        let skills = ship_platform::services::skills::service::SkillsServiceImpl::arc(ship_dir.clone());
        let agents = ship_platform::services::agents::service::AgentsServiceImpl::arc(ship_dir.clone());
        let pipelines =
            ship_platform::services::pipelines::service::PipelinesServiceImpl::arc(ship_dir.clone());
        let git = ship_platform::services::git::service::GitServiceImpl::arc(ship_dir.clone());
        let data_service: Arc<tokio::sync::Mutex<Option<ship_substrate::data::DataService>>> =
            Arc::new(tokio::sync::Mutex::new(None));
        let collections =
            ship_platform::services::collections::service::CollectionsServiceImpl::arc(
                ship_dir.clone(),
                kernel.clone(),
                data_service.clone(),
            );
        let artifacts = ship_platform::services::artifacts::service::ArtifactsServiceImpl::arc(
            ship_dir.clone(),
            kernel.clone(),
            data_service.clone(),
        );
        // Permissions reaches into Mcp via the trait for tool_patterns.
        // Wire the same arc both places so HTTP and trait-to-trait paths
        // observe the same catalog.
        let permissions: std::sync::Arc<dyn ship_platform::services::permissions::service::PermissionsService> =
            std::sync::Arc::new(
                ship_platform::services::permissions::service::PermissionsServiceImpl::with_roots(
                    ship_dir.clone(),
                    None,
                    mcp.clone(),
                ),
            );
        let account = test_account_service(&ship_dir);
        Self {
            config: RuntimeConfig::default(),
            kernel,
            mesh_registry,
            agent_mailboxes: Arc::new(Mutex::new(HashMap::new())),
            pty_connections: Arc::new(Mutex::new(HashMap::new())),
            ship_dir,
            daemon_dir,
            mesh: tokio::sync::OnceCell::new(),
            jobs: tokio::sync::OnceCell::new(),
            dispatch_handle: std::sync::OnceLock::new(),
            session_scanner: tokio::sync::OnceCell::new(),
            session_registry: Arc::new(Mutex::new(HashMap::new())),
            health_registry: HealthRegistry::new(),
            diag_registry: std::sync::OnceLock::new(),
            data_service,
            view_sessions: Arc::new(crate::views::session::ViewSessionRegistry::default()),
            pty_sessions: crate::sessions::SessionRegistry::new(),
            actor_sessions: crate::sessions::ActorRegistry::new(),
            rules,
            permissions,
            mcp,
            skills,
            agents,
            pipelines,
            git,
            collections,
            artifacts,
            account: account.clone(),
            workspace_dir,
            started_at: std::time::Instant::now(),
            hook_engine: std::sync::OnceLock::new(),
            config_watcher: std::sync::OnceLock::new(),
            bundler: std::sync::OnceLock::new(),
            worktree_resolver: default_worktree_resolver(),
        }
    }

    /// Test ctor variant — accepts a custom `WorktreeResolver` so tests
    /// can map workspace ids to their tempdir paths instead of consulting
    /// the daemon's CWD via `git worktree list`. See `project_api/tests.rs`.
    pub fn new_test_with_worktree_resolver(
        ship_dir: std::path::PathBuf,
        kernel: SharedKernel,
        worktree_resolver: WorktreeResolver,
    ) -> Self {
        let mut rt = Self::new_test(ship_dir, kernel);
        rt.worktree_resolver = worktree_resolver;
        rt
    }
}

/// Synthesise an `AccountService` for unit tests. Uses a tempdir-shaped
/// fake token store rooted at `ship_dir` (tests don't write to it) and a
/// localhost gateway URL so client construction succeeds without DNS.
fn test_account_service(
    ship_dir: &std::path::Path,
) -> std::sync::Arc<ship_platform::account::AccountService> {
    let store = std::sync::Arc::new(
        ship_platform::account::FileTokenStore::for_daemon_dir(ship_dir),
    );
    std::sync::Arc::new(
        ship_platform::account::AccountService::new(store, "http://127.0.0.1:1")
            .expect("test account service ctor"),
    )
}
