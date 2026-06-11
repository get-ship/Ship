//! Config types, type aliases, and small utility types for ShipRuntime.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

// ---- Config types ----

/// How the daemon authenticates incoming requests.
#[derive(Debug, Clone)]
pub enum AuthMode {
    /// Token-based auth for local daemon (default).
    Local,
}

/// Runtime configuration — host, port, auth.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub host: String,
    pub port: u16,
    pub auth_mode: AuthMode,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9315,
            auth_mode: AuthMode::Local,
        }
    }
}

/// Paths and metadata for the daemon process.
pub struct AppMetadata {
    pub global_dir: PathBuf,
    /// Optional project workspace. When set, view handlers and job dispatch
    /// read from `<workspace>/.ship/` instead of `<global>/`. Without this
    /// the daemon is workspace-agnostic and serves only global config —
    /// that surface bled into project views more than once. Projects pass
    /// the flag explicitly.
    pub workspace_dir: Option<PathBuf>,
}

/// Type alias used as axum shared state.
pub type AppState = Arc<super::ShipRuntime>;

/// Canonical name for the axum shared state. Both `ApiState` and
/// `AppState` refer to the same `Arc<ShipRuntime>` handle.
pub type ApiState = Arc<super::ShipRuntime>;

// Re-export for backward compat in handlers.
pub type SharedKernel = Arc<Mutex<ship_substrate::events::KernelRouter>>;

/// RAII guard that stops a kernel actor when dropped. Used by SSE
/// handlers (`routes::runtime_api`, `views::*`) so disconnect/cancel
/// cleans up the spawned actor without leaking mailbox state.
///
/// Lives here next to `SharedKernel` so consumers across `routes/`,
/// `views/`, and any future SSE-producing module can reach it without
/// the module-DAG-violating `views → routes` import path.
pub struct ActorGuard {
    pub actor_id: String,
    pub kernel: SharedKernel,
}

impl Drop for ActorGuard {
    fn drop(&mut self) {
        let actor_id = self.actor_id.clone();
        let kernel = self.kernel.clone();
        tokio::spawn(async move {
            let _ = kernel.lock().await.stop_actor(&actor_id);
        });
    }
}
