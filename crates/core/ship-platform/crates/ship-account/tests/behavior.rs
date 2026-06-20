//! Behavioral proofs for `ship-account`.
//!
//! These prove the design's intended behaviors over the substrate ports — token
//! at rest via config_doc, `account.*` audit via KernelLog (landing in
//! `account.db`), and the offline-logout invariant. No network: the gateway
//! base URL points at an unroutable address so `token_exchange`/`revoke_key`
//! fail fast, exercising the local-only fallbacks.

use std::sync::Arc;

use ship_account::service::{AccountService, KernelOwnerBinder, OwnerBinder};
use ship_account::store::{AccountTokenStore, FileTokenStore};
use ship_account::types::{AccountState, AccountToken};

use ship_substrate::events::global_kernel_router::new_isolated_for_test;
use ship_substrate::home::ShipHome;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_substrate::ports::KernelLog;

// ── Fixtures ──────────────────────────────────────────────────────────────────

/// Owner-binder that records bind/unbind calls — proves the service drives the
/// owner-binding step without coupling the crate to ship-platform's `attest`.
#[derive(Default)]
struct RecordingBinder {
    bound: std::sync::Mutex<Vec<String>>,
    unbound: std::sync::Mutex<Vec<String>>,
}

#[async_trait::async_trait]
impl OwnerBinder for RecordingBinder {
    async fn bind(&self, user_id: &str) {
        self.bound.lock().unwrap().push(user_id.to_string());
    }
    async fn unbind(&self, user_id: &str) {
        self.unbound.lock().unwrap().push(user_id.to_string());
    }
}

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = new_isolated_for_test(ship_dir).unwrap();
    Arc::new(KernelLog::new(router))
}

/// Build a service over a temp token store + isolated kernel, pointed at a
/// reachable-looking but offline gateway so login/logout exercise local paths.
fn make_service(
    tmp: &tempfile::TempDir,
    kernel: Arc<KernelLog>,
    owner: Arc<dyn OwnerBinder>,
) -> (Arc<FileTokenStore>, AccountService) {
    let store = Arc::new(FileTokenStore::new(tmp.path().join("daemon/account.jsonc")));
    let svc = AccountService::new(store.clone(), kernel, owner, "https://api.getship.dev").unwrap();
    (store, svc)
}

/// Offline service: gateway base points at an unroutable port so revoke/exchange
/// fail fast.
fn make_service_offline(
    tmp: &tempfile::TempDir,
    kernel: Arc<KernelLog>,
    owner: Arc<dyn OwnerBinder>,
) -> (Arc<FileTokenStore>, AccountService) {
    let store = Arc::new(FileTokenStore::new(tmp.path().join("daemon/account.jsonc")));
    let svc = AccountService::new(store.clone(), kernel, owner, "http://127.0.0.1:1").unwrap();
    (store, svc)
}

/// Read all events of a type back from `account.db` (Account scope).
fn account_events(tmp: &tempfile::TempDir, event_type: &str) -> Vec<ship_substrate::events::EventEnvelope> {
    let root = ShipHome::project_ship_dir(tmp.path());
    let path = ship_substrate::db::account_db_path(&root);
    if !path.exists() {
        return Vec::new();
    }
    let store = SqliteEventStore::new_at_db_path(&path).unwrap();
    store
        .query(&ship_substrate::events::EventFilter {
            event_type: Some(event_type.to_string()),
            ..Default::default()
        })
        .unwrap()
}

fn sample_token(user: &str) -> AccountToken {
    AccountToken {
        key: format!("ship_{user}_key"),
        user_id: user.to_string(),
        expires_at: 0,
        stored_at: 0,
    }
}

// ── State proofs ──────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn state_is_logged_out_before_login() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let (_store, svc) = make_service(&tmp, kernel, Arc::new(RecordingBinder::default()));
    assert!(matches!(svc.state().await.unwrap(), AccountState::LoggedOut));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn state_is_logged_in_after_token_stored() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let (store, svc) = make_service(&tmp, kernel, Arc::new(RecordingBinder::default()));
    store.write(sample_token("user-1")).await.unwrap();
    match svc.state().await.unwrap() {
        AccountState::LoggedIn { user_id, .. } => assert_eq!(user_id, "user-1"),
        other => panic!("expected LoggedIn, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expired_token_reads_as_logged_out() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let (store, svc) = make_service(&tmp, kernel, Arc::new(RecordingBinder::default()));
    store
        .write(AccountToken { key: "k".into(), user_id: "u".into(), expires_at: 1, stored_at: 0 })
        .await
        .unwrap();
    assert!(
        matches!(svc.state().await.unwrap(), AccountState::LoggedOut),
        "expired token must read as logged_out, never logged_in",
    );
}

// ── Login-start proofs ─────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn start_login_registers_pending_and_returns_browser_url() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let (_store, svc) = make_service(&tmp, kernel, Arc::new(RecordingBinder::default()));
    let resp = svc.start_login("http://127.0.0.1:54321/callback").await.unwrap();
    assert!(resp.browser_url.starts_with("https://api.getship.dev/v1/auth/cli?"));
    assert!(resp.browser_url.contains("code_challenge="));
    assert!(resp.browser_url.contains(&format!("state={}", urlencoding::encode(&resp.state_id))));
    assert_eq!(svc.pending_len().await, 1, "pending entry recorded");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn complete_login_with_unknown_state_errors_invalid_or_expired() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let (_store, svc) = make_service(&tmp, kernel, Arc::new(RecordingBinder::default()));
    let err = svc.complete_login("bogus-state", "any-code").await.unwrap_err();
    assert!(err.to_string().contains("invalid or expired"), "got: {err}");
}

