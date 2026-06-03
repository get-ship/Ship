//! Space service tests — proof the relocated product logic behaves identically.
//!
//! Reads that hit the substrate (projects projection, actor projection,
//! space_settings overlay) route through the auto-created per-thread test
//! global dir; the window-touching tests pre-ensure runtime.db and stamp a
//! daemon space exactly like the (now-thin) daemon handler's tests did. The
//! `SpaceEvents` / `WindowSessions` seams are recording fakes so the *decision*
//! is asserted without a live kernel or session registry.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use super::ports::{SpaceEvents, WindowSessions};
use super::{config, info, windows, SpaceErrorKind};
use windows::{TargetSpace, WindowIntent};

// ── Fakes ────────────────────────────────────────────────────────────────

/// Records each `space.setting.changed` emit as `(space_id, key, value_json)`.
#[derive(Default)]
struct RecordingEvents {
    emitted: Mutex<Vec<(String, String, String)>>,
}

#[async_trait]
impl SpaceEvents for RecordingEvents {
    async fn emit_setting_changed(&self, space_id: &str, key: &str, value_json: &str) -> Result<()> {
        self.emitted
            .lock()
            .unwrap()
            .push((space_id.to_string(), key.to_string(), value_json.to_string()));
        Ok(())
    }
}

/// `SpaceEvents` that fails — proves the service surfaces emit failures.
struct FailingEvents;

#[async_trait]
impl SpaceEvents for FailingEvents {
    async fn emit_setting_changed(&self, _: &str, _: &str, _: &str) -> Result<()> {
        anyhow::bail!("kernel emit failed")
    }
}

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

// ── config::set ────────────────────────────────────────────────────────────

#[tokio::test]
async fn set_rejects_non_json_value() {
    let events = RecordingEvents::default();
    let err = config::set(&events, "space-a", "theme", "not json at all")
        .await
        .expect_err("invalid JSON is rejected");
    assert_eq!(err.kind, SpaceErrorKind::BadArgs);
    assert!(events.emitted.lock().unwrap().is_empty(), "nothing emitted on bad input");
}

#[tokio::test]
async fn set_emits_canonical_value_and_echoes() {
    let events = RecordingEvents::default();
    // Whitespace-laden but valid JSON re-serializes to canonical form.
    let out = config::set(&events, "space-a", "theme", "  \"dark\"  ")
        .await
        .expect("valid JSON is accepted");
    assert_eq!(out.key, "theme");
    assert_eq!(out.value_json, "\"dark\"");
    let emitted = events.emitted.lock().unwrap();
    assert_eq!(
        *emitted,
        vec![("space-a".to_string(), "theme".to_string(), "\"dark\"".to_string())],
        "exactly one emit, space rides the call, value canonicalized"
    );
}

#[tokio::test]
async fn set_surfaces_emit_failure_as_service_error() {
    let err = config::set(&FailingEvents, "space-a", "theme", "\"dark\"")
        .await
        .expect_err("emit failure propagates");
    assert_eq!(err.kind, SpaceErrorKind::Service);
}

// ── info::compose (overlay) ────────────────────────────────────────────────

#[test]
fn compose_passes_settings_through_when_no_space() {
    let base = json!({ "theme": "light", "x": 1 });
    let info = info::compose(None, base.clone());
    assert_eq!(info.space_id, None);
    assert_eq!(info.root, None);
    assert_eq!(info.name, None);
    assert_eq!(info.settings, base, "no space ⟹ unoverlaid base");
}

#[test]
fn compose_overlays_space_overrides_over_global_base() {
    // No registered project + no overrides for this space id ⟹ identity is
    // empty and the base passes through. The overlay-precedence + identity
    // resolution against live projections is exercised by the integration
    // test below; here we pin the no-override branch (base preserved).
    let base = json!({ "theme": "light" });
    let composed = info::compose(Some("space-unknown"), base.clone());
    assert_eq!(composed.space_id.as_deref(), Some("space-unknown"));
    // Unknown space ⟹ no projects row ⟹ no identity, base unchanged.
    assert_eq!(composed.settings, base);
}

