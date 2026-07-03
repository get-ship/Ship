//! Substrate Agent pre-register — `spawn_process` must register every
//! kind, including Agent, with the kernel before the child starts.
//!
//! This file is a single integration-test binary so the
//! `init_kernel_router` OnceLock isn't contended with other tests.

use std::collections::HashMap;

use ship_substrate::events::{init_kernel_router, kernel_router};
use ship_substrate::supervisor::{ProcessKind, ProcessProfile, TransportKind, spawn_process};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_pre_register_visible_in_kernel_before_mcp() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    init_kernel_router(ship_dir).expect("init kernel router");

    let log_dir = tmp.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();
    let profile = ProcessProfile {
        actor_id_hint: None,
        kind: ProcessKind::Agent,
        transport: TransportKind::Pty,
        one_shot: false,
        bin: "/bin/cat".into(),
        args: vec![],
        env: HashMap::new(),
        work_dir: std::env::temp_dir(),
        workspace_id: None,
        agent_id: Some("test-agent".into()),
        harness: Some("claude".into()),
        parent_actor_id: None,
        log_dir: Some(log_dir),
        subscribe_namespaces: vec![],
    };

    let handle = spawn_process(profile).expect("spawn_process must not error on Agent pre-register");

    let kr = kernel_router().expect("kernel router initialized");
    let kernel = kr.lock().await;
    assert!(
        kernel.has_actor(&handle.actor_id),
        "kernel router must observe the Agent actor immediately after spawn_process returns"
    );
    drop(kernel);

    handle.stop().unwrap();
}