// ── Logout proofs ──────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn logout_is_idempotent_with_no_token() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let (_store, svc) = make_service(&tmp, kernel, Arc::new(RecordingBinder::default()));
    let first = svc.logout().await.unwrap();
    assert!(!first.had_token);
    let second = svc.logout().await.unwrap();
    assert!(!second.had_token);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn logout_clears_local_token_when_gateway_revoke_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let owner = Arc::new(RecordingBinder::default());
    let (store, svc) = make_service_offline(&tmp, kernel, owner.clone());
    store
        .write(AccountToken {
            key: "ship_unrevokable".into(),
            user_id: "user-offline".into(),
            expires_at: 0,
            stored_at: 0,
        })
        .await
        .unwrap();

    let outcome = svc.logout().await.unwrap();

    // Offline-logout invariant: local token cleared even though revoke failed.
    assert!(store.read().await.unwrap().is_none(), "local token must be cleared");
    assert!(outcome.had_token, "had_token records the prior token");
    assert!(!outcome.key_revoked, "key_revoked false when gateway revoke fails");

    // Owner unbind ran for the prior user.
    assert_eq!(owner.unbound.lock().unwrap().as_slice(), ["user-offline"]);

    // account.logout event records had_token=true, key_revoked=false.
    let events = account_events(&tmp, "account.logout");
    let ev = events
        .iter()
        .find(|e| e.entity_id == "user-offline")
        .expect("account.logout for user-offline emitted");
    let payload: serde_json::Value = serde_json::from_str(&ev.payload_json).unwrap();
    assert_eq!(payload["had_token"], serde_json::json!(true));
    assert_eq!(payload["key_revoked"], serde_json::json!(false));
}

// ── Kernel-emit proofs (account.* lands in account.db via the KernelLog port) ──

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn start_login_emits_account_login_started_via_kernel() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let (_store, svc) = make_service(&tmp, kernel, Arc::new(RecordingBinder::default()));
    svc.start_login("http://127.0.0.1:12345/cb").await.unwrap();
    let events = account_events(&tmp, "account.login_started");
    assert!(
        events.iter().any(|e| e.event_type == "account.login_started"),
        "expected account.login_started in account.db, found: {events:?}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn logout_emits_account_logout_via_kernel() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let (_store, svc) = make_service(&tmp, kernel, Arc::new(RecordingBinder::default()));
    svc.logout().await.unwrap();
    let events = account_events(&tmp, "account.logout");
    assert!(
        events.iter().any(|e| e.event_type == "account.logout"),
        "expected account.logout in account.db even on idempotent logout, found: {events:?}",
    );
}

// ── get_key proofs (the SINGLE key-read path) ───────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_key_returns_stored_key() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let (store, svc) = make_service(&tmp, kernel, Arc::new(RecordingBinder::default()));
    store.write(sample_token("user-k")).await.unwrap();
    assert_eq!(svc.get_key().await.unwrap(), "ship_user-k_key");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_key_errors_when_logged_out() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let (_store, svc) = make_service(&tmp, kernel, Arc::new(RecordingBinder::default()));
    assert!(svc.get_key().await.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_key_errors_when_expired() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let (store, svc) = make_service(&tmp, kernel, Arc::new(RecordingBinder::default()));
    store
        .write(AccountToken { key: "k".into(), user_id: "u".into(), expires_at: 1, stored_at: 0 })
        .await
        .unwrap();
    assert!(svc.get_key().await.is_err());
}

// ── Owner-binding via the kernel binder (daemon.owner.* on the kernel path) ────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kernel_owner_binder_emits_daemon_owner_unbound_on_logout() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let owner = Arc::new(KernelOwnerBinder::new(kernel.clone()));
    let (store, svc) = make_service_offline(&tmp, kernel, owner);
    store.write(sample_token("user-bound")).await.unwrap();

    svc.logout().await.unwrap();

    let events = account_events(&tmp, "daemon.owner.unbound");
    assert!(
        events.iter().any(|e| e.entity_id == "user-bound"),
        "expected daemon.owner.unbound for user-bound on the kernel path, found: {events:?}",
    );
}
