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
        n, 80,
        "the full event vocabulary changed size. Bump this pin with your event addition/removal so vocabulary changes stay conscious. (B-wave2 / 2026-06-20: -3 kernel.actor events cut (snapshot, suspended, restored), -1 space.setting.changed cut, -1 actor.mailbox_full cut. 114 - 5 = 109. 2026-07-03: registry had drifted to 108 without a pin bump — re-synced; then -3 dead mesh.* events (message/register/deregister) cut → 105. 2026-07-10 space cutover: -3 project.* events (registered/seen/switched) cut, +2 space.* (opened/seen) added → 104. 2026-07-10 engine cut: -15 engine-emitted pipeline.* registrations + -4 supervisor.* registrations → 85. 2026-07-12 stack membership: +1 stack.registered → 86. 2026-07-13 studio cut: -4 studio.* events → 82; then +2 drift re-synced to 84. 2026-07-15 library collapse: -2 stack.installed/stack.unregistered events cut → 82. 2026-07-17 skill cleanup: -2 skill.started/skill.completed events cut → 80. Future bumps land here with a dated note.)"
    );
}

/// Windowed pin — the Windowed(TTL) tier is a CLOSED, ruled list. A type in
/// this tier is persisted + projection-folded like Durable and swept past
/// its TTL, so every projection folding it must be correct from the
/// surviving window alone (see ship-platform/tests/retention_replay.rs).
/// Adding a type here means accepting that bound for all of its
/// projections — argue the row with the change, don't just append it.
/// (Windowed tier introduced 2026-07-18: op.tool_use_* ×3,
/// session.turn.* ×2, dispatch.requested, sync.push.completed at 7d;
/// actor.stopped and account.key_refreshed left the Ephemeral tier for
/// Durable in the same re-triage.)
#[test]
fn windowed_events_are_the_ruled_set() {
    let mut actual: Vec<(&str, u16)> = event_registry::registry()
        .filter_map(|d| d.retention.ttl_days().map(|ttl| (d.name, ttl)))
        .collect();
    actual.sort_unstable();
    assert_eq!(
        actual,
        [
            ("dispatch.requested", 7),
            ("op.tool_use_completed", 7),
            ("op.tool_use_denied", 7),
            ("op.tool_use_started", 7),
            ("session.turn.ended", 7),
            ("session.turn.started", 7),
            ("sync.push.completed", 7),
        ],
        "the Windowed retention set changed. This list is ruled: a new \
         Windowed type must have projections that fold correctly from a \
         swept log (bounded by its TTL horizon) — verify, then update this \
         pin with a dated note."
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
    assert!(checked >= 80, "expected the full linked registry, got {checked}");
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
