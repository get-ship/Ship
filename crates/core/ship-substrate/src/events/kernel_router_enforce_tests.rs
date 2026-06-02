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
use ship_types::{Authority, ScopeBinding, ScopeViolation};

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

// ── Allowed-set (Model C) — these run in ship-substrate so the inventory
//    registry (payload_registry.rs) is linked and `allowed_scopes(name)` is
//    populated. The pure `enforce_scope` fn is exercised directly here. ──────

use ship_types::enforce_scope;
use ship_types::event_registry::allowed_scopes;

/// A REGISTERED Space-singleton event: the chosen scope == primary == only
/// allowed element → membership passes, key stamped. Behavior-preserving proof
/// for the size-1 set.
#[test]
fn allowed_set_chosen_in_set_and_authorized_passes() {
    assert_eq!(
        allowed_scopes("artifact.created"),
        &[EventScope::Space],
        "artifact.created must be a Space singleton in Phase A"
    );
    let mut e = EventEnvelope::new("artifact.created", "art-1", &serde_json::json!({}))
        .expect("envelope");
    // new() stamped scope=Space (the primary). Space-bound binding authorizes it.
    assert_eq!(e.scope, EventScope::Space);
    enforce_scope(&mut e, &space_binding("space-A")).expect("chosen∈set + authorized");
    assert_eq!(e.scope_key.as_deref(), Some("space-A"));
}

/// Chosen scope ∉ the type's allowed-set → `ScopeNotAllowed`, regardless of
/// authorization. `artifact.created` only allows Space; choosing System rejects.
#[test]
fn allowed_set_chosen_not_in_set_is_rejected() {
    let mut e = EventEnvelope::new("artifact.created", "art-1", &serde_json::json!({}))
        .expect("envelope")
        .with_scope(EventScope::System, None);
    let err = enforce_scope(&mut e, &space_binding("space-A")).unwrap_err();
    assert_eq!(
        err,
        ScopeViolation::ScopeNotAllowed {
            event_type: "artifact.created".to_string(),
            chosen: EventScope::System,
            allowed: vec![EventScope::Space],
        }
    );
}

/// Chosen scope ∈ set but the binding is NOT authorized for it → the existing
/// per-scope authorization still rejects. A Space-allowed event from a
/// space-unbound binding is `NoSpaceForSpaceEvent`.
#[test]
fn allowed_set_chosen_in_set_but_unauthorized_is_rejected() {
    let mut e = EventEnvelope::new("artifact.created", "art-1", &serde_json::json!({}))
        .expect("envelope");
    assert_eq!(e.scope, EventScope::Space);
    let err = enforce_scope(&mut e, &spaceless_binding()).unwrap_err();
    assert_eq!(err, ScopeViolation::NoSpaceForSpaceEvent);
}

/// Unregistered (ad-hoc / test) event types carry an EMPTY allowed-set — the
/// membership check is skipped so they are never scope-rejected on set grounds.
#[test]
fn allowed_set_unregistered_event_skips_membership() {
    assert!(
        allowed_scopes("test.adhoc").is_empty(),
        "unregistered events have no allowed-set"
    );
    let mut e = EventEnvelope::new("test.adhoc", "x", &serde_json::json!({}))
        .expect("envelope")
        .with_scope(EventScope::System, None);
    // System chosen, no declared restriction → passes (key None).
    enforce_scope(&mut e, &space_binding("space-A")).expect("unregistered: no membership reject");
    assert_eq!(e.scope_key, None);
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
