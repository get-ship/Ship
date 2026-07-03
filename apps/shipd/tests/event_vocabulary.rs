//! Full-linkage event-vocabulary guards.
//!
//! This integration test links the ENTIRE daemon — every domain crate's
//! `register_event!` lines are present in the inventory registry — so unlike
//! the per-crate guards (ship-substrate sees only L0 registrations, agent-def
//! only L0+its own) these assertions hold over the complete vocabulary with
//! NO exception lists. This is the successor of the retired hand-maintained
//! `ship_types::event_types::ALL` mirror and its justified-absence ratchets.

use std::collections::BTreeSet;

// Force the daemon lib (and so every domain crate's inventory registrations)
// into this test binary's link even though no symbol below names it.
use shipd as _;

use ship_types::event_registry::{self, EventScope, Retention};
use ship_types::event_types::canonical_session::{ALL_CANONICAL_OP, ALL_CANONICAL_SESSION};

/// Pinned count of the COMPLETE registry. The conscious-update forcing
/// function the old `ALL.len()` test provided, now counting registrations
/// (the truth) instead of consts (the mirror). Adding or removing an event
/// anywhere in the workspace must bump this with the change. De-bucketing a
/// domain L0→L1 does NOT change it — the registration moves, the vocabulary
/// doesn't.
#[test]
fn full_registry_count_is_pinned() {
    let n = event_registry::all_names().len();
    assert_eq!(
        n, 105,
        "the full event vocabulary changed size. Bump this pin with your event addition/removal so vocabulary changes stay conscious. (B-wave2 / 2026-06-20: -3 kernel.actor events cut (snapshot, suspended, restored), -1 space.setting.changed cut, -1 actor.mailbox_full cut. 114 - 5 = 109. 2026-07-03: registry had drifted to 108 without a pin bump — re-synced; then -3 dead mesh.* events (message/register/deregister) cut → 105.)"
    );
}

/// Every registration's carried validator is a LIVE wire: it must reject
/// non-JSON for every event in the full vocabulary. Phase 4a made the
/// validator a mandatory registration field; this proves none of them is a
/// pass-through stub.
#[test]
fn every_registered_validator_rejects_garbage() {
    let mut checked = 0;
    for def in event_registry::registry() {
        assert!(
            (def.validate_payload)("][ not json").is_err(),
            "validator for `{}` accepted non-JSON — it is not a live \
             deserializer",
            def.name
        );
        checked += 1;
    }
    assert!(checked >= 105, "expected the full linked registry, got {checked}");
}

/// The domain-owned closed family lists are fully registered at full linkage.
/// (agent-def asserts its own families; this re-checks them in the binary
/// that actually runs, plus the collection family which agent-def can't see.)
#[test]
fn domain_family_slices_are_fully_registered() {
    let registry: BTreeSet<&str> = event_registry::all_names().into_iter().collect();
    for (family, names) in [
        ("canonical session", ALL_CANONICAL_SESSION),
        ("canonical op", ALL_CANONICAL_OP),
        ("collection", ship_collection::events::ALL_COLLECTION),
        ("artifact", ship_artifacts::events::ALL_ARTIFACT),
        ("studio", ship_agents::studio_events::ALL_STUDIO),
    ] {
        for name in names {
            assert!(
                registry.contains(name),
                "{family} family event `{name}` is not registered at full \
                 linkage — its register_event! line was lost"
            );
        }
    }
}

/// Space+Ephemeral pin. Space-scoped events are normally Durable (Space
/// restoration replays them); exactly three watcher-ping events today are
/// Space-scoped yet Ephemeral — AND they drift from their config-family
/// siblings (config/skill/pipeline/rule/collection.changed are
/// Runtime/Ephemeral). Whether the fix is rescoping these to Runtime or
/// ratifying Space pings is the owning lane's call — a scope change is
/// runtime behavior, not a guard edit. Until then this list is a RATCHET:
/// it may only SHRINK, and any NEW Space+Ephemeral event must either be
/// Durable or argue its row here.
#[test]
fn space_ephemeral_events_are_a_pinned_shrinking_set() {
    // The ratchet has shrunk again (job/transient-events): `agent.def.updated`
    // moved from Space+Ephemeral to Space+Durable because the kernel's
    // `route` now honors `Retention` (Ephemeral → deliver only, no persist).
    // The read-only agent-def index and the `ship-agents` watch consumers
    // key off the persisted row, so the event must be Durable. See
    // `.artifacts/transient-events-design.md` for the rationale.
    let mut actual: Vec<&str> = event_registry::registry()
        .filter(|d| {
            d.primary_scope() == EventScope::Space && d.retention == Retention::Ephemeral
        })
        .map(|d| d.name)
        .collect();
    actual.sort_unstable();
    assert_eq!(
        actual,
        ["agent.changed", "view.changed"],
        "the Space+Ephemeral set changed. Space-scoped events should be \
         Durable (restoration replays them) — make the new event Durable, \
         rescope it to Runtime like its config-family siblings, or justify \
         a new row here (this list may only shrink)."
    );
}
