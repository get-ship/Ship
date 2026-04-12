//! Tests defining the ShipRuntime manager interface.
//!
//! These tests drive the refactor of `run_network` from ad-hoc init into
//! a structured `ShipRuntime` manager. All tests reference types and methods
//! that don't exist yet — they are expected to fail with compile errors.

use std::sync::Arc;
use tempfile::TempDir;

use crate::ship_runtime::{AppMetadata, AuthMode, RuntimeConfig, ShipRuntime};

// ── Helpers ─────────────────────────────────────────────────────────────

fn temp_metadata() -> (TempDir, AppMetadata) {
    let dir = TempDir::new().unwrap();
    let meta = AppMetadata {
        global_dir: dir.path().to_path_buf(),
    };
    (dir, meta)
}

fn test_config() -> RuntimeConfig {
    RuntimeConfig {
        host: "127.0.0.1".to_string(),
        port: 9999,
        auth_mode: AuthMode::Local,
    }
}

async fn make_runtime() -> (TempDir, ShipRuntime) {
    let (dir, meta) = temp_metadata();
    let config = test_config();
    let rt = ShipRuntime::new(meta, config).await.unwrap();
    (dir, rt)
}

// ── Construction ────────────────────────────────────────────────────────

#[tokio::test]
async fn ship_runtime_creates_from_config() {
    let (_dir, meta) = temp_metadata();
    let config = RuntimeConfig {
        host: "127.0.0.1".to_string(),
        port: 9315,
        auth_mode: AuthMode::Local,
    };
    let result = ShipRuntime::new(meta, config).await;
    assert!(result.is_ok(), "ShipRuntime::new should succeed with valid config");
}

// ── Accessors ───────────────────────────────────────────────────────────

#[tokio::test]
async fn ship_runtime_exposes_kernel() {
    let (_dir, rt) = make_runtime().await;
    let kernel = rt.kernel();
    let has = kernel.lock().await.has_actor("nonexistent-actor-xyz");
    assert!(!has, "empty kernel should not contain a random actor");
}

#[tokio::test]
async fn ship_runtime_exposes_ship_dir() {
    let (dir, rt) = make_runtime().await;
    assert_eq!(
        rt.ship_dir(),
        dir.path(),
        "ship_dir() should match the global_dir from AppMetadata"
    );
}

#[tokio::test]
async fn ship_runtime_exposes_config() {
    let (_dir, rt) = make_runtime().await;
    let config = rt.config();
    assert_eq!(config.port, 9999, "config().port should match the value passed at construction");
}

// ── RuntimeConfig defaults ──────────────────────────────────────────────

#[test]
fn runtime_config_from_defaults() {
    let config = RuntimeConfig::default();
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 9315);
    assert!(
        matches!(config.auth_mode, AuthMode::Local),
        "default auth_mode should be Local"
    );
}

// ── AppState type alias ─────────────────────────────────────────────────

#[tokio::test]
async fn api_state_is_arc_ship_runtime() {
    let (_dir, rt) = make_runtime().await;
    // Verify the AppState type alias compiles and can be constructed.
    let _state: crate::ship_runtime::AppState = Arc::new(rt);
}

// ── Manager access ──────────────────────────────────────────────────────

#[tokio::test]
async fn mesh_manager_accessible() {
    let (_dir, rt) = make_runtime().await;
    let mesh = rt.mesh().await;
    // MeshManager exists and is returned — exact assertions are for
    // the mesh manager's own tests.
    let _ = mesh;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_manager_accessible() {
    let (_dir, rt) = make_runtime().await;
    let jobs = rt.jobs().await;
    let _ = jobs;
}

// ── Shutdown ────────────────────────────────────────────────────────────

#[tokio::test]
async fn shutdown_cleans_files() {
    let (dir, rt) = make_runtime().await;
    let base = dir.path();

    // Simulate the files that run_network writes on startup.
    std::fs::write(base.join("daemon.token"), "test-token").unwrap();
    std::fs::write(base.join("daemon.port"), "9999").unwrap();
    std::fs::write(base.join("daemon.pid"), "12345").unwrap();

    rt.shutdown().await;

    // Token is intentionally preserved across restarts so CLI clients
    // don't need to re-handshake. Only PID and port (liveness indicators) are removed.
    assert!(base.join("daemon.token").exists(), "daemon.token should be preserved after shutdown");
    assert!(!base.join("daemon.port").exists(), "daemon.port should be removed after shutdown");
    assert!(!base.join("daemon.pid").exists(), "daemon.pid should be removed after shutdown");
}

// ── Source-level gate ───────────────────────────────────────────────────

#[test]
fn run_network_is_small() {
    let source = include_str!("lib.rs");
    let mut in_fn = false;
    let mut brace_depth: i32 = 0;
    let mut line_count = 0u32;

    for line in source.lines() {
        if !in_fn && line.contains("pub async fn run_network(") {
            in_fn = true;
            brace_depth = 0;
        }
        if in_fn {
            brace_depth += line.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= line.chars().filter(|&c| c == '}').count() as i32;
            line_count += 1;
            if brace_depth == 0 {
                break;
            }
        }
    }

    assert!(in_fn, "run_network function not found in lib.rs");
    assert!(
        line_count <= 50,
        "run_network is {line_count} lines — should be under 50 after refactor"
    );
}
