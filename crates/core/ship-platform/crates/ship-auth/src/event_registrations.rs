//! Event registrations for the AUTH domain.
//!
//! Registers the 8 auth/account events with the global event registry
//! using the hoisted `register_event!` macro from `ship_types`.
//!
//! Keystone-A Phase A bundle 2 (2026-06-10): the AUTH domain owns its events
//! at L1. These registrations moved from `ship_substrate::events::payload_registry`
//! into this crate so that registrations co-locate with their domain owner.
//!
//! Payload structs remain in `ship_types::payload` — the auth/account types are
//! consumed by both ship-auth and ship-account (an independent L1 crate), so they
//! cannot move without introducing a dependency or a shim.

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

// ── auth.session.* ──────────────────────────────────────────────────────────
register_event!("auth.session.created", [EventScope::Account], Retention::Durable, Stability::Stable, "AuthSessionCreated", ship_types::payload::AuthSessionCreated);
register_event!("auth.session.expired", [EventScope::Account], Retention::Durable, Stability::Stable, "AuthSessionExpired", ship_types::payload::AuthSessionExpired);
register_event!("auth.session.revoked", [EventScope::Account], Retention::Durable, Stability::Stable, "AuthSessionRevoked", ship_types::payload::AuthSessionRevoked);

// ── account.* ───────────────────────────────────────────────────────────────
register_event!("account.key_refreshed", [EventScope::Account], Retention::Ephemeral, Stability::Stable, "AccountKeyRefreshed", ship_types::payload::AccountKeyRefreshed);
register_event!("account.login_completed", [EventScope::Account], Retention::Durable, Stability::Stable, "AccountLoginCompleted", ship_types::payload::AccountLoginCompleted);
register_event!("account.login_started", [EventScope::Account], Retention::Durable, Stability::Stable, "AccountLoginStarted", ship_types::payload::AccountLoginStarted);
register_event!("account.logout", [EventScope::Account], Retention::Durable, Stability::Stable, "AccountLogout", ship_types::payload::AccountLogout);

// ── agent.grants.provisioned ─────────────────────────────────────────────────
// Account-scoped: provisioned grant baseline, approved by an operator, used
// by the token-mint path. Folded by ProvisionedGrantsProjection (moved here
// from ship-substrate as part of this bundle).
register_event!("agent.grants.provisioned", [EventScope::Account], Retention::Durable, Stability::Stable, "AgentGrantsProvisioned", ship_types::payload::AgentGrantsProvisioned);
