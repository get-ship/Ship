//! Scope model + session registry — pure capability policy.
//!
//! Scopes are coarse-grained capabilities assigned to sessions based on
//! the agent's role. The daemon's HTTP layer (in `apps/shipd/src/auth/`)
//! consumes this module via `require_scope`, `session_register`, and the
//! middleware that populates `AuthContext`.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Coarse-grained capability scopes for daemon session access control.
/// Each handler declares which scope it requires; the auth layer checks
/// the session's granted scopes before the handler runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    ReadWorkspace,
    WriteWorkspace,
    ReadRuns,
    CreateRuns,
    ManageRuns,
    EmitEvents,
    MeshSend,
    MeshBroadcast,
    LogProgress,
    StepDone,
    AdminDaemon,
    /// SUI / view authoring tool-group — the structured compiler/LSP, live
    /// canvas, and snapshot tools. Granted to design agents (the `designer`
    /// role); invisible to every other agent. This is the one gated MCP
    /// tool-group; the daemon binds it at session-register time. Not a
    /// self-declared role at call time — a capability the daemon mints.
    SuiAuthoring,
    /// Fine-grained artifact grants — granted in addition to workspace grants.
    ReadArtifactsRun,
    WriteArtifactsRun,
    DeleteArtifactsRun,
    ReadArtifactsProject,
    WriteArtifactsProject,
    DeleteArtifactsProject,
    ReadArtifactsUser,
    WriteArtifactsUser,
    DeleteArtifactsUser,
    /// Promote an artifact from one scope to another.
    PromoteArtifact,
    /// Bypass all scope checks — used by daemon.token and contributor profile.
    Full,
}

impl Scope {
    /// Returns true if `granted` contains this scope or `Full`.
    pub fn is_granted_by(self, granted: &[Scope]) -> bool {
        granted.contains(&Scope::Full) || granted.contains(&self)
    }
}

/// Per-connection MCP identity, threaded from the daemon's `AuthContext` into
/// the in-daemon-mounted MCP server's tool gate.
///
/// `apps/mcp` cannot depend on `apps/shipd` (that would cycle: shipd → mcp →
/// shipd), so the gate can't read shipd's `AuthContext` directly. Instead shipd
/// projects the gate's needs into this `ship-platform` type, inserts it as a
/// request extension before the `/mcp` nest-service, and the MCP gate reads it
/// from `Extension<http::request::Parts>`. The token is the source of truth —
/// scopes are daemon-minted at session-register, never self-declared.
#[derive(Debug, Clone)]
pub struct McpAuth {
    pub agent_id: String,
    pub scopes: Vec<Scope>,
    pub workspace_id: Option<String>,
}

impl McpAuth {
    /// True when the connection's token grants `scope` (or `Full`).
    pub fn allows(&self, scope: Scope) -> bool {
        scope.is_granted_by(&self.scopes)
    }
}

/// Route-level auth requirement. Every `Route` variant declares one via
/// `/// @auth ...` in `apps/shipd/src/routes.rs`. The router enforces this
/// at registration time (`ship_route` wraps the handler in a middleware
/// that reads `AuthContext` from request extensions and rejects with 401
/// /403 before the handler runs).
///
/// - `Public`: unauthenticated. Registered via `ship_public_route` in the
///   public group (no auth middleware applied). `ship_public_route` panics
///   if a route declares anything other than `Public`.
/// - `Cli`: requires `CallerKind::Cli` (daemon token or browser cookie).
///   MCP session tokens are categorically rejected even if they carry
///   `Scope::Full`. Account ops use this.
/// - `Scope(s)`: any authenticated caller whose scopes satisfy `s` (or
///   `Scope::Full`).
/// - `Handler`: the handler enforces its own scope (used when the required
///   scope depends on request body / DB state, e.g. artifact handlers
///   whose scope is read from the artifact record). The `route_scope_enum`
///   test suite asserts that every `Handler`-tagged handler still calls
///   `require_scope` (or `can_*_artifacts`) so a bare `Handler` annotation
///   can't be used to silently disable enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredAuth {
    Public,
    Cli,
    Scope(Scope),
    Handler,
}

