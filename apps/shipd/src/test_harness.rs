//! Reusable test harness for daemon integration tests.
//!
//! `TestDaemon` spins up a real kernel and temp directory. Connect tests
//! construct their own client/router over `state`.

use std::sync::Arc;
use tempfile::TempDir;

use crate::ship_runtime::{ApiState, SharedKernel};

/// Self-contained test daemon — temp dir, kernel, clean teardown.
pub struct TestDaemon {
    pub state: ApiState,
    // Never read — kept alive only so the tempdir isn't deleted (RAII guard)
    // for the duration of the harness. `state`'s `ShipRuntime` holds its own
    // `ship_dir: PathBuf` clone, not this guard.
    #[allow(dead_code)]
    pub temp_dir: TempDir,
}

// Note: state is Arc<ShipRuntime>, so .clone() gives another Arc handle.

impl TestDaemon {
    /// Create a fully wired TestDaemon with temp dir, DB, and kernel.
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let ship_dir = temp_dir.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();

        let kernel: SharedKernel = Arc::new(tokio::sync::Mutex::new(
            ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
        ));

        let api_state: ApiState = Arc::new(
            crate::ship_runtime::ShipRuntime::new_test(ship_dir.clone(), kernel),
        );

        Self {
            state: api_state,
            temp_dir,
        }
    }
}