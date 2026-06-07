//! Device window-service proofs — the relocated window-management logic behaves
//! identically to the legacy space-window surface.
//!
//! Substrate-backed reads (the actor projection) and writes (register/close emit
//! `actor.created`/`actor.stopped` through the event-sourced actor-events path)
//! route through a `SHIP_HOME` pinned to a fresh tempdir. The `WindowSessions`
//! and `SpaceRegistry` seams are recording / configurable fakes so the *decision*
//! is asserted without a live session registry or the L1 projects projection.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use anyhow::Result;
use async_trait::async_trait;

use super::{close, intent, list, register, WindowIntent};
use crate::error::DeviceErrorKind;
use crate::ports::{SpaceRegistry, WindowSessions};

// ── Fakes ────────────────────────────────────────────────────────────────

/// Records mint calls; revoke returns a configurable result and records.
#[derive(Default)]
struct RecordingSessions {
    minted: Mutex<Vec<(String, u32)>>,
    revoked: Mutex<Vec<(String, u32)>>,
    revoke_result: AtomicBool,
}

#[async_trait]
impl WindowSessions for RecordingSessions {
    async fn mint_window_session(&self, space_id: &str, pid: u32) -> Result<String> {
        self.minted.lock().unwrap().push((space_id.to_string(), pid));
        Ok(format!("ship_sess_fake_{space_id}_{pid}"))
    }
    async fn revoke_window_session(&self, space_id: &str, pid: u32) -> bool {
        self.revoked.lock().unwrap().push((space_id.to_string(), pid));
        self.revoke_result.load(Ordering::SeqCst)
    }
}

/// A space registry that answers a fixed membership set, or fails the lookup.
struct FakeRegistry {
    /// Ids that are "registered". Unknown ids miss.
    known: Vec<String>,
    /// When true, every lookup surfaces an error (proves the service maps a
    /// registry failure to a service error, not "absent").
    fail: bool,
}

impl FakeRegistry {
    fn known(ids: &[&str]) -> Self {
        Self { known: ids.iter().map(|s| s.to_string()).collect(), fail: false }
    }
    fn empty() -> Self {
        Self { known: vec![], fail: false }
    }
    fn failing() -> Self {
        Self { known: vec![], fail: true }
    }
}

impl SpaceRegistry for FakeRegistry {
    fn space_exists(&self, space_id: &str) -> Result<bool> {
        if self.fail {
            anyhow::bail!("registry unreachable");
        }
        Ok(self.known.iter().any(|id| id == space_id))
    }
}

// ── Substrate-backed test env (SHIP_HOME pin) ──────────────────────────────

/// Serializes every global-dir-pinning window test so they never interleave (so
/// `SHIP_HOME` mutation is single-writer). Register/close emit through
/// `spawn_blocking`, which runs on a *different* OS thread than the test; the
/// per-thread auto test-global-dir would give that thread a different db than the
/// read-side. Pinning `SHIP_HOME` to one tempdir (auto-dir off) makes every
/// thread resolve the SAME runtime.db, so the read sees the write.
static GLOBAL_DIR_LOCK: Mutex<()> = Mutex::new(());

/// Pins `SHIP_HOME` to a fresh `.ship` tempdir for the test's lifetime (auto
/// test-global-dir disabled), pre-ensures runtime.db, and yields a unique space
/// id. The actor projection ships in the substrate default bus, so register/close
/// materialize the `actors` table without any extra projection install. Restores
/// env on drop.
struct DeviceTestEnv {
    _guard: std::sync::MutexGuard<'static, ()>,
    _tmp: tempfile::TempDir,
    prev_home: Option<std::ffi::OsString>,
    prev_disable: Option<std::ffi::OsString>,
    space: String,
}

impl DeviceTestEnv {
    fn new() -> Self {
        let guard = GLOBAL_DIR_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = ship_substrate::testutil::init_project_minimal(tmp.path()).unwrap();
        let prev_home = std::env::var_os("SHIP_HOME");
        let prev_disable = std::env::var_os("SHIP_DISABLE_AUTO_TEST_GLOBAL_DIR");
        // SAFETY: the static lock is held for the lifetime of this guard, so no
        // other window test mutates these vars concurrently. Restored on drop.
        unsafe {
            std::env::set_var("SHIP_DISABLE_AUTO_TEST_GLOBAL_DIR", "1");
            std::env::set_var("SHIP_HOME", &ship_dir);
        }
        ship_substrate::db::open_runtime_db(&ship_dir).unwrap();
        let space = ship_substrate::gen_ulid();
        Self { _guard: guard, _tmp: tmp, prev_home, prev_disable, space }
    }
}

impl Drop for DeviceTestEnv {
    fn drop(&mut self) {
        // SAFETY: lock still held; restore mirrors acquire.
        unsafe {
            match &self.prev_home {
                Some(v) => std::env::set_var("SHIP_HOME", v),
                None => std::env::remove_var("SHIP_HOME"),
            }
            match &self.prev_disable {
                Some(v) => std::env::set_var("SHIP_DISABLE_AUTO_TEST_GLOBAL_DIR", v),
                None => std::env::remove_var("SHIP_DISABLE_AUTO_TEST_GLOBAL_DIR"),
            }
        }
    }
}

