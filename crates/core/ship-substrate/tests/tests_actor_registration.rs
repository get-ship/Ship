//! Actor registration lifecycle tests — KernelRouter primitives.
//!
//! The mesh_register REST handler tests were removed with the mesh.*/actor.*
//! op cut. These tests exercise KernelRouter directly.

use std::sync::Arc;
use tokio::sync::Mutex;
use tempfile::TempDir;

use ship_substrate::events::{ActorConfig, CallerKind, EmitContext, EventEnvelope, KernelRouter};

fn make_kernel(dir: &TempDir) -> KernelRouter {
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    KernelRouter::new(ship_dir).unwrap()
}

fn actor_config(id: &str) -> ActorConfig {
    ActorConfig {
        namespace: id.into(),
        write_namespaces: vec!["mesh.".into()],
        read_namespaces: vec!["mesh.".into()],
        subscribe_namespaces: vec!["mesh.".into()],
        see_directed_events: false,
    }
}

fn cli_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Cli,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    }
}

// ── KernelRouter primitives ─────────────────────────────────────────────────

#[test]
fn spawn_actor_fails_if_actor_exists() {
    let dir = TempDir::new().unwrap();
    let mut kernel = make_kernel(&dir);
    kernel.spawn_actor("agent.x", actor_config("agent.x")).unwrap();
    let err = kernel
        .spawn_actor("agent.x", actor_config("agent.x"))
        .expect_err("second spawn must fail when actor already exists");
    assert!(
        err.to_string().contains("already exists"),
        "error must say 'already exists', got: {err}"
    );
}

#[test]
fn stop_then_respawn_succeeds() {
    let dir = TempDir::new().unwrap();
    let mut kernel = make_kernel(&dir);
    kernel.spawn_actor("agent.x", actor_config("agent.x")).unwrap();
    kernel.stop_actor("agent.x").unwrap();
    kernel.spawn_actor("agent.x", actor_config("agent.x")).unwrap();
    assert!(kernel.has_actor("agent.x"), "actor must be present after stop + respawn");
}

#[tokio::test(flavor = "multi_thread")]
async fn stop_removes_from_subscriptions() {
    let dir = TempDir::new().unwrap();
    let mut kernel = make_kernel(&dir);
    let mut mailbox = kernel
        .spawn_actor("agent.sub", actor_config("agent.sub"))
        .unwrap();
    kernel.stop_actor("agent.sub").unwrap();

    let ev = EventEnvelope::new("mesh.test", "src", &serde_json::json!({})).unwrap();
    let _ = kernel.emit(ev, &cli_ctx()).await;

    assert!(
        mailbox.try_recv().is_err(),
        "stopped actor must not receive events after stop — subscription leak"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_actor_idempotent_via_stop_respawn() {
    let dir = TempDir::new().unwrap();
    let kernel = Arc::new(Mutex::new(make_kernel(&dir)));
    for _ in 0..2 {
        let mut k = kernel.lock().await;
        let _ = k.stop_actor("agent.idem");
        k.spawn_actor("agent.idem", actor_config("agent.idem")).unwrap();
    }
    assert!(
        kernel.lock().await.has_actor("agent.idem"),
        "actor must be registered after stop+respawn cycle"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_actor_cleans_subscriptions_completely() {
    let dir = TempDir::new().unwrap();
    let mut kernel = make_kernel(&dir);
    let config = ActorConfig {
        namespace: "agent.dup".into(),
        write_namespaces: vec!["mesh.".into()],
        read_namespaces: vec!["mesh.".into()],
        subscribe_namespaces: vec!["mesh.".into(), "ship.".into()],
        see_directed_events: false,
    };
    kernel.spawn_actor("agent.dup", config.clone()).unwrap();
    kernel.stop_actor("agent.dup").unwrap();
    let mut mailbox = kernel.spawn_actor("agent.dup", config).unwrap();

    let ev = EventEnvelope::new("mesh.ping", "src", &serde_json::json!({})).unwrap();
    let _ = kernel.emit(ev, &cli_ctx()).await;

    let mut count = 0u32;
    while mailbox.try_recv().is_ok() {
        count += 1;
    }
    assert_eq!(
        count, 1,
        "event delivered {count} times — subscription leak after stop+respawn causes double delivery"
    );
}
