//! Behavior spec for `enforce_scope` — the RATIFIED stamp-or-reject rule.

use ship_policy::{enforce_scope, Authority, ScopeBinding, ScopeViolation};
use ship_types::event_registry::EventScope;
use ship_types::{AccountId, DeviceId, EventEnvelope, SpaceId};

fn env(scope: EventScope, key: Option<&str>) -> EventEnvelope {
    EventEnvelope::new("test.event", "entity-1", &())
        .expect("envelope")
        .with_scope(scope, key)
}

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

fn spaceless_binding() -> ScopeBinding {
    ScopeBinding::from_session(
        DeviceId::new("dev-1"),
        Some(AccountId::new("acct-1")),
        None,
        Authority::Sdk,
    )
}

#[test]
fn space_event_absent_key_is_stamped_from_binding() {
    let mut e = env(EventScope::Space, None);
    enforce_scope(&mut e, &space_binding("space-A")).expect("stamp");
    assert_eq!(e.scope_key.as_deref(), Some("space-A"));
}

#[test]
fn space_event_matching_key_passes_untouched() {
    let mut e = env(EventScope::Space, Some("space-A"));
    enforce_scope(&mut e, &space_binding("space-A")).expect("pass");
    assert_eq!(e.scope_key.as_deref(), Some("space-A"));
}

#[test]
fn space_event_mismatched_key_is_cross_space_reject() {
    let mut e = env(EventScope::Space, Some("space-B"));
    let err = enforce_scope(&mut e, &space_binding("space-A")).unwrap_err();
    assert_eq!(
        err,
        ScopeViolation::CrossSpace {
            expected: "space-A".to_string(),
            got: "space-B".to_string(),
        }
    );
}

#[test]
fn spaceless_caller_emitting_space_event_is_hard_reject() {
    let mut e = env(EventScope::Space, None);
    let err = enforce_scope(&mut e, &spaceless_binding()).unwrap_err();
    assert_eq!(err, ScopeViolation::NoSpaceForSpaceEvent);
}

#[test]
fn account_event_is_stamped_with_account_key() {
    let mut e = env(EventScope::Account, None);
    enforce_scope(&mut e, &space_binding("space-A")).expect("stamp");
    assert_eq!(e.scope_key.as_deref(), Some("acct-1"));
}

#[test]
fn runtime_event_is_stamped_with_machine_key() {
    let mut e = env(EventScope::Runtime, None);
    enforce_scope(&mut e, &space_binding("space-A")).expect("stamp");
    assert_eq!(e.scope_key.as_deref(), Some("dev-1"));
}

#[test]
fn runtime_event_key_is_machine_id_even_with_leftover() {
    // A leftover key on a Runtime event is overwritten by the binding's machine id.
    let mut e = env(EventScope::Runtime, Some("leftover"));
    enforce_scope(&mut e, &space_binding("space-A")).expect("stamp");
    assert_eq!(e.scope_key.as_deref(), Some("dev-1"));
}

#[test]
fn key_for_returns_axis_keys() {
    let b = space_binding("space-A");
    assert_eq!(b.key_for(EventScope::Space).as_deref(), Some("space-A"));
    assert_eq!(b.key_for(EventScope::Account).as_deref(), Some("acct-1"));
    assert_eq!(b.key_for(EventScope::Runtime).as_deref(), Some("dev-1"));
    assert_eq!(spaceless_binding().key_for(EventScope::Space), None);
}