// ── register: pid attestation + target resolution ─────────────────────────
//
// The daemon always resolves a CONCRETE registered space id before calling
// `register` — there is no home/global fallback in the crate (S2 home-space
// kill). The space-less "no target" refusal lives at the transport (shipd
// `device_rpc::register_window`), so the crate sees only a concrete id.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_requires_attested_pid() {
    let sessions = RecordingSessions::default();
    let registry = FakeRegistry::known(&["space-a"]);
    let err = register(&sessions, &registry, None, "space-a")
        .await
        .expect_err("no pid ⟹ refused");
    assert_eq!(err.kind, DeviceErrorKind::Forbidden);
    assert!(sessions.minted.lock().unwrap().is_empty(), "nothing minted without a pid");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_unregistered_target_space_is_not_found() {
    let _env = DeviceTestEnv::new();
    let sessions = RecordingSessions::default();
    let registry = FakeRegistry::empty();
    let err = register(
        &sessions,
        &registry,
        Some(std::process::id()),
        "space-does-not-exist",
    )
    .await
    .expect_err("a window can't conjure a space");
    assert_eq!(err.kind, DeviceErrorKind::NotFound);
    assert!(sessions.minted.lock().unwrap().is_empty(), "no mint for a missing target");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_surfaces_registry_failure_as_service_error() {
    let _env = DeviceTestEnv::new();
    let sessions = RecordingSessions::default();
    let registry = FakeRegistry::failing();
    let err = register(&sessions, &registry, Some(std::process::id()), "space-a")
        .await
        .expect_err("a registry lookup failure is a service error, not 'absent'");
    assert_eq!(err.kind, DeviceErrorKind::Service);
    assert!(sessions.minted.lock().unwrap().is_empty());
}

// ── register: the actor + mint happy path, observable via intent/list ───────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_validated_target_id_binds_that_space() {
    let env = DeviceTestEnv::new();
    let space = env.space.clone();
    let sessions = RecordingSessions::default();
    // The registry vouches for the explicit, concrete target id.
    let registry = FakeRegistry::known(&[space.as_str()]);
    let pid = std::process::id();
    let resp = register(&sessions, &registry, Some(pid), &space)
        .await
        .expect("a registered target id is accepted");
    assert_eq!(resp.space_id, space, "binds the validated target id");
    assert!(!resp.actor_id.is_empty());
    // The session was minted for exactly this (space, pid).
    assert_eq!(*sessions.minted.lock().unwrap(), vec![(space.clone(), pid)]);
    assert_eq!(resp.session_token, format!("ship_sess_fake_{space}_{pid}"));
    // The window is now the live window for its space → intent focuses it.
    let decision = intent(&space).expect("intent read");
    assert_eq!(decision, WindowIntent::Focus { actor_id: resp.actor_id.clone() });
    // And it shows up in the window list.
    let listed = list().expect("list windows");
    assert!(listed
        .iter()
        .any(|w| w.id == resp.actor_id && w.space_id.as_deref() == Some(&space)));
}

// ── intent: spawn when no live window ───────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn intent_spawns_when_no_live_window() {
    let env = DeviceTestEnv::new();
    let space = env.space.clone();
    let decision = intent(&space).expect("intent read");
    assert_eq!(decision, WindowIntent::Spawn);
}

// ── close: revoke + deregister ──────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn close_revokes_token_and_deregisters() {
    let env = DeviceTestEnv::new();
    let space = env.space.clone();
    let sessions = RecordingSessions::default();
    sessions.revoke_result.store(true, Ordering::SeqCst);
    let registry = FakeRegistry::known(&[space.as_str()]);
    let pid = std::process::id();
    let reg = register(&sessions, &registry, Some(pid), &space)
        .await
        .expect("register");

    let closed = close(&sessions, &reg.actor_id).await.expect("close");
    assert!(closed.closed);
    assert!(closed.token_revoked, "revoke returned true");
    assert_eq!(*sessions.revoked.lock().unwrap(), vec![(space.clone(), pid)]);
    // Deregistered → no longer the live window for its space.
    let decision = intent(&space).expect("intent read");
    assert_eq!(decision, WindowIntent::Spawn, "closed window leaves the live set");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn close_unknown_actor_is_bad_args() {
    let _env = DeviceTestEnv::new();
    let sessions = RecordingSessions::default();
    let err = close(&sessions, "no-such-actor")
        .await
        .expect_err("unknown actor rejected");
    assert_eq!(err.kind, DeviceErrorKind::BadArgs);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn close_non_window_actor_is_bad_args() {
    let env = DeviceTestEnv::new();
    let space = env.space.clone();
    // Register a non-window actor, then try to close it as a window.
    let actor_id = ship_substrate::gen_ulid();
    let aid = actor_id.clone();
    let sid = space.clone();
    tokio::task::spawn_blocking(move || {
        ship_substrate::db::actor_events::emit_actor_created_simple(
            &aid, "agent", Some(&sid), None, None, None, None,
        )
    })
    .await
    .unwrap()
    .unwrap();
    let sessions = RecordingSessions::default();
    let err = close(&sessions, &actor_id)
        .await
        .expect_err("non-window actor rejected");
    assert_eq!(err.kind, DeviceErrorKind::BadArgs);
    assert!(sessions.revoked.lock().unwrap().is_empty(), "no revoke for a non-window");
}
