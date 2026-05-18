use super::*;
use crate::account::store::FileTokenStore;
use crate::account::types::AccountToken;
use tempfile::TempDir;

fn make_service() -> (TempDir, AccountService) {
    let tmp = TempDir::new().unwrap();
    let store = Arc::new(FileTokenStore::for_daemon_dir(tmp.path()));
    let svc = AccountService::new(store, "https://api.getship.dev").unwrap();
    (tmp, svc)
}

/// Service pointed at an unroutable address so revoke_key fails fast.
/// Used to drive the offline-logout fallback path without depending on
/// real network reachability.
fn make_service_offline() -> (TempDir, Arc<FileTokenStore>, AccountService) {
    let tmp = TempDir::new().unwrap();
    let store = Arc::new(FileTokenStore::for_daemon_dir(tmp.path()));
    let svc = AccountService::new(store.clone(), "http://127.0.0.1:1").unwrap();
    (tmp, store, svc)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn state_is_logged_out_before_login() {
    let (_tmp, svc) = make_service();
    matches!(svc.state().await.unwrap(), AccountState::LoggedOut);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn start_login_registers_pending_and_returns_browser_url() {
    let (_tmp, svc) = make_service();
    let resp = svc
        .start_login("http://127.0.0.1:54321/callback")
        .await
        .unwrap();
    assert!(resp.browser_url.starts_with("https://api.getship.dev/v1/auth/cli?"));
    assert!(resp.browser_url.contains("code_challenge="));
    assert!(resp.browser_url.contains(&format!(
        "state={}",
        urlencoding::encode(&resp.state_id)
    )));
    // pending state recorded
    assert_eq!(svc.pending.lock().await.len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn complete_login_with_unknown_state_errors() {
    let (_tmp, svc) = make_service();
    let err = svc.complete_login("bogus-state", "any-code").await;
    assert!(err.is_err());
    assert!(err.unwrap_err().to_string().contains("invalid or expired"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn logout_is_idempotent() {
    let (_tmp, svc) = make_service();
    svc.logout().await.unwrap();
    svc.logout().await.unwrap();
}

fn query_event_types(types: &[&str]) -> Vec<String> {
    use ship_substrate::events::EventFilter;
    use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
    let store = SqliteEventStore::new().expect("event store");
    let mut found = Vec::new();
    for t in types {
        let events = store
            .query(&EventFilter {
                event_type: Some((*t).to_string()),
                ..Default::default()
            })
            .expect("query events");
        for ev in events {
            found.push(ev.event_type);
        }
    }
    found
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn start_login_emits_account_login_started() {
    let (_tmp, svc) = make_service();
    svc.start_login("http://127.0.0.1:12345/cb")
        .await
        .unwrap();
    let found = query_event_types(&["account.login_started"]);
    assert!(
        found.iter().any(|t| t == "account.login_started"),
        "expected account.login_started in log, found: {found:?}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn logout_emits_account_logout() {
    let (_tmp, svc) = make_service();
    svc.logout().await.unwrap();
    let found = query_event_types(&["account.logout"]);
    assert!(
        found.iter().any(|t| t == "account.logout"),
        "expected account.logout in log even on idempotent logout, found: {found:?}",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn logout_clears_local_token_when_gateway_revoke_fails() {
    // Token stored, gateway unreachable. Logout must still clear the
    // local file (offline-logout invariant) and emit `account.logout`
    // with key_revoked=false so audit can distinguish "logged out +
    // revoked" from "logged out (local only)".
    let (_tmp, store, svc) = make_service_offline();
    store
        .write(AccountToken {
            key: "ship_unrevokable".into(),
            user_id: "user-offline".into(),
            expires_at: 0,
            stored_at: 0,
        })
        .await
        .unwrap();

    svc.logout().await.unwrap();

    assert!(
        store.read().await.unwrap().is_none(),
        "local token should be cleared even when gateway revoke fails",
    );

    use ship_substrate::events::EventFilter;
    use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
    let events = SqliteEventStore::new()
        .unwrap()
        .query(&EventFilter {
            event_type: Some("account.logout".into()),
            entity_id: Some("user-offline".into()),
            ..Default::default()
        })
        .unwrap();
    let ev = events
        .iter()
        .find(|e| e.entity_id == "user-offline")
        .expect("account.logout for user-offline emitted");
    let payload: serde_json::Value = serde_json::from_str(&ev.payload_json).unwrap();
    assert_eq!(payload["had_token"], serde_json::json!(true));
    assert_eq!(payload["key_revoked"], serde_json::json!(false));
}