// ── windows: intent / register / close (substrate-backed) ──────────────────

/// Serializes every global-dir-pinning window test so they never interleave
/// (and so `SHIP_HOME` mutation is single-writer). Window register/close emit
/// through `spawn_blocking`, which runs on a *different* OS thread than the
/// test; the per-thread auto test-global-dir would give that thread a different
/// db than the read-side. Pinning `SHIP_HOME` to one tempdir (auto-dir off)
/// makes every thread resolve the SAME runtime.db, so the read sees the write.
static GLOBAL_DIR_LOCK: Mutex<()> = Mutex::new(());

/// Pins `SHIP_HOME` to a fresh `.ship` tempdir for the test's lifetime (auto
/// test-global-dir disabled), installs the L1 extra projections, pre-ensures
/// runtime.db, and returns a unique space id. Restores env on drop.
struct SpaceTestEnv {
    _guard: std::sync::MutexGuard<'static, ()>,
    _tmp: tempfile::TempDir,
    prev_home: Option<std::ffi::OsString>,
    prev_disable: Option<std::ffi::OsString>,
    space: String,
}

impl SpaceTestEnv {
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
        // Idempotent: registers ProjectsProjection + SpaceSettingsProjection on
        // the default-bus factory so emits materialize the read models.
        crate::install_defaults();
        ship_substrate::db::open_runtime_db(&ship_dir).unwrap();
        let space = ship_substrate::gen_ulid();
        Self { _guard: guard, _tmp: tmp, prev_home, prev_disable, space }
    }

    /// The project root (the tempdir holding `.ship/`).
    fn root(&self) -> &std::path::Path {
        self._tmp.path()
    }
}

