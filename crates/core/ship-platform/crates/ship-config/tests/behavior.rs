//! Behavioral proofs for the unified, scope-discriminated `SettingsService`.
//!
//! The kernel is rooted at the per-test-isolated global dir (`get_global_dir`),
//! so the `settings.changed` events the service authors route — via the kernel's
//! `PartitionedSink` — to the SAME per-scope db files the read-model stores bind
//! (`runtime.db` / `account.db` / `spaces/<id>/space.db`). Each proof asserts the
//! read-model row AND/OR the event landed in the correct scope db.
//!
//! Proofs (the S3 deliverable):
//!  1. per-scope read/write round-trips (SPACE/ACCOUNT/RUNTIME land right);
//!  2. a SPACE setting is isolated to its space's db;
//!  3. `Set(SPACE)` by a non-operator is REJECTED (authority gate);
//!  4. `Set` authors a `settings.changed` event the projection reflects.

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use ship_config::service::{SettingScope, SettingsCtx, SettingsService};
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_substrate::ports::{DataStore, KernelLog, SingleBackendDataStore};

// ── Fixtures ──────────────────────────────────────────────────────────────────

/// The per-test-isolated global dir — the SAME root the kernel + every read-model
/// store bind, so the event log and the read model share each per-scope db.
fn global_dir() -> std::path::PathBuf {
    ship_substrate::project::get_global_dir().unwrap()
}

/// A kernel rooted at the global dir, so its `PartitionedSink` writes to
/// `<global>/{runtime,account,spaces/<id>/space}.db`.
fn make_kernel() -> Arc<KernelLog> {
    let root = global_dir();
    std::fs::create_dir_all(&root).unwrap();
    Arc::new(KernelLog::new(new_isolated_for_test(root).unwrap()))
}

/// A `DataStore` over a per-scope db under the global dir, bound to `partition`.
fn store_at(db_path: &std::path::Path, partition: &str) -> Arc<dyn DataStore> {
    let svc = DataService::open_at(db_path).unwrap();
    Arc::new(SingleBackendDataStore::new(svc, partition))
}

fn account_store() -> Arc<dyn DataStore> {
    store_at(&ship_substrate::db::account_db_path(&global_dir()), "account")
}

fn space_store(space_id: &str) -> Arc<dyn DataStore> {
    store_at(&ship_substrate::db::space_db_path(&global_dir(), space_id), space_id)
}

/// A `SettingsCtx` with a live active space and the given operator verdict.
fn ctx_with_space(kernel: Arc<KernelLog>, space_id: &str, is_operator: bool) -> SettingsCtx {
    SettingsCtx {
        kernel,
        account_store: account_store(),
        space: Some((space_id.to_string(), space_store(space_id))),
        is_operator,
    }
}

/// A `SettingsCtx` with NO active space.
fn ctx_no_space(kernel: Arc<KernelLog>, is_operator: bool) -> SettingsCtx {
    SettingsCtx { kernel, account_store: account_store(), space: None, is_operator }
}

/// True iff `id` is present in the per-scope db at `path`. A missing file (no
/// event routed there) reads as absent.
fn db_has_event(path: &std::path::Path, id: &str) -> bool {
    if !path.exists() {
        return false;
    }
    SqliteEventStore::new_at_db_path(path).unwrap().get(id).unwrap().is_some()
}

