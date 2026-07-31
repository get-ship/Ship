//! Kernel Agent pre-register — `spawn_process` with the
//! [`ship_agents::supervisor_sink::KernelSupervisorSink`] must register every
//! kind, including Agent, with the kernel before the child starts.
#![cfg(feature = "bridge")]

use std::collections::HashMap;

use bridge::{ProcessKind, ProcessProfile, TransportKind, spawn_process};
use ship_agents::supervisor_sink::supervisor_context;
use ship_substrate::events::init_kernel_router;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_pre_register_visible_in_kernel_before_mcp() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let kr = init_kernel_router(ship_dir).expect("init kernel router");

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
        agent_id: Some("test-agent".into()),
        harness: Some("claude".into()),
        parent_actor_id: None,
        log_dir: Some(log_dir),
        subscribe_namespaces: vec![],
    };

    let handle = spawn_process(profile, supervisor_context(kr.clone()).expect("ctx"))
        .expect("spawn_process must not error on Agent pre-register");

    let kernel = kr.lock().await;
    assert!(
        kernel.has_actor(&handle.actor_id),
        "kernel router must observe the Agent actor immediately after spawn_process returns"
    );
    drop(kernel);

    handle.stop().unwrap();
}
