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

/// Read the persisted envelope back from the per-scope db it routes to. The
/// kernel sink partitions by scope, so the reader names the scope (and, for
/// Space, the space key) to resolve the right db. A db that was never opened
/// (rejected event, or no event of that scope) reads back as `None`.
fn read_back_scope(
    tmp: &tempfile::TempDir,
    id: &str,
    scope: EventScope,
    space_key: Option<&str>,
) -> Option<EventEnvelope> {
    let root = tmp.path().join(".ship");
    let path = match scope {
        EventScope::Runtime => crate::db::runtime_db_path(&root),
        EventScope::Account => crate::db::account_db_path(&root),
        EventScope::Space => crate::db::space_db_path(&root, space_key.unwrap_or("space-A")),
    };
    if !path.exists() {
        return None;
    }
    SqliteEventStore::new_at_db_path(&path).unwrap().get(id).unwrap()
}

/// Convenience: read a Space event back from space "space-A"'s db.
fn read_back(tmp: &tempfile::TempDir, id: &str) -> Option<EventEnvelope> {
    read_back_scope(tmp, id, EventScope::Space, Some("space-A"))
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
    let mut mb = router.spawn_actor("studio", studio_config()).unwrap();

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

/// A REGISTERED Account-singleton event: the chosen scope == primary == only
/// allowed element → membership passes, key stamped. Behavior-preserving proof
/// for the size-1 set.
/// Uses `daemon.owner.bound` — an Account-singleton that is L0-registered.
/// (studio.annotation.created, previously used here, moved to ship-agents L1
/// in keystone-A de-bucket 2026-06-10; no Space-only events remain at L0.)
#[test]
fn allowed_set_chosen_in_set_and_authorized_passes() {
    assert_eq!(
        allowed_scopes("daemon.owner.bound"),
        &[EventScope::Account],
        "daemon.owner.bound must be an Account singleton"
    );
    let mut e = EventEnvelope::new("daemon.owner.bound", "dev-1", &serde_json::json!({}))
        .expect("envelope");
    // new() stamped scope=Account (the primary). spaceless_binding carries
    // account_id "acct-1" — sufficient for Account scope.
    assert_eq!(e.scope, EventScope::Account);
    enforce_scope(&mut e, &spaceless_binding()).expect("chosen∈set + authorized");
    assert_eq!(e.scope_key.as_deref(), Some("acct-1"));
}

/// Chosen scope ∉ the type's allowed-set → `ScopeNotAllowed`, regardless of
/// authorization. `daemon.owner.bound` only allows Account; choosing Space rejects.
#[test]
fn allowed_set_chosen_not_in_set_is_rejected() {
    let mut e = EventEnvelope::new("daemon.owner.bound", "dev-1", &serde_json::json!({}))
        .expect("envelope")
        .with_scope(EventScope::Space, None);
    let err = enforce_scope(&mut e, &space_binding("space-A")).unwrap_err();
    assert_eq!(
        err,
        ScopeViolation::ScopeNotAllowed {
            event_type: "daemon.owner.bound".to_string(),
            chosen: EventScope::Space,
            allowed: vec![EventScope::Account],
        }
    );
}

/// Chosen scope ∈ set but the binding is NOT authorized for it → the existing
/// per-scope authorization still rejects. `settings.changed` is poly-scoped
/// (allows Runtime/Account/Space); choosing Space from a space-unbound binding
/// is still `NoSpaceForSpaceEvent` — the allowed-set check passes, the binding
/// authorization check fails. This proves Model C and per-scope auth are
/// independent guards.
#[test]
fn allowed_set_chosen_in_set_but_unauthorized_is_rejected() {
    // settings.changed allows [Runtime, Account, Space] — Space is in the set.
    assert!(
        allowed_scopes("settings.changed").contains(&EventScope::Space),
        "settings.changed must allow Space scope"
    );
    let mut e = EventEnvelope::new("settings.changed", "proj-1", &serde_json::json!({}))
        .expect("envelope")
        .with_scope(EventScope::Space, None);
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
        .with_scope(EventScope::Runtime, None);
    // Runtime chosen, no declared restriction → passes; key stamped from the
    // binding's device/machine id.
    enforce_scope(&mut e, &space_binding("space-A")).expect("unregistered: no membership reject");
    assert_eq!(e.scope_key.as_deref(), Some("dev-1"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emit_stamps_non_space_keys_without_cross_check() {
    let (tmp, router) = setup();
    let binding = space_binding("space-A");

    // Account → account key.
    let acct = ev("studio.canvas_updated").with_scope(EventScope::Account, None);
    let acct_id = acct.id.clone();
    router.emit(acct, &ctx_with(binding.clone())).await.unwrap();
    let p = read_back_scope(&tmp, &acct_id, EventScope::Account, None).unwrap();
    assert_eq!(p.scope_key.as_deref(), Some("acct-1"));

    // Runtime → machine/device key, no cross-check.
    let rt = ev("studio.canvas_updated").with_scope(EventScope::Runtime, None);
    let rt_id = rt.id.clone();
    router.emit(rt, &ctx_with(binding)).await.unwrap();
    let p = read_back_scope(&tmp, &rt_id, EventScope::Runtime, None).unwrap();
    assert_eq!(p.scope_key.as_deref(), Some("dev-1"));
    assert_eq!(p.author_device_id.as_deref(), Some("dev-1"));
}