// ── Proof 1: per-scope read/write round-trips land in the right db ────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn runtime_scope_round_trips_through_daemon_settings() {
    let kernel = make_kernel();
    let svc = SettingsService::new(ctx_no_space(kernel, true));

    let set = svc.set(SettingScope::Runtime, "terminal.program", json!("wt")).await.unwrap();
    assert_eq!(set.scope, SettingScope::Runtime);
    assert!(!set.is_default, "a set value is never a default");

    let got = svc.get(SettingScope::Runtime, "terminal.program").await.unwrap();
    assert_eq!(got.value, json!("wt"));
    assert_eq!(got.scope, SettingScope::Runtime);
    assert!(!got.is_default, "stored row → not default");

    // An unset infra key falls back to its hardcoded default.
    let dflt = svc.get(SettingScope::Runtime, "cmdk.send_key").await.unwrap();
    assert_eq!(dflt.value, json!("enter"));
    assert!(dflt.is_default);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn account_scope_round_trips_through_account_db() {
    let kernel = make_kernel();
    let svc = SettingsService::new(ctx_no_space(kernel, true));

    svc.set(SettingScope::Account, "acct.rt.theme", json!("dark")).await.unwrap();
    let got = svc.get(SettingScope::Account, "acct.rt.theme").await.unwrap();
    assert_eq!(got.value, json!("dark"));
    assert_eq!(got.scope, SettingScope::Account);

    // The row landed in the per-account read-model store specifically.
    let stored = ship_config::overlay::get(account_store().as_ref(), "acct.rt.theme").await.unwrap();
    assert_eq!(stored, Some(json!("dark")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn space_scope_round_trips_through_space_db() {
    let kernel = make_kernel();
    let svc = SettingsService::new(ctx_with_space(kernel, "space-rt", true));

    svc.set(SettingScope::Space, "editor.font", json!("mono")).await.unwrap();
    let got = svc.get(SettingScope::Space, "editor.font").await.unwrap();
    assert_eq!(got.value, json!("mono"));
    assert_eq!(got.scope, SettingScope::Space);

    // The row landed in THIS space's read-model store.
    let stored = ship_config::overlay::get(space_store("space-rt").as_ref(), "editor.font").await.unwrap();
    assert_eq!(stored, Some(json!("mono")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn space_scope_with_no_active_space_errors_cleanly() {
    let kernel = make_kernel();
    let svc = SettingsService::new(ctx_no_space(kernel, true));

    // Operator, but no active space ⟹ a clean NotFound (no `global` fallback).
    let err = svc.set(SettingScope::Space, "editor.font", json!("mono")).await.unwrap_err();
    assert!(matches!(err, ship_config::error::ConfigError::NotFound(_)));
    let err = svc.get(SettingScope::Space, "editor.font").await.unwrap_err();
    assert!(matches!(err, ship_config::error::ConfigError::NotFound(_)));
}

// ── Proof 2: a SPACE setting is isolated to its space's db ────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn space_setting_is_isolated_to_its_space() {
    let kernel = make_kernel();
    let svc_a = SettingsService::new(ctx_with_space(kernel.clone(), "iso-a", true));
    let svc_b = SettingsService::new(ctx_with_space(kernel, "iso-b", true));

    svc_a.set(SettingScope::Space, "iso.font", json!("iso-a-value")).await.unwrap();

    // iso-a sees it.
    assert_eq!(
        svc_a.get(SettingScope::Space, "iso.font").await.unwrap().value,
        json!("iso-a-value"),
    );
    // iso-b does NOT — the override lives in iso-a's partition only.
    let err = svc_b.get(SettingScope::Space, "iso.font").await.unwrap_err();
    assert!(matches!(err, ship_config::error::ConfigError::NotFound(_)));

    // Directly: iso-b's read-model store has no such row.
    let in_b = ship_config::overlay::get(space_store("iso-b").as_ref(), "iso.font").await.unwrap();
    assert!(in_b.is_none(), "iso-b partition is untouched");
}

// ── Proof 3: Set(SPACE) by a non-operator is REJECTED (authority gate) ────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn set_space_by_non_operator_is_rejected() {
    let kernel = make_kernel();
    let svc = SettingsService::new(ctx_with_space(kernel.clone(), "reject-sp", false));
    let mut rx = kernel.subscribe();

    let err = svc.set(SettingScope::Space, "reject.font", json!("nope")).await.unwrap_err();
    assert!(
        matches!(err, ship_config::error::ConfigError::Forbidden(_)),
        "non-operator SPACE write is forbidden, got {err:?}",
    );

    // Nothing persisted, nothing emitted.
    let in_space = ship_config::overlay::get(space_store("reject-sp").as_ref(), "reject.font").await.unwrap();
    assert!(in_space.is_none(), "rejected write persists nothing");
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(rx.try_recv().is_err(), "rejected write emits no event");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn non_operator_account_write_is_also_rejected_under_restrictive_default() {
    // The restrictive default: ALL non-operator writes are rejected (the
    // owner/contributor + agent-subset role model is a flagged follow).
    let kernel = make_kernel();
    let svc = SettingsService::new(ctx_no_space(kernel, false));
    let err = svc.set(SettingScope::Account, "acct.reject.theme", json!("dark")).await.unwrap_err();
    assert!(matches!(err, ship_config::error::ConfigError::Forbidden(_)));
}

// ── Proof 4: Set authors a settings.changed event the projection reflects ─────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn set_authors_settings_changed_in_the_chosen_scope() {
    let kernel = make_kernel();
    let mut rx = kernel.subscribe();
    let svc = SettingsService::new(ctx_with_space(kernel.clone(), "evt-sp", true));

    svc.set(SettingScope::Space, "evt.font", json!("mono")).await.unwrap();

    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("timeout waiting for settings.changed")
        .expect("channel closed");
    assert_eq!(event.event_type, "settings.changed", "exactly settings.changed");
    assert_eq!(event.entity_id, "evt.font", "entity_id = key");
    // The chosen scope rode the envelope. (The authoritative `scope_key` is
    // stamped onto the PERSISTED copy inside the router's `enforce_scope` — the
    // broadcast copy here predates that stamp, mirroring the legacy config test —
    // so the space-routing proof is the durable read below.)
    assert_eq!(event.scope, ship_types::event_registry::EventScope::Space);
    let payload: serde_json::Value = serde_json::from_str(&event.payload_json).unwrap();
    assert_eq!(payload["key"], json!("evt.font"));
    assert_eq!(payload["scope"], json!("space"), "payload echoes the scope");

    // The event is DURABLE in this space's OWN db — which is the proof that the
    // scope + active-space key routed it correctly (the PartitionedSink keys the
    // space db path off the stamped `(Space, scope_key)`). The persisted copy
    // carries the stamped key.
    let space_db = ship_substrate::db::space_db_path(&global_dir(), "evt-sp");
    let persisted = SqliteEventStore::new_at_db_path(&space_db).unwrap().get(&event.id).unwrap();
    let persisted = persisted.expect("event durable in evt-sp's space.db");
    assert_eq!(persisted.scope, ship_types::event_registry::EventScope::Space);
    assert_eq!(persisted.scope_key.as_deref(), Some("evt-sp"), "stamped scope_key = active space");
    // It is NOT in runtime.db or account.db (scope routing).
    assert!(!db_has_event(&ship_substrate::db::runtime_db_path(&global_dir()), &event.id));
    assert!(!db_has_event(&ship_substrate::db::account_db_path(&global_dir()), &event.id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn account_set_event_routes_to_account_db() {
    let kernel = make_kernel();
    let mut rx = kernel.subscribe();
    let svc = SettingsService::new(ctx_no_space(kernel.clone(), true));

    svc.set(SettingScope::Account, "acct.evt.theme", json!("dark")).await.unwrap();

    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("timeout")
        .expect("closed");
    assert_eq!(event.event_type, "settings.changed");
    assert_eq!(event.scope, ship_types::event_registry::EventScope::Account);

    let account_db = ship_substrate::db::account_db_path(&global_dir());
    assert!(db_has_event(&account_db, &event.id), "event durable in account.db");
    assert!(!db_has_event(&ship_substrate::db::runtime_db_path(&global_dir()), &event.id));
}

// ── Watch: the reactive surface ───────────────────────────────────────────────

/// The wire's `Watch` RPC returns a live stream whose FIRST frame is the initial
/// scope snapshot.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_returns_a_live_stream_with_initial_snapshot() {
    use connectrpc::RequestContext;
    use futures_util::StreamExt;
    use ship_config::SettingsWire;
    use ship_ops_rpc::{
        OwnedWatchSettingsReqView, SettingScope as ProtoScope, SettingsService as _,
        WatchSettingsReq,
    };

    let kernel = make_kernel();
    let kernel_for_ctx = kernel.clone();
    let wire = SettingsWire::new(kernel.clone(), move |_rctx: &RequestContext| {
        ctx_no_space(kernel_for_ctx.clone(), true)
    });

    let rctx = RequestContext::default();
    let req = OwnedWatchSettingsReqView::from_owned(&WatchSettingsReq {
        scope: buffa::EnumValue::from(ProtoScope::SETTING_SCOPE_RUNTIME),
        ..Default::default()
    })
    .unwrap();
    let resp = wire.watch(rctx, req).await.expect("watch stream");
    let mut stream = resp.body;

    let first = tokio::time::timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("timeout on initial frame")
        .expect("stream ended")
        .expect("stream error");
    assert!(first.body.is_some(), "initial frame carries a patch body");
}
