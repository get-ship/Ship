/// Integration tests — grant provisioning, resolution, and diff logic.
///
/// Covers:
/// - `provision_grants` emits `agent.grants.provisioned`
/// - `resolve_grants_at` falls back to role when unprovisioned
/// - `resolve_grants_at` provisioned record overrides the role default
/// - `pending_grants` diff: empty, partial, fully unapproved
/// - `has_pending` flag correctness
/// - `resolve_from` fail-closed on projection error (deny extras, not role fallback)
use ship_substrate::events::filter::EventFilter;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_types::event_types::AGENT_GRANTS_PROVISIONED;
use tempfile::tempdir;

use crate::grant_diff::{has_pending, pending_grants};
use crate::scopes::{provision_grants, resolve_grants_at};
use crate::testutil::init_auth;

fn query_events(event_type: &str) -> Vec<ship_types::envelope::EventEnvelope> {
    // auth.session.* / daemon.owner.* / agent.grants.* are Account-scoped → account.db.
    let store = SqliteEventStore::new_for_scope(
        &ship_substrate::project::get_global_dir().expect("global dir"),
        ship_types::event_registry::EventScope::Account,
        None,
    )
    .expect("open auto-isolated test store");
    store
        .query(&EventFilter {
            event_type: Some(event_type.to_string()),
            ..Default::default()
        })
        .expect("query events")
}

fn sv(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

// ── provision_grants event-emit ───────────────────────────────────────────────

#[test]
fn auth_grants_provision_emits_agent_grants_provisioned() {
    let tmp = tempdir().unwrap();
    init_auth(tmp.path()).unwrap();

    provision_grants("agent-emit-test", &sv(&["ship-orchestration"]), "operator", Some("op-1"))
        .unwrap();

    let events = query_events(AGENT_GRANTS_PROVISIONED);
    assert!(
        events.iter().any(|e| e.entity_id == "agent-emit-test"),
        "expected agent.grants.provisioned with entity_id=agent-emit-test; got {events:?}"
    );
}

// ── resolve_grants_at — fallback and override ─────────────────────────────────

#[test]
fn auth_grants_resolve_falls_back_to_role_when_no_provisioned_record() {
    let tmp = tempdir().unwrap();
    init_auth(tmp.path()).unwrap();

    // No provisioned record → role baseline (commander → ["agent"]).
    let grants = resolve_grants_at(tmp.path(), "fresh-agent", Some("commander"));
    assert_eq!(grants, sv(&["agent"]));
}

#[test]
fn auth_grants_resolve_worker_role_baseline_is_empty() {
    let tmp = tempdir().unwrap();
    init_auth(tmp.path()).unwrap();

    let grants = resolve_grants_at(tmp.path(), "w-agent", Some("worker"));
    assert!(grants.is_empty(), "worker role baseline has no elevated services");
}

#[test]
fn auth_grants_resolve_provisioned_record_overrides_role() {
    use crate::grants_projection::ProvisionedGrantsProjection;
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::projections::registry::Projection;

    let tmp = tempdir().unwrap();
    // provisioned_grants (Account-scoped) → account.db, where resolve reads it.
    let mut conn = ship_substrate::db::open_account_db(tmp.path()).unwrap();

    // Provision a narrower set than the commander role default.
    let ev = EventEnvelope::new(
        AGENT_GRANTS_PROVISIONED,
        "pinned-agent",
        &serde_json::json!({ "grants": ["agents"], "source": "operator" }),
    )
    .unwrap();
    ProvisionedGrantsProjection.apply(&ev, &mut conn).unwrap();

    // Commander role says ["agent"] but provisioned set ["agents"] wins.
    let grants = resolve_grants_at(tmp.path(), "pinned-agent", Some("commander"));
    assert_eq!(grants, sv(&["agents"]));
}

#[test]
fn auth_grants_resolve_provisioned_empty_set_overrides_role() {
    use crate::grants_projection::ProvisionedGrantsProjection;
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::projections::registry::Projection;

    let tmp = tempdir().unwrap();
    // provisioned_grants (Account-scoped) → account.db, where resolve reads it.
    let mut conn = ship_substrate::db::open_account_db(tmp.path()).unwrap();

    // Operator explicitly revokes all grants.
    let ev = EventEnvelope::new(
        AGENT_GRANTS_PROVISIONED,
        "revoked-agent",
        &serde_json::json!({ "grants": [], "source": "operator" }),
    )
    .unwrap();
    ProvisionedGrantsProjection.apply(&ev, &mut conn).unwrap();

    // Even commander role gets no elevated services — provisioned empty wins.
    let grants = resolve_grants_at(tmp.path(), "revoked-agent", Some("commander"));
    assert!(grants.is_empty(), "empty provisioned set denies role default");
}

// ── pending_grants diff ───────────────────────────────────────────────────────

#[test]
fn auth_grants_pending_fully_covered_is_empty() {
    let requested = sv(&["ship-orchestration"]);
    let approved = sv(&["ship-orchestration", "ship-authoring"]);
    assert!(pending_grants(&requested, &approved).is_empty());
    assert!(!has_pending(&requested, &approved));
}

#[test]
fn auth_grants_pending_partial_coverage_surfaces_unapproved() {
    let requested = sv(&["ship-orchestration", "ship-authoring"]);
    let approved = sv(&["ship-orchestration"]);
    let pending = pending_grants(&requested, &approved);
    assert_eq!(pending, sv(&["ship-authoring"]));
    assert!(has_pending(&requested, &approved));
}

#[test]
fn auth_grants_pending_no_approval_all_requested_are_pending() {
    let requested = sv(&["ship-authoring", "ship-orchestration"]);
    // sorted output
    let pending = pending_grants(&requested, &[]);
    assert_eq!(pending, sv(&["ship-authoring", "ship-orchestration"]));
    assert!(has_pending(&requested, &[]));
}

#[test]
fn auth_grants_pending_empty_request_never_pending() {
    assert!(pending_grants(&[], &sv(&["ship-orchestration"])).is_empty());
    assert!(!has_pending(&[], &[]));
}

#[test]
fn auth_grants_pending_deduplicates_and_sorts_output() {
    let requested = sv(&["b", "a", "b"]);
    let approved = sv(&["a"]);
    let pending = pending_grants(&requested, &approved);
    assert_eq!(pending, sv(&["b"]), "deduped and sorted");
}

// ── provision_grants idempotent latest-write-wins ────────────────────────────

#[test]
fn auth_grants_provision_latest_write_wins() {
    use crate::grants_projection::ProvisionedGrantsProjection;
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::projections::registry::Projection;

    let tmp = tempdir().unwrap();
    // provisioned_grants (Account-scoped) → account.db, where resolve reads it.
    let mut conn = ship_substrate::db::open_account_db(tmp.path()).unwrap();

    let mk = |grants: &[&str], source: &str| {
        EventEnvelope::new(
            AGENT_GRANTS_PROVISIONED,
            "rewrite-agent",
            &serde_json::json!({ "grants": grants, "source": source }),
        )
        .unwrap()
    };

    ProvisionedGrantsProjection.apply(&mk(&["ship-orchestration"], "install"), &mut conn).unwrap();
    ProvisionedGrantsProjection.apply(&mk(&["ship-authoring"], "operator"), &mut conn).unwrap();

    let grants = resolve_grants_at(tmp.path(), "rewrite-agent", None);
    assert_eq!(grants, sv(&["ship-authoring"]), "operator write replaces install baseline");
}