/// Artifact scopes granted to agents running inside a pipeline (worker/lead).
fn pipeline_agent_artifact_scopes() -> Vec<Scope> {
    vec![
        Scope::ReadArtifactsRun,
        Scope::WriteArtifactsRun,
        Scope::DeleteArtifactsRun,
        Scope::ReadArtifactsProject,
    ]
}

/// Map an agent role to its granted scopes.
pub fn scopes_for_role(role: Option<&str>) -> Vec<Scope> {
    match role {
        Some("commander") => {
            let mut s = vec![
                Scope::ReadWorkspace,
                Scope::WriteWorkspace,
                Scope::ReadRuns,
                Scope::CreateRuns,
                Scope::ManageRuns,
                Scope::EmitEvents,
                Scope::MeshSend,
                Scope::MeshBroadcast,
                Scope::LogProgress,
            ];
            s.extend(pipeline_agent_artifact_scopes());
            s
        }
        Some("lead") => {
            let mut s = vec![
                Scope::ReadWorkspace,
                Scope::WriteWorkspace,
                Scope::ReadRuns,
                Scope::CreateRuns,
                Scope::LogProgress,
                Scope::StepDone,
                Scope::MeshSend,
                Scope::EmitEvents,
            ];
            s.extend(pipeline_agent_artifact_scopes());
            s
        }
        Some("designer") => {
            // Design agents (shipwright, ship-designer) author Ship's own
            // views. They get the workspace read/write surface plus the
            // gated SuiAuthoring tool-group. Not orchestrators — no
            // CreateRuns/ManageRuns.
            let mut s = vec![
                Scope::ReadWorkspace,
                Scope::WriteWorkspace,
                Scope::ReadRuns,
                Scope::EmitEvents,
                Scope::LogProgress,
                Scope::SuiAuthoring,
            ];
            s.extend(pipeline_agent_artifact_scopes());
            s
        }
        Some("contributor") => vec![Scope::Full],
        Some("worker") | None => {
            let mut s = vec![
                Scope::ReadWorkspace,
                Scope::ReadRuns,
                Scope::LogProgress,
                Scope::StepDone,
                Scope::MeshSend,
                Scope::EmitEvents,
            ];
            s.extend(pipeline_agent_artifact_scopes());
            s
        }
        _ => vec![Scope::ReadWorkspace, Scope::ReadRuns, Scope::LogProgress],
    }
}

/// The **elevated services** (the C2 set) a role confers — the gate's scope axis
/// (distinct from `scopes`, which drives REST route-auth + artifact perms until
/// those die with the REST surface). The gate allows a C2 op only when its
/// service is here; the **C1 baseline is implicit** (every agent reads +
/// workspace-writes in every service); **C3 is never role-conferred**.
///
/// This is the role-derived provisioning source today; the install/approval
/// flow (`.artifacts/capability-provisioning-and-approval.md`) refines it into
/// per-agent operator-approved sets. Authoring services are listed ahead of
/// their C2 re-tiering — harmless while those ops are still C1.
pub fn grants_for_role(role: Option<&str>) -> Vec<String> {
    let g: &[&str] = match role {
        // Commander orchestrates: dispatch/stop/steer live in the `agent` service.
        Some("commander") | Some("lead") => &["agent"],
        // Designer authors: the C2 authoring services (agent profiles, skills,
        // mcp, surface creation, view edit). Note `view`/`create` are singular —
        // the actual op-domains.
        Some("designer") => &["agents", "skills", "mcp", "create", "view"],
        // contributor = the operator's own profile: every elevated service.
        Some("contributor") => &["agent", "agents", "skills", "mcp", "create", "view"],
        // worker / unknown: no elevated services (the C1 baseline only).
        _ => &[],
    };
    g.iter().map(|s| s.to_string()).collect()
}

