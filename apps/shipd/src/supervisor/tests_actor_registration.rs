//! Actor registration lifecycle tests.
//!
//! Directly exercise KernelRouter and mesh_register to isolate failure modes:
//! re-registration after disconnect, concurrent registration of the same
//! agent, and idempotent double-registration.

use axum::extract::{Json, State};
use axum::response::IntoResponse;
use axum::Extension;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

use runtime::events::{ActorConfig, CallerKind, EmitContext, EventEnvelope, KernelRouter};
use crate::rest_api::{ApiState, MeshRegisterReq, mesh_register};
use crate::test_harness::TestDaemon;

fn make_kernel(dir: &TempDir) -> KernelRouter {
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    runtime::db::ensure_db_at(&ship_dir).unwrap();
    KernelRouter::new(ship_dir).unwrap()
}

fn make_api_state(dir: &TempDir) -> ApiState {
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    runtime::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = Arc::new(Mutex::new(KernelRouter::new(ship_dir.clone()).unwrap()));
    let mesh_registry = Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));
    ApiState {
        kernel,
        mesh_registry,
        agent_mailboxes: Arc::new(Mutex::new(std::collections::HashMap::new())),
        pty_connections: Arc::new(Mutex::new(std::collections::HashMap::new())),
        ship_dir,
    }
}

fn actor_config(id: &str) -> ActorConfig {
    ActorConfig {
        namespace: id.into(),
        write_namespaces: vec!["mesh.".into()],
        read_namespaces: vec!["mesh.".into()],
        subscribe_namespaces: vec!["mesh.".into()],
    }
}

fn cli_ctx() -> EmitContext {
    EmitContext { caller_kind: CallerKind::Cli, skill_id: None, workspace_id: None, session_id: None }
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
    let _ = kernel.route(ev, &cli_ctx()).await;

    assert!(
        mailbox.try_recv().is_err(),
        "stopped actor must not receive events after stop — subscription leak"
    );
}

// ── mesh_register failure modes ─────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn mesh_register_idempotent() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);
    for _ in 0..2 {
        mesh_register(
            State(state.clone()),
            Extension(TestDaemon::cli_auth()),
            Json(MeshRegisterReq { agent_id: "agent.idem".into(), capabilities: vec![] }),
        )
        .await;
    }
    assert!(
        state.kernel.lock().await.has_actor("agent.idem"),
        "actor must be registered after double mesh_register — idempotent re-register must succeed"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn mesh_register_after_stop_actor() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);
    mesh_register(
        State(state.clone()),
        Extension(TestDaemon::cli_auth()),
        Json(MeshRegisterReq { agent_id: "agent.reconnect".into(), capabilities: vec![] }),
    )
    .await;
    state.kernel.lock().await.stop_actor("agent.reconnect").unwrap();
    mesh_register(
        State(state.clone()),
        Extension(TestDaemon::cli_auth()),
        Json(MeshRegisterReq { agent_id: "agent.reconnect".into(), capabilities: vec![] }),
    )
    .await;
    assert!(
        state.kernel.lock().await.has_actor("agent.reconnect"),
        "actor must be re-registered after external stop — reconnect must succeed"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mesh_register_concurrent_same_agent() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);
    let s1 = state.clone();
    let s2 = state.clone();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        async {
            let h1 = tokio::spawn(async move {
                mesh_register(
                    State(s1),
                    Extension(TestDaemon::cli_auth()),
                    Json(MeshRegisterReq { agent_id: "agent.race".into(), capabilities: vec!["v1".into()] }),
                )
                .await;
            });
            let h2 = tokio::spawn(async move {
                mesh_register(
                    State(s2),
                    Extension(TestDaemon::cli_auth()),
                    Json(MeshRegisterReq { agent_id: "agent.race".into(), capabilities: vec!["v2".into()] }),
                )
                .await;
            });
            h1.await.unwrap();
            h2.await.unwrap();
        },
    )
    .await;
    assert!(result.is_ok(), "concurrent registration must not deadlock — must complete within 2s");
    assert!(
        state.kernel.lock().await.has_actor("agent.race"),
        "actor must exist after concurrent registration — one registration must win"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spawn_actor_error_is_not_silent() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);
    let s1 = state.clone();
    let s2 = state.clone();
    let (resp1, resp2) = tokio::join!(
        async move {
            mesh_register(
                State(s1),
                Extension(TestDaemon::cli_auth()),
                Json(MeshRegisterReq { agent_id: "agent.silent".into(), capabilities: vec![] }),
            )
            .await
            .into_response()
        },
        async move {
            mesh_register(
                State(s2),
                Extension(TestDaemon::cli_auth()),
                Json(MeshRegisterReq { agent_id: "agent.silent".into(), capabilities: vec![] }),
            )
            .await
            .into_response()
        },
    );
    let has = state.kernel.lock().await.has_actor("agent.silent");
    // Invariant: 200 OK is only valid when the actor is registered.
    // Silent failure: spawn_actor Err is swallowed, response is 200 but actor absent.
    assert!(
        has || !resp1.status().is_success() || !resp2.status().is_success(),
        "both calls returned 2xx but actor is not registered — spawn_actor error is swallowed silently"
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
    };
    kernel.spawn_actor("agent.dup", config.clone()).unwrap();
    kernel.stop_actor("agent.dup").unwrap();
    let (_store, mut mailbox) = kernel.spawn_actor("agent.dup", config).unwrap();

    let ev = EventEnvelope::new("mesh.ping", "src", &serde_json::json!({})).unwrap();
    let _ = kernel.route(ev, &cli_ctx()).await;

    let mut count = 0u32;
    while mailbox.try_recv().is_ok() {
        count += 1;
    }
    assert_eq!(
        count, 1,
        "event delivered {count} times — subscription leak after stop+respawn causes double delivery"
    );
}