impl Drop for SpaceTestEnv {
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_requires_attested_pid() {
    let sessions = RecordingSessions::default();
    let err = windows::register(&sessions, None, TargetSpace::Home, Some("space-a"))
        .await
        .expect_err("no pid ⟹ refused");
    assert_eq!(err.kind, SpaceErrorKind::Forbidden);
    assert!(sessions.minted.lock().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_home_with_no_home_space_is_conflict() {
    let sessions = RecordingSessions::default();
    let err = windows::register(&sessions, Some(1234), TargetSpace::Home, None)
        .await
        .expect_err("no home space ⟹ conflict");
    assert_eq!(err.kind, SpaceErrorKind::Conflict);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_unregistered_target_space_is_not_found() {
    let _env = SpaceTestEnv::new();
    let sessions = RecordingSessions::default();
    let err = windows::register(
        &sessions,
        Some(std::process::id()),
        TargetSpace::Id("space-does-not-exist"),
        None,
    )
    .await
    .expect_err("a window can't conjure a space");
    assert_eq!(err.kind, SpaceErrorKind::NotFound);
    assert!(sessions.minted.lock().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn register_home_registers_actor_and_mints_session() {
    let env = SpaceTestEnv::new();
    let space = env.space.clone();
    let sessions = RecordingSessions::default();
    let pid = std::process::id();
    let resp = windows::register(&sessions, Some(pid), TargetSpace::Home, Some(&space))
        .await
        .expect("operator registers a window onto home");
    assert_eq!(resp.space_id, space, "no target ⟹ binds the home space");
    assert!(!resp.actor_id.is_empty());
    // The session was minted for exactly this (space, pid).
    assert_eq!(*sessions.minted.lock().unwrap(), vec![(space.clone(), pid)]);
    assert_eq!(resp.session_token, format!("ship_sess_fake_{space}_{pid}"));
    // The window is now the live window for its space → intent focuses it.
    let intent = windows::intent(&space).expect("intent read");
    assert_eq!(intent, WindowIntent::Focus { actor_id: resp.actor_id.clone() });
    // And it shows up in the window list.
    let listed = windows::list().expect("list windows");
    assert!(listed.iter().any(|w| w.id == resp.actor_id && w.space_id.as_deref() == Some(&space)));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn intent_spawns_when_no_live_window() {
    let env = SpaceTestEnv::new();
    let space = env.space.clone();
    let intent = windows::intent(&space).expect("intent read");
    assert_eq!(intent, WindowIntent::Spawn);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn close_revokes_token_and_deregisters() {
    let env = SpaceTestEnv::new();
    let space = env.space.clone();
    let sessions = RecordingSessions::default();
    sessions.revoke_result.store(true, Ordering::SeqCst);
    let pid = std::process::id();
    let reg = windows::register(&sessions, Some(pid), TargetSpace::Home, Some(&space))
        .await
        .expect("register");

    let closed = windows::close(&sessions, &reg.actor_id).await.expect("close");
    assert!(closed.closed);
    assert!(closed.token_revoked, "revoke returned true");
    assert_eq!(*sessions.revoked.lock().unwrap(), vec![(space.clone(), pid)]);
    // Deregistered → no longer the live window for its space.
    let intent = windows::intent(&space).expect("intent read");
    assert_eq!(intent, WindowIntent::Spawn, "closed window leaves the live set");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn close_unknown_actor_is_bad_args() {
    let _env = SpaceTestEnv::new();
    let sessions = RecordingSessions::default();
    let err = windows::close(&sessions, "no-such-actor")
        .await
        .expect_err("unknown actor rejected");
    assert_eq!(err.kind, SpaceErrorKind::BadArgs);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn close_non_window_actor_is_bad_args() {
    let env = SpaceTestEnv::new();
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
    let err = windows::close(&sessions, &actor_id)
        .await
        .expect_err("non-window actor rejected");
    assert_eq!(err.kind, SpaceErrorKind::BadArgs);
    assert!(sessions.revoked.lock().unwrap().is_empty(), "no revoke for a non-window");
}

// ── info::compose against a live project + override (integration) ──────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn compose_resolves_identity_and_overlays_live_override() {
    let env = SpaceTestEnv::new();
    // Register a project so the projects projection has identity. The space id
    // is the projection's primary key (the path hash) — use the returned id as
    // the authoritative space identity.
    let record = crate::projects::register(env.root(), Some("Acme")).unwrap();
    let space = record.id.clone();

    // Emit a per-space override so space_settings::get_all sees it.
    write_space_override(&space, "theme", "\"dark\"");

    let base: Value = json!({ "theme": "light", "untouched": true });
    let composed = info::compose(Some(&space), base);
    assert_eq!(composed.space_id.as_deref(), Some(space.as_str()));
    assert_eq!(composed.name.as_deref(), Some("Acme"));
    assert_eq!(composed.root.as_deref(), Some(record.path.as_str()));
    // Space override wins per-key; non-overridden keys survive.
    assert_eq!(composed.settings["theme"], json!("dark"));
    assert_eq!(composed.settings["untouched"], json!(true));
}

/// Emit a `space.setting.changed` event for `(space, key)` riding the space on
/// the envelope, so the `space_settings` projection materializes the overlay.
fn write_space_override(space_id: &str, key: &str, value_json: &str) {
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
    use ship_types::event_types::SPACE_SETTING_CHANGED;
    use ship_types::payload::SpaceSettingChanged;

    let payload = SpaceSettingChanged {
        key: key.to_string(),
        value_json: value_json.to_string(),
    };
    let envelope = EventEnvelope::new(SPACE_SETTING_CHANGED, key, &payload)
        .unwrap()
        .with_context(Some(space_id), None);
    let store = SqliteEventStore::new_for_scope(
        &ship_substrate::project::get_global_dir().unwrap(),
        ship_types::event_registry::EventScope::Space,
        Some(space_id),
    )
    .unwrap();
    EventStore::append(&store, &envelope).unwrap();
}