/// The grants the mint path stamps onto a new session for `agent_id`.
///
/// The per-agent **provisioned baseline** (operator-approved, seeded at install
/// from the agent's resolved skill grants) is the source of truth. When no
/// baseline has been recorded yet, fall back to the role-derived grants — this
/// keeps un-provisioned agents (and dev-mode pairs) working exactly as before.
/// The provisioned record, once present, fully replaces the role default.
pub fn resolve_grants(agent_id: &str, role: Option<&str>) -> Vec<String> {
    resolve_from(crate::projections::provisioned_grants_for(agent_id), agent_id, role)
}

/// Test/`_at` variant of [`resolve_grants`] against a specific home dir.
pub fn resolve_grants_at(
    home: &std::path::Path,
    agent_id: &str,
    role: Option<&str>,
) -> Vec<String> {
    resolve_from(
        crate::projections::provisioned_grants::provisioned_grants_for_at(home, agent_id),
        agent_id,
        role,
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
    let store = ship_substrate::persistence::event_store::SqliteEventStore::new()?;
    ship_substrate::persistence::event_store::EventStore::append(&store, &envelope)?;
    Ok(())
}

/// Compose a provisioned-lookup result with the role fallback.
fn resolve_from(
    lookup: anyhow::Result<Option<Vec<String>>>,
    agent_id: &str,
    role: Option<&str>,
) -> Vec<String> {
    match lookup {
        Ok(Some(grants)) => grants,
        Ok(None) => grants_for_role(role),
        Err(e) => {
            // Fail closed-ish: a projection read error must not silently grant
            // the role baseline's restricted domains. Log and deny extras.
            tracing::warn!(
                agent_id,
                "resolve_grants: provisioned read failed ({e}); denying restricted grants"
            );
            Vec::new()
        }
    }
}

// ── Session registry ────────────────────────────────────────────────────────

/// A daemon-scoped session — maps a session token to an agent's granted scopes.
#[derive(Debug, Clone)]
pub struct DaemonSession {
    pub token: String,
    pub agent_id: String,
    pub scopes: Vec<Scope>,
    /// Restricted capability grants (the gate's grant axis) — minted from the
    /// agent's provisioned set at register. Distinct from `scopes` (REST
    /// route-auth + artifacts). See `.artifacts/capability-provisioning-and-approval.md`.
    pub grants: Vec<String>,
    pub workspace_id: Option<String>,
    pub created_at: std::time::Instant,
    /// Unix seconds when this session expires. None = never expires (in-memory only).
    pub expires_at_unix: Option<i64>,
    /// If true, the session lives only in memory (dev-mode auto-pair). Not persisted.
    pub ephemeral: bool,
}

impl DaemonSession {
    /// Returns true if this session has passed its expiry.
    pub fn is_expired(&self) -> bool {
        match self.expires_at_unix {
            Some(t) => chrono::Utc::now().timestamp() >= t,
            None => false,
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
    let scopes_json = match serde_json::to_string(&session.scopes) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("session persist: scopes serialize failed: {e}");
            return;
        }
    };
    let payload = ship_substrate::events::types::AuthSessionCreated {
        agent_id: session.agent_id.clone(),
        scopes_json,
        project_id: session.workspace_id.clone(),
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
    let store = match ship_substrate::persistence::event_store::SqliteEventStore::new() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("session persist: store open failed: {e}");
            return;
        }
    };
    if let Err(e) = ship_substrate::persistence::event_store::EventStore::append(&store, &envelope) {
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
    let store = match ship_substrate::persistence::event_store::SqliteEventStore::new() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("session revoke: store open failed: {e}");
            return;
        }
    };
    if let Err(e) = ship_substrate::persistence::event_store::EventStore::append(&store, &envelope) {
        tracing::warn!("session revoke: append failed: {e}");
    }
}

