//! Account-lifecycle audit events, emitted through the `KernelLog` port.
//!
//! `account.*` and `daemon.owner.*` are **Account-scoped** (registered in the
//! substrate payload registry) → the kernel routes them to `account.db`. These
//! helpers replace the legacy `SqliteEventStore::append` kernel-bypass: every
//! emit goes through `KernelLog::emit` with a daemon-internal `EmitContext`
//! (no space key — account identity is process-global, not space-partitioned).
//!
//! Emit failures are propagated as `AccountError` so a lifecycle method can
//! decide whether to surface or log; the service swallows audit-emit errors
//! after logging so an audit write never breaks the user-visible flow.

use ship_substrate::events::types::event_types::{
    ACCOUNT_LOGIN_COMPLETED, ACCOUNT_LOGIN_STARTED, ACCOUNT_LOGOUT, DAEMON_OWNER_BOUND,
    DAEMON_OWNER_UNBOUND,
};
use ship_substrate::events::types::{
    AccountLoginCompleted, AccountLoginStarted, AccountLogout, DaemonOwnerBound, DaemonOwnerUnbound,
};
use ship_substrate::events::validator::EmitContext;
use ship_substrate::events::EventEnvelope;
use ship_substrate::ports::KernelLog;

use crate::error::AccountError;

/// Daemon-internal emit context for Account-scoped events. No space — account
/// identity is process-global; `enforce_scope` stamps the (absent) account key
/// without a cross-check.
fn account_ctx() -> EmitContext {
    EmitContext::daemon_internal(None)
}

async fn emit(kernel: &KernelLog, envelope: EventEnvelope) -> Result<(), AccountError> {
    kernel.emit(envelope, &account_ctx()).await?;
    Ok(())
}

/// Emit `account.login_started`. `entity_id` is the opaque PKCE state id.
pub async fn emit_login_started(
    kernel: &KernelLog,
    state_id: &str,
    redirect_uri: &str,
) -> Result<(), AccountError> {
    let payload = AccountLoginStarted { redirect_uri: redirect_uri.to_string() };
    let env = EventEnvelope::new(ACCOUNT_LOGIN_STARTED, state_id, &payload)
        .map_err(|e| AccountError::Internal(e.to_string()))?;
    emit(kernel, env).await
}

/// Emit `account.login_completed`. `entity_id` is the user_id.
pub async fn emit_login_completed(kernel: &KernelLog, user_id: &str) -> Result<(), AccountError> {
    let env = EventEnvelope::new(ACCOUNT_LOGIN_COMPLETED, user_id, &AccountLoginCompleted {})
        .map_err(|e| AccountError::Internal(e.to_string()))?;
    emit(kernel, env).await
}

/// Emit `account.logout`. `entity_id` is the prior user_id, or `"none"` on an
/// idempotent (no-token) logout. `had_token` distinguishes the two; `key_revoked`
/// records whether the gateway-side key was successfully invalidated.
pub async fn emit_logout(
    kernel: &KernelLog,
    user_id: Option<&str>,
    had_token: bool,
    key_revoked: bool,
) -> Result<(), AccountError> {
    let entity = user_id.unwrap_or("none");
    let payload = AccountLogout { had_token, key_revoked };
    let env = EventEnvelope::new(ACCOUNT_LOGOUT, entity, &payload)
        .map_err(|e| AccountError::Internal(e.to_string()))?;
    emit(kernel, env).await
}

/// Emit `daemon.owner.bound`. `entity_id` is the user_id. The single-row
/// owner projection replaces any prior binding.
pub async fn emit_owner_bound(kernel: &KernelLog, user_id: &str) -> Result<(), AccountError> {
    let payload = DaemonOwnerBound {
        source: "account".to_string(),
        bound_at_unix: chrono::Utc::now().timestamp(),
    };
    let env = EventEnvelope::new(DAEMON_OWNER_BOUND, user_id, &payload)
        .map_err(|e| AccountError::Internal(e.to_string()))?;
    emit(kernel, env).await
}

/// Emit `daemon.owner.unbound`. `entity_id` is the prior user_id.
pub async fn emit_owner_unbound(kernel: &KernelLog, user_id: &str) -> Result<(), AccountError> {
    let env = EventEnvelope::new(DAEMON_OWNER_UNBOUND, user_id, &DaemonOwnerUnbound {})
        .map_err(|e| AccountError::Internal(e.to_string()))?;
    emit(kernel, env).await
}
