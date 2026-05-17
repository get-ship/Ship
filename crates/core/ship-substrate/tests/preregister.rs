//! Substrate Agent pre-register — `spawn_actor` must register every
//! kind, including Agent, with the kernel before the child starts.
//!
//! This file is a single integration-test binary so the
//! `init_kernel_router` OnceLock isn't contended with other tests.

use std::collections::HashMap;

use ship_substrate::events::{init_kernel_router, kernel_router};
use ship_substrate::supervisor::{
    ActorKind, ActorProfile, TransportKind, spawn_actor, take_pre_registered_mailbox,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_pre_register_visible_in_kernel_before_mcp() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    init_kernel_router(ship_dir).expect("init kernel router");

    let log_dir = tmp.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();
    let profile = ActorProfile {
        actor_id_hint: None,
        kind: ActorKind::Agent,
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

    let handle = spawn_actor(profile).expect("spawn_actor must not error on Agent pre-register");

    let kr = kernel_router().expect("kernel router initialized");
    let kernel = kr.lock().await;
    assert!(
        kernel.has_actor(&handle.actor_id),
        "kernel router must observe the Agent actor immediately after spawn_actor returns"
    );
    drop(kernel);

    // Substrate stashed the Mailbox so the MCP-side refine path can
    // drain it into agent_mailboxes for SSE delivery. Take it now to
    // prove it was stashed; otherwise the SSE drain would 404 once
    // the MCP child connects.
    let stashed = take_pre_registered_mailbox(&handle.actor_id);
    assert!(
        stashed.is_some(),
        "Agent pre-register must stash a Mailbox for the MCP-side refine path to drain"
    );

    handle.stop().unwrap();
}
