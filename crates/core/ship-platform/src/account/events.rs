//! Account-lifecycle audit events.
//!
//! Thin wrappers around `SqliteEventStore::append` so account flows can
//! emit without taking a `&KernelRouter`. The kernel layer is bypassed
//! deliberately — these events are short-lived audit records, not
//! orchestration inputs, and account flows already write directly to the
//! event log for the daemon-owner binding alongside them.
//!
//! All emit_* helpers swallow store errors after logging — an audit-event
//! failure must not break the user-visible flow. Callers should still
//! propagate primary errors from the operation itself.

use ship_substrate::events::EventEnvelope;
use ship_substrate::events::types::event_types::{
    ACCOUNT_KEY_REFRESHED, ACCOUNT_LOGIN_COMPLETED, ACCOUNT_LOGIN_STARTED, ACCOUNT_LOGOUT,
};
use ship_substrate::events::types::{
    AccountKeyRefreshed, AccountLoginCompleted, AccountLoginStarted, AccountLogout,
};
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};

fn append(envelope: EventEnvelope) {
    match SqliteEventStore::new() {
        Ok(store) => {
            if let Err(e) = store.append(&envelope) {
                tracing::warn!("account event append failed ({}): {e}", envelope.event_type);
            }
        }
        Err(e) => tracing::warn!("account event store open failed: {e}"),
    }
}

/// Emit `account.login_started`. `entity_id` is the opaque PKCE state id.
pub fn emit_login_started(state_id: &str, redirect_uri: &str) {
    let payload = AccountLoginStarted {
        redirect_uri: redirect_uri.to_string(),
    };
    match EventEnvelope::new(ACCOUNT_LOGIN_STARTED, state_id, &payload) {
        Ok(env) => append(env),
        Err(e) => tracing::warn!("account event build failed (login_started): {e}"),
    }
}

/// Emit `account.login_completed`. `entity_id` is the user_id.
pub fn emit_login_completed(user_id: &str) {
    match EventEnvelope::new(ACCOUNT_LOGIN_COMPLETED, user_id, &AccountLoginCompleted {}) {
        Ok(env) => append(env),
        Err(e) => tracing::warn!("account event build failed (login_completed): {e}"),
    }
}

/// Emit `account.logout`. `entity_id` is the previously-bound user_id,
/// or `"none"` if logout was idempotent. `had_token` distinguishes the
/// two cases for audit consumers.
pub fn emit_logout(user_id: Option<&str>, had_token: bool) {
    let entity = user_id.unwrap_or("none");
    let payload = AccountLogout { had_token };
    match EventEnvelope::new(ACCOUNT_LOGOUT, entity, &payload) {
        Ok(env) => append(env),
        Err(e) => tracing::warn!("account event build failed (logout): {e}"),
    }
}

/// Emit `account.key_refreshed`. `entity_id` is the user_id.
pub fn emit_key_refreshed(user_id: &str) {
    match EventEnvelope::new(ACCOUNT_KEY_REFRESHED, user_id, &AccountKeyRefreshed {}) {
        Ok(env) => append(env),
        Err(e) => tracing::warn!("account event build failed (key_refreshed): {e}"),
    }
}
