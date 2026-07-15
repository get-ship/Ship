//! Reusable test harness for daemon integration tests.
//!
//! `TestDaemon` spins up the full Axum router with a real kernel and temp
//! directory. Requests go through tower::ServiceExt (no TCP listener
//! needed).

use axum::Router;
use std::sync::Arc;
use tempfile::TempDir;

use crate::auth::AuthContext;
use crate::ship_runtime::{ApiState, SharedKernel};

/// Self-contained test daemon — temp dir, kernel, router, clean teardown.
pub struct TestDaemon {
    pub router: Router,
    pub state: ApiState,
    pub kernel: SharedKernel,
    pub temp_dir: TempDir,
}

// Note: state is Arc<ShipRuntime>, so .clone() gives another Arc handle.

impl TestDaemon {
    /// Create a fully wired TestDaemon with temp dir, DB, kernel, and router.
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let ship_dir = temp_dir.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();

        let kernel = Arc::new(tokio::sync::Mutex::new(
            ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
        ));

        let api_state: ApiState = Arc::new(
            crate::ship_runtime::ShipRuntime::new_test(ship_dir.clone(), kernel.clone()),
        );

        // The REST surface was deleted (Connect is the only path). The test
        // daemon now provides a minimal router — a tower::ServiceExt target
        // for tests that need to make HTTP requests but don't depend on any
        // particular REST route. Connect tests construct their own client.
        let router: Router = Router::new();

        Self {
            router,
            state: api_state,
            kernel,
            temp_dir,
        }
    }

    /// A verified **operator-cred** AuthContext — the elevated default for
    /// tests that exercise operator-only behavior (S6, #124). Mirrors the bearer
    /// middleware's operator-session branch: `operator_cred: true` is the SOLE
    /// elevation source. Use this where a test previously relied on
    /// `CallerKind::Cli` to mean operator.
    pub fn operator_auth() -> AuthContext {
        AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            agent_id: None,
            space_id: None,
            session_id: None,
            grants: vec![],
            // THE elevation source: a verified operator cred.
            operator_cred: true,
        }
    }

    /// A bare CLI AuthContext — capped non-operator (`operator_cred: false`), so
    /// the gate denies C2/C3. For operator behavior use
    /// [`operator_auth`](Self::operator_auth).
    pub fn cli_auth() -> AuthContext {
        AuthContext {
            caller_kind: ship_substrate::events::CallerKind::Cli,
            agent_id: None,
            space_id: None,
            session_id: None,
            grants: vec![],
            operator_cred: false,
        }
    }

}