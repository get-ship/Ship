//! Event registrations for the ACCOUNT domain.
//!
//! Registers the 3 `account.login_*` / `account.logout` events with the global
//! event registry using the hoisted `register_event!` macro from `ship_types`.
//!
//! These registrations moved from `ship-auth/src/event_registrations.rs`
//! (keystone-A correctness fix, 2026-06-19): the `account.*` audit events are
//! EMITTED by this crate (`ship-account::events` → `SpaceContext.kernel()`),
//! not by ship-auth. Registration must be link-time present in the emitter's
//! crate, or the kernel can't scope/route the event when the emitter's own
//! test binary is built (ship-auth is not a transitive dep of every test
//! linking the emitter). The `account.key_refreshed` registration is left in
//! ship-auth for a separate cleanup (no current emitter).
//!
//! Payload structs remain in `ship_types::payload` — the `Account*` types are
//! the shared `account.proto` shape consumed by both this crate and ship-auth.

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

register_event!("account.login_completed", [EventScope::Account], Retention::Durable, Stability::Stable, "AccountLoginCompleted", ship_types::payload::AccountLoginCompleted);
register_event!("account.login_started", [EventScope::Account], Retention::Durable, Stability::Stable, "AccountLoginStarted", ship_types::payload::AccountLoginStarted);
register_event!("account.logout", [EventScope::Account], Retention::Durable, Stability::Stable, "AccountLogout", ship_types::payload::AccountLogout);
