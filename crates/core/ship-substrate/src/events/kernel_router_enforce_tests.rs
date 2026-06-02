//! D-enforce: emit chokepoint stamps + rejects from the binding.
//!
//! These go THROUGH `KernelRouter::emit(...).await` (not the pure `enforce_scope`
//! fn) and assert the PERSISTED envelope, proving the wiring — not just the rule.
//! Shared fixtures (`setup`/`ev`/`studio_config`) are borrowed from the sibling
//! `kernel_router_tests` module.

use ship_types::envelope::EventEnvelope;
use crate::events::validator::{CallerKind, EmitContext};
use crate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_types::event_registry::EventScope;
use ship_types::identity::{AccountId, DeviceId, SpaceId};
use ship_types::{Authority, ScopeBinding};

use super::kernel_router_tests::{ev, setup, studio_config};

/// A session binding bound to `space`, carrying device + account identity.
fn space_binding(space: &str) -> ScopeBinding {
    ScopeBinding::from_session(
        DeviceId::new("dev-1"),
        Some(AccountId::new("acct-1")),
        Some(SpaceId::new(space)),
        Authority::Space {
            space_id: SpaceId::new(space),
        },
    )
}

/// A space-unbound session binding (device + account, no space).
fn spaceless_binding() -> ScopeBinding {
    ScopeBinding::from_session(
        DeviceId::new("dev-1"),
        Some(AccountId::new("acct-1")),
        None,
        Authority::Sdk,
    )
}

fn ctx_with(binding: ScopeBinding) -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding,
        session_id: None,
    }
}

/// Read the persisted envelope back from platform.db at the router's base dir.
fn read_back(tmp: &tempfile::TempDir, id: &str) -> Option<EventEnvelope> {
    let store = SqliteEventStore::new_at(&crate::ShipHome::project_ship_dir(tmp.path())).unwrap();
    store.get(id).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emit_stamps_space_key_and_author_from_binding() {
    let (tmp, router) = setup();
    // Space event with NO scope_key, emitted under a space-bound binding.
    let event = ev("studio.canvas_updated").with_scope(EventScope::Space, None);
    let id = event.id.clone();
    router
        .emit(event, &ctx_with(space_binding("space-A")))
        .await
        .unwrap();

    let persisted = read_back(&tmp, &id).expect("event must be persisted");
    assert_eq!(persisted.scope, EventScope::Space);
    assert_eq!(persisted.scope_key.as_deref(), Some("space-A"));
    assert_eq!(persisted.author_device_id.as_deref(), Some("dev-1"));
    assert_eq!(persisted.author_account_id.as_deref(), Some("acct-1"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emit_rejects_cross_space_and_does_not_persist() {
    let (tmp, mut router) = setup();
    // Subscribe an actor so we can also assert non-delivery.
    let (_store, mut mb) = router.spawn_actor("studio", studio_config()).unwrap();

    // Space event keyed to space-B, but binding is space-A → CrossSpace reject.
    let event = ev("studio.canvas_updated").with_scope(EventScope::Space, Some("space-B"));
    let id = event.id.clone();
    let err = router
        .emit(event, &ctx_with(space_binding("space-A")))
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("cross-space"),
        "unexpected error: {err}"
    );

    assert!(read_back(&tmp, &id).is_none(), "rejected event must not persist");
    assert!(mb.try_recv().is_err(), "rejected event must not be delivered");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emit_rejects_space_event_from_space_unbound_binding() {
    let (tmp, router) = setup();
    let event = ev("studio.canvas_updated").with_scope(EventScope::Space, None);
    let id = event.id.clone();
    let err = router
        .emit(event, &ctx_with(spaceless_binding()))
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("space-unbound"),
        "unexpected error: {err}"
    );
    assert!(read_back(&tmp, &id).is_none(), "rejected event must not persist");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emit_stamps_non_space_keys_without_cross_check() {
    let (tmp, router) = setup();
    let binding = space_binding("space-A");

    // Account → account key.
    let acct = ev("studio.canvas_updated").with_scope(EventScope::Account, None);
    let acct_id = acct.id.clone();
    router.emit(acct, &ctx_with(binding.clone())).await.unwrap();
    let p = read_back(&tmp, &acct_id).unwrap();
    assert_eq!(p.scope_key.as_deref(), Some("acct-1"));

    // Device → device key.
    let dev = ev("studio.canvas_updated").with_scope(EventScope::Device, None);
    let dev_id = dev.id.clone();
    router.emit(dev, &ctx_with(binding.clone())).await.unwrap();
    let p = read_back(&tmp, &dev_id).unwrap();
    assert_eq!(p.scope_key.as_deref(), Some("dev-1"));

    // System → None, no cross-check.
    let sys = ev("studio.canvas_updated").with_scope(EventScope::System, None);
    let sys_id = sys.id.clone();
    router.emit(sys, &ctx_with(binding)).await.unwrap();
    let p = read_back(&tmp, &sys_id).unwrap();
    assert_eq!(p.scope_key, None);
    assert_eq!(p.author_device_id.as_deref(), Some("dev-1"));
}