/// Load all non-expired sessions from the projection into the in-memory
/// registry. Expired rows are skipped.
pub async fn hydrate_sessions(registry: &SessionRegistry) {
    let rows = match crate::projections::auth::list_active_sessions() {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!("session hydrate: list failed: {e}");
            return;
        }
    };
    let mut reg = registry.lock().await;
    let mut loaded = 0usize;
    for row in rows {
        let scopes: Vec<Scope> =
            serde_json::from_str(&row.scopes_json).unwrap_or_default();
        let session = DaemonSession {
            token: row.token.clone(),
            agent_id: row.agent_id,
            scopes,
            // Persisted sessions are operator/paired (is_operator clears grants);
            // restricted grants aren't persisted — re-minted on register.
            grants: vec![],
            workspace_id: row.project_id,
            created_at: std::time::Instant::now(),
            expires_at_unix: Some(row.expires_at_unix),
            ephemeral: false,
        };
        reg.insert(row.token, session);
        loaded += 1;
    }
    if loaded > 0 {
        tracing::info!(loaded, "auth: hydrated sessions from projection");
    }
}

/// Thread-safe session registry shared across the daemon.
pub type SessionRegistry = Arc<Mutex<HashMap<String, DaemonSession>>>;

/// Generate a session token with the `ship_sess_` prefix.
///
/// Returns an error if the OS entropy source is unavailable.
pub fn generate_session_token() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes)?;
    Ok(format!("ship_sess_{}", hex_encode(&bytes)))
}

/// Returns true if the token is a session token (ship_sess_ prefix).
pub fn is_session_token(token: &str) -> bool {
    token.starts_with("ship_sess_")
}

