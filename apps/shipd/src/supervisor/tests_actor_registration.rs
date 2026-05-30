//! Actor registration lifecycle tests.
//!
//! Directly exercise KernelRouter to isolate failure modes: re-registration
//! after disconnect, idempotent respawn, and subscription cleanup on stop.
//! (The HTTP-level registration path is `POST /api/v1/actor/spawn`; its
//! concurrency / idempotency is covered in `integration_tests`.)

use tempfile::TempDir;

use ship_substrate::events::{ActorConfig, CallerKind, EmitContext, EventEnvelope, KernelRouter};

fn make_kernel(dir: &TempDir) -> KernelRouter {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
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
    EmitContext { caller_kind: CallerKind::Cli, skill_id: None, space_id: None, session_id: None }
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
    let (_store, mut mailbox) = kernel
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

#[tokio::test(flavor = "multi_thread")]
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
    let (_store, mut mailbox) = kernel.spawn_actor("agent.dup", config).unwrap();

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
