/// Integration tests — daemon owner (attest) lifecycle.
///
/// Covers:
/// - `set_daemon_owner` emits `daemon.owner.bound`
/// - `clear_daemon_owner` emits `daemon.owner.unbound`
/// - `daemon_owner` reads the projection round-trip
/// - `clear_daemon_owner` no-ops when not bound
/// - `AttestationConfig::new` and `from_defaults` ctor correctness
use ship_substrate::events::filter::EventFilter;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_types::event_types::{DAEMON_OWNER_BOUND, DAEMON_OWNER_UNBOUND};
use tempfile::tempdir;

use crate::auth::attest::{
    AttestationConfig, DEFAULT_ATTESTATION_ISSUER, DEFAULT_ATTESTATION_JWKS_URL,
    clear_daemon_owner, daemon_owner, set_daemon_owner,
};
use crate::testutil::init_project;

fn query_events(event_type: &str) -> Vec<ship_types::envelope::EventEnvelope> {
    // daemon.owner.* is Account-scoped → account.db.
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

// ── AttestationConfig ─────────────────────────────────────────────────────────

#[test]
fn auth_attest_config_new_stores_supplied_values() {
    let cfg = AttestationConfig::new("https://example.com", "https://example.com/jwks");
    assert_eq!(cfg.issuer, "https://example.com");
    assert_eq!(cfg.jwks_url, "https://example.com/jwks");
}

#[test]
fn auth_attest_config_from_defaults_uses_compiled_in_values() {
    let cfg = AttestationConfig::from_defaults();
    assert_eq!(cfg.issuer, DEFAULT_ATTESTATION_ISSUER);
    assert_eq!(cfg.jwks_url, DEFAULT_ATTESTATION_JWKS_URL);
}

#[test]
fn auth_attest_config_defaults_decouple_issuer_from_jwks_host() {
    // The brand-domain issuer and the gateway-hosted JWKS URL are intentionally
    // different hosts — this verifies the deliberate decoupling.
    let cfg = AttestationConfig::from_defaults();
    assert!(cfg.issuer.contains("getship.dev"), "issuer should be brand domain");
    assert!(cfg.jwks_url.contains("api.getship.dev"), "JWKS on API gateway");
    assert_ne!(cfg.issuer, cfg.jwks_url);
}

// ── daemon_owner round-trip ───────────────────────────────────────────────────

#[test]
fn auth_attest_set_daemon_owner_round_trips() {
    let tmp = tempdir().unwrap();
    init_project(tmp.path()).unwrap();

    set_daemon_owner("user-round-trip").unwrap();
    assert_eq!(daemon_owner().as_deref(), Some("user-round-trip"));
}

#[test]
fn auth_attest_set_daemon_owner_emits_daemon_owner_bound() {
    let tmp = tempdir().unwrap();
    init_project(tmp.path()).unwrap();

    set_daemon_owner("user-emit-bound").unwrap();

    let events = query_events(DAEMON_OWNER_BOUND);
    assert!(
        events.iter().any(|e| e.entity_id == "user-emit-bound"),
        "expected daemon.owner.bound with entity_id=user-emit-bound; got {events:?}"
    );
}

// ── clear_daemon_owner ────────────────────────────────────────────────────────

#[test]
fn auth_attest_clear_daemon_owner_emits_daemon_owner_unbound() {
    let tmp = tempdir().unwrap();
    init_project(tmp.path()).unwrap();

    set_daemon_owner("user-to-clear").unwrap();
    clear_daemon_owner().unwrap();

    let events = query_events(DAEMON_OWNER_UNBOUND);
    assert!(
        events.iter().any(|e| e.entity_id == "user-to-clear"),
        "expected daemon.owner.unbound with entity_id=user-to-clear; got {events:?}"
    );
}

#[test]
fn auth_attest_clear_daemon_owner_noop_when_not_bound() {
    let tmp = tempdir().unwrap();
    init_project(tmp.path()).unwrap();

    // No binding — clear is a no-op (returns Ok without emitting).
    let pre = query_events(DAEMON_OWNER_UNBOUND).len();
    clear_daemon_owner().unwrap();
    let post = query_events(DAEMON_OWNER_UNBOUND).len();
    assert_eq!(pre, post, "clear on unbound daemon must not emit an event");
}

#[test]
fn auth_attest_set_twice_replaces_binding() {
    let tmp = tempdir().unwrap();
    init_project(tmp.path()).unwrap();

    set_daemon_owner("user-first").unwrap();
    set_daemon_owner("user-second").unwrap();

    // The projection's single-row table: latest binding wins.
    assert_eq!(daemon_owner().as_deref(), Some("user-second"));
}