/// Hex-encode a byte slice with lowercase ASCII. Used by token formatters
/// across the auth module; defined here so the platform side doesn't
/// depend on shipd's `auth::hex_encode`.
pub fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_full_grants_everything() {
        let granted = vec![Scope::Full];
        assert!(Scope::CreateRuns.is_granted_by(&granted));
        assert!(Scope::AdminDaemon.is_granted_by(&granted));
        assert!(Scope::MeshBroadcast.is_granted_by(&granted));
    }

    #[test]
    fn scope_specific_grant() {
        let granted = vec![Scope::ReadWorkspace, Scope::LogProgress];
        assert!(Scope::ReadWorkspace.is_granted_by(&granted));
        assert!(Scope::LogProgress.is_granted_by(&granted));
        assert!(!Scope::CreateRuns.is_granted_by(&granted));
    }

    #[test]
    fn scopes_for_worker_role() {
        let scopes = scopes_for_role(Some("worker"));
        assert!(Scope::ReadWorkspace.is_granted_by(&scopes));
        assert!(Scope::LogProgress.is_granted_by(&scopes));
        assert!(!Scope::CreateRuns.is_granted_by(&scopes));
        assert!(!Scope::ManageRuns.is_granted_by(&scopes));
    }

    #[test]
    fn scopes_for_commander_role() {
        let scopes = scopes_for_role(Some("commander"));
        assert!(Scope::CreateRuns.is_granted_by(&scopes));
        assert!(Scope::ManageRuns.is_granted_by(&scopes));
        assert!(Scope::MeshBroadcast.is_granted_by(&scopes));
    }

    #[test]
    fn scopes_for_contributor_role() {
        let scopes = scopes_for_role(Some("contributor"));
        assert!(Scope::Full.is_granted_by(&scopes));
        assert!(Scope::AdminDaemon.is_granted_by(&scopes));
    }

    #[test]
    fn resolve_grants_falls_back_to_role_when_unprovisioned() {
        let tmp = tempfile::tempdir().unwrap();
        // No provisioned record → role baseline applies (migration-safe).
        let grants = resolve_grants_at(tmp.path(), "fresh-agent", Some("commander"));
        assert_eq!(grants, vec!["agent".to_string()]);
        // A worker with no record gets no elevated services (C1 baseline only).
        assert!(resolve_grants_at(tmp.path(), "w", Some("worker")).is_empty());
    }

    #[test]
    fn resolve_grants_provisioned_record_overrides_role() {
        use ship_substrate::events::EventEnvelope;
        use crate::projections::provisioned_grants::ProvisionedGrantsProjection;
        use ship_substrate::projections::registry::Projection;

        let tmp = tempfile::tempdir().unwrap();
        ship_substrate::db::ensure_db_at(tmp.path()).unwrap();
        let mut conn = ship_substrate::db::open_db_at(
            &ship_substrate::db::platform_db_path(tmp.path()),
        )
        .unwrap();
        // Operator provisions a *narrower* set than the commander role default.
        let ev = EventEnvelope::new(
            ship_types::event_types::AGENT_GRANTS_PROVISIONED,
            "pinned",
            &serde_json::json!({ "grants": ["agents"], "source": "operator" }),
        )
        .unwrap();
        ProvisionedGrantsProjection.apply(&ev, &mut conn).unwrap();

        // Role says commander (→ `agent`), but the provisioned set wins.
        let grants = resolve_grants_at(tmp.path(), "pinned", Some("commander"));
        assert_eq!(grants, vec!["agents".to_string()]);
    }

    #[test]
    fn designer_role_grants_sui_authoring_but_not_orchestration() {
        let scopes = scopes_for_role(Some("designer"));
        assert!(Scope::SuiAuthoring.is_granted_by(&scopes));
        assert!(Scope::WriteWorkspace.is_granted_by(&scopes));
        // Designers are not orchestrators.
        assert!(!Scope::CreateRuns.is_granted_by(&scopes));
        assert!(!Scope::ManageRuns.is_granted_by(&scopes));
    }

    #[test]
    fn non_designer_roles_lack_sui_authoring() {
        assert!(!Scope::SuiAuthoring.is_granted_by(&scopes_for_role(Some("worker"))));
        assert!(!Scope::SuiAuthoring.is_granted_by(&scopes_for_role(Some("commander"))));
        // Full bypasses, so contributor implicitly has it.
        assert!(Scope::SuiAuthoring.is_granted_by(&scopes_for_role(Some("contributor"))));
    }

    #[test]
    fn scopes_for_none_role_defaults_to_worker() {
        let worker = scopes_for_role(Some("worker"));
        let none = scopes_for_role(None);
        assert_eq!(worker, none);
    }

    #[test]
    fn session_token_format() {
        let token = generate_session_token().unwrap();
        assert!(is_session_token(&token));
    }

    #[test]
    fn worker_can_write_run_artifacts_but_not_project() {
        let scopes = scopes_for_role(Some("worker"));
        assert!(Scope::WriteArtifactsRun.is_granted_by(&scopes));
        assert!(!Scope::WriteArtifactsProject.is_granted_by(&scopes));
    }

    #[test]
    fn worker_cannot_promote_artifacts() {
        let scopes = scopes_for_role(Some("worker"));
        assert!(!Scope::PromoteArtifact.is_granted_by(&scopes));
    }

    #[test]
    fn only_contributor_and_full_can_promote_artifacts() {
        assert!(!Scope::PromoteArtifact.is_granted_by(&scopes_for_role(Some("commander"))));
        assert!(!Scope::PromoteArtifact.is_granted_by(&scopes_for_role(Some("lead"))));
        assert!(!Scope::PromoteArtifact.is_granted_by(&scopes_for_role(Some("worker"))));
        assert!(Scope::PromoteArtifact.is_granted_by(&[Scope::Full]));
    }

    #[test]
    fn worker_can_read_project_artifacts_but_not_write() {
        let scopes = scopes_for_role(Some("worker"));
        assert!(Scope::ReadArtifactsProject.is_granted_by(&scopes));
        assert!(!Scope::WriteArtifactsProject.is_granted_by(&scopes));
        assert!(!Scope::DeleteArtifactsProject.is_granted_by(&scopes));
    }
}
