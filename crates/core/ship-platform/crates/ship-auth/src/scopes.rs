//! L1 policy bits kept here after the `ship-auth-types` carve (P2).
//!
//! The pure types (`DaemonSession`, `SessionRegistry`, `PeerIdentity`, the
//! session-token mint/parse, the operator/space-window constructors) live in
//! `ship_auth_types`. This module re-exports them so existing
//! `ship_platform::auth::scopes::DaemonSession` import paths remain stable,
//! and holds the L1 pieces that need the event store + projections:
//!
//! - `account_store` — the Account-scope sqlite event-store opener.
//! - `provision_grants` — write an agent's provisioned grant baseline
//!   via an `agent.grants.provisioned` event.
//! - `persist_session` / `remove_persisted_session` — write session
//!   lifecycle events.
//! - `hydrate_sessions` — read `crate::projection::list_active_sessions`
//!   and seed the in-memory registry.
//! - `resolve_grants` / `resolve_grants_at` / `resolve_from` — read the
//!   provisioned-grants projection to compute an agent's grant set.


// Stable surface re-exports — every prior `ship_platform::auth::scopes::Foo`
// import path continues to resolve through this module.
pub use ship_auth_types::{
    DaemonSession, SessionRegistry, drop_session, generate_session_token, hex_encode,
    is_session_token, mint_session, operator_session, space_window_session,
};

/// Auth-domain projections that must ride the Account-scope bus for
/// off-kernel emits (`auth.session.*`, `daemon.owner.*`, `agent.grants.*`).
/// Domain-local DI: the AUTH crate owns these projections and registers them
/// on the store it opens — no process-global factory.
fn auth_account_projections() -> Vec<Box<dyn ship_substrate::projections::Projection>> {
    vec![
        Box::new(crate::projection::AuthSessionsProjection),
        Box::new(crate::projection::DaemonOwnerProjection),
        Box::new(crate::grants_projection::ProvisionedGrantsProjection),
    ]
}

/// Open the Account-scope `account.db` for off-kernel auth/owner emits.
///
/// `auth.session.*` and `daemon.owner.*` are Account-owned → they land in
/// account.db, the db the `auth_sessions` / `daemon_owner` projections read.
/// The store bus carries the AUTH projections this domain owns so same-TX
/// folds land without ambient process state.
pub fn account_store(
) -> anyhow::Result<ship_substrate::persistence::event_store::SqliteEventStore> {
    ship_substrate::persistence::event_store::SqliteEventStore::new_for_scope_with(
        &ship_substrate::project::get_global_dir()?,
        ship_types::event_registry::EventScope::Account,
        None,
        auth_account_projections(),
    )
}

/// The grants the mint path stamps onto a new session for `agent_id`.
///
/// The per-agent **provisioned baseline** (operator-approved, seeded at install
/// from the agent's resolved skill grants) is the sole source. An agent with no
/// provisioned record carries no elevated grants (the C0/C1 baseline is implicit
/// at the gate).
pub fn resolve_grants(agent_id: &str) -> Vec<String> {
    resolve_from(crate::grants_projection::provisioned_grants_for(agent_id), agent_id)
}

/// Test/`_at` variant of [`resolve_grants`] against a specific home dir.
pub fn resolve_grants_at(home: &std::path::Path, agent_id: &str) -> Vec<String> {
    resolve_from(
        crate::grants_projection::provisioned_grants_for_at(home, agent_id),
        agent_id,
    )
}

/// Write an agent's provisioned grant baseline via an `agent.grants.provisioned`
/// event. `source` is "install" (seeded from resolved skill grants) or
/// "operator" (explicit approval); `approved_by` carries the operator id for
/// the latter. Append is transactional-with-projection, so a subsequent
/// [`resolve_grants`] for the same agent observes this write immediately.
///
/// This is **operator/daemon authority** — the agent never calls it. An agent
/// controls its skill files but not its provisioned baseline.
pub fn provision_grants(
    agent_id: &str,
    grants: &[String],
    source: &str,
    approved_by: Option<&str>,
) -> anyhow::Result<()> {
    let payload = ship_substrate::events::types::AgentGrantsProvisioned {
        grants: grants.to_vec(),
        source: source.to_string(),
        approved_by: approved_by.map(|s| s.to_string()),
    };
    let envelope = ship_substrate::events::EventEnvelope::new(
        ship_substrate::events::types::event_types::AGENT_GRANTS_PROVISIONED,
        agent_id,
        &payload,
    )?;
    let store = account_store()?;
    store.append_enforced(&envelope, &ship_types::ScopeBinding::daemon_sdk())?;
    Ok(())
}

/// Resolve a provisioned-lookup result to the agent's grant set. No record
/// means no elevated grants (the C0/C1 baseline is implicit at the gate).
fn resolve_from(lookup: anyhow::Result<Option<Vec<String>>>, agent_id: &str) -> Vec<String> {
    match lookup {
        Ok(Some(grants)) => grants,
        Ok(None) => Vec::new(),
        Err(e) => {
            // Fail closed: a projection read error must not silently grant
            // restricted domains. Log and deny extras.
            tracing::warn!(
                agent_id,
                "resolve_grants: provisioned read failed ({e}); denying restricted grants"
            );
            Vec::new()
        }
    }
}

fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Persist a session via the event log. Skips ephemeral or no-expiry
/// sessions (those live only in memory).
pub fn persist_session(session: &DaemonSession) {
    if session.ephemeral {
        return;
    }
    let Some(expires_at_unix) = session.expires_at_unix else {
        return;
    };
    let created_unix = now_unix()
        .saturating_sub(session.created_at.elapsed().as_secs() as i64);
    let payload = ship_substrate::events::types::AuthSessionCreated {
        agent_id: session.agent_id.clone(),
        // The session's OWN resolved space (or None) — never a fabricated
        // home/global space (#3: the global pseudo-space is gone). A space-window
        // session carries its space; operator/daemon-token sessions key None.
        space_id: session.space_id.clone(),
        created_at_unix: created_unix,
        expires_at_unix,
        source: "persist".to_string(),
    };
    let envelope = match ship_substrate::events::EventEnvelope::new(
        ship_substrate::events::types::event_types::AUTH_SESSION_CREATED,
        &session.token,
        &payload,
    ) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("session persist: envelope build failed: {e}");
            return;
        }
    };
    let store = match account_store() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("session persist: store open failed: {e}");
            return;
        }
    };
    if let Err(e) = store.append_enforced(&envelope, &ship_types::ScopeBinding::daemon_sdk()) {
        tracing::warn!("session persist: append failed: {e}");
    }
}

/// Revoke a persisted session via an `auth.session.revoked` event.
pub fn remove_persisted_session(token: &str) {
    let payload = ship_substrate::events::types::AuthSessionRevoked {
        reason: "user".to_string(),
    };
    let envelope = match ship_substrate::events::EventEnvelope::new(
        ship_substrate::events::types::event_types::AUTH_SESSION_REVOKED,
        token,
        &payload,
    ) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("session revoke: envelope build failed: {e}");
            return;
        }
    };
    let store = match account_store() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("session revoke: store open failed: {e}");
            return;
        }
    };
    if let Err(e) = store.append_enforced(&envelope, &ship_types::ScopeBinding::daemon_sdk()) {
        tracing::warn!("session revoke: append failed: {e}");
    }
}

/// Load all non-expired sessions from the projection into the in-memory
/// registry. Expired rows are skipped.
pub async fn hydrate_sessions(registry: &SessionRegistry) {
    let rows = match crate::projection::list_active_sessions() {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!("session hydrate: list failed: {e}");
            return;
        }
    };
    let mut reg = registry.lock().await;
    let mut loaded = 0usize;
    for row in rows {
        let session = DaemonSession {
            token: row.token.clone(),
            agent_id: row.agent_id,
            // Persisted sessions are operator/paired (is_operator clears grants);
            // restricted grants aren't persisted — re-minted on register.
            grants: vec![],
            // The persisted `auth_sessions.space_id` carries the SPACE id
            // (see `persist_session`), not a branch slug — restore it into
            // the explicit `space_id` field, which is what the emit path reads.
            space_id: row.space_id,
            // Hydrated (persisted) sessions are operator/paired, not pid-bound.
            // Window tokens are ephemeral and never persisted, so a restored
            // session is never pid-restricted.
            pid: None,
            created_at: std::time::Instant::now(),
            expires_at_unix: Some(row.expires_at_unix),
            ephemeral: false,
            // Operator creds are pid-attested + ephemeral, so they are never
            // persisted; a hydrated session is never the operator. The operator
            // re-mints by re-registering its shell session on launch.
            is_operator_cred: false,
        };
        reg.insert(row.token, session);
        loaded += 1;
    }
    if loaded > 0 {
        tracing::info!(loaded, "auth: hydrated sessions from projection");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_grants_empty_when_unprovisioned() {
        let tmp = tempfile::tempdir().unwrap();
        // No provisioned record → no elevated grants (C0/C1 baseline is implicit).
        assert!(resolve_grants_at(tmp.path(), "fresh-agent").is_empty());
    }

    #[test]
    fn resolve_grants_returns_provisioned_record() {
        use crate::grants_projection::ProvisionedGrantsProjection;
        use ship_substrate::events::EventEnvelope;
        use ship_substrate::projections::registry::Projection;

        let tmp = tempfile::tempdir().unwrap();
        // provisioned_grants (Account-scoped) lives in account.db, where
        // resolve_grants_at → provisioned_grants_for_at reads it.
        let mut conn = ship_substrate::db::open_account_db(tmp.path()).unwrap();
        // Operator provisions an explicit elevated set for the agent.
        let ev = EventEnvelope::new(
            ship_types::event_types::AGENT_GRANTS_PROVISIONED,
            "pinned",
            &serde_json::json!({ "grants": ["agents"], "source": "operator" }),
        )
        .unwrap();
        ProvisionedGrantsProjection.apply(&ev, &mut conn).unwrap();

        // The provisioned set is the agent's grants.
        let grants = resolve_grants_at(tmp.path(), "pinned");
        assert_eq!(grants, vec!["agents".to_string()]);
    }
}
