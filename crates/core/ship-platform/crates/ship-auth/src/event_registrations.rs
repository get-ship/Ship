//! Event registrations for the AUTH domain.
//!
//! Registers the auth-session / account-key-refresh / agent-grants events
//! with the global event registry using the hoisted `register_event!` macro
//! from `ship_types`.
//!
//! Keystone-A Phase A bundle 2 (2026-06-10): the AUTH domain owns its events
//! at L1. These registrations moved from `ship_substrate::events::payload_registry`
//! into this crate so that registrations co-locate with their domain owner.
//!
//! Payload structs (`AuthSession*`, `AccountKeyRefreshed`, `AgentGrantsProvisioned`)
//! remain in `ship_types::payload`. The `AccountLogin*` / `AccountLogout`
//! payloads are co-owned by ship-account (it emits them) and live alongside
//! the others; their registrations moved to `ship-account/src/event_registrations.rs`
//! so the registration is link-time present in the emitter's crate.

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

// ── auth.session.* ──────────────────────────────────────────────────────────
register_event!("auth.session.created", [EventScope::Account], Retention::Durable, Stability::Stable, "AuthSessionCreated", ship_types::payload::AuthSessionCreated);
register_event!("auth.session.expired", [EventScope::Account], Retention::Durable, Stability::Stable, "AuthSessionExpired", ship_types::payload::AuthSessionExpired);
register_event!("auth.session.revoked", [EventScope::Account], Retention::Durable, Stability::Stable, "AuthSessionRevoked", ship_types::payload::AuthSessionRevoked);

// ── account.* ───────────────────────────────────────────────────────────────
register_event!("account.key_refreshed", [EventScope::Account], Retention::Ephemeral, Stability::Stable, "AccountKeyRefreshed", ship_types::payload::AccountKeyRefreshed);

// ── agent.grants.provisioned ─────────────────────────────────────────────────
// Account-scoped: provisioned grant baseline, approved by an operator, used
// by the token-mint path. Folded by ProvisionedGrantsProjection (moved here
// from ship-substrate as part of this bundle).
register_event!("agent.grants.provisioned", [EventScope::Account], Retention::Durable, Stability::Stable, "AgentGrantsProvisioned", ship_types::payload::AgentGrantsProvisioned);
