//! Scope model + session registry — pure capability types and the
//! in-memory session handle. The L0 seam: role→scope tables, the
//! `Scope`/`DaemonSession` value types, the `SessionRegistry` handle,
//! session-token mint/parse, and the operator/space-window constructors.
//!
//! What stays L1 in `ship_platform::auth::scopes`:
//! - `account_store` (substrate event-store writer helper),
//! - `provision_grants` / `persist_session` / `remove_persisted_session`
//!   (event-store writes),
//! - `hydrate_sessions` / `resolve_grants` / `resolve_grants_at` /
//!   `resolve_from` (projection reads).
//!
//! `scopes_for_role` and `grants_for_role` live here (they are pure
//! role tables, no I/O); the projection-aware `resolve_grants` calls
//! them as the role fallback in L1.

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
/// projects the gate's needs into this `ship-auth-types` type, inserts it as a
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
/// - `Scope(s)`: any authenticated caller whose scopes satisfy `s` (or
///   `Scope::Full`).
/// - `Handler`: the handler enforces its own scope (used when the required
///   scope depends on request body / DB state, e.g. artifact handlers
///   whose scope is read from the artifact record). The `route_scope_enum`
///   test suite asserts that every `Handler`-tagged handler still calls
///   `require_scope` (or `can_*_artifacts`) so a bare `Handler` annotation
///   can't be used to silently disable enforcement.
///
/// The `Cli` variant was removed (S6, #124): "requires the CLI transport" was an
/// elevation shortcut. Operator authority now derives SOLELY from a verified
/// operator cred at the gate (`command::call`); routes that were operator-only
/// either enforce `Scope::Full` in-handler (`Handler`) or run through the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredAuth {
    Public,
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
    /// The space this session belongs to (path-hash space identity). Authoritative
    /// per-connection space for the emit path: a space-window session carries its
    /// OWN space here (`space_window_session`), so events it emits are stamped with
    /// that space. `None` for sessions with no resolvable space — keyed `None` at
    /// emit, never a fabricated home/global space (S2 home-space kill). Distinct
    /// from `workspace_id` (a git branch slug). See spec
    /// `.artifacts/specs/space/128-multi-space-emit.md`.
    pub space_id: Option<String>,
    /// Kernel-verified pid this token is bound to, if any. Set for webview
    /// window tokens (minted from the connection's `SO_PEERCRED` pid): the
    /// token is then two-factor — the secret *and* being that exact process.
    /// `None` for unbound sessions (CLI daemon token, agent peer sessions,
    /// legacy/hydrated sessions), which are not pid-restricted.
    pub pid: Option<u32>,
    /// True iff this session is a **verified operator cred** — the SOLE source
    /// of operator authority (S6 elevation-source kill). Minted today only by
    /// [`operator_session`] from the pid-attested shell session (`pid` must be
    /// `Some` and `pid_authorizes` enforced at the transport). The CLI transport
    /// and the local peer-uid grant NO elevation — they leave this `false`. A
    /// cloud-issued operator cred can later set this through its own verified
    /// minter without touching the gate. Distinct from `Scope::Full` (the dying
    /// REST route-auth axis): only this flag flips `Authority::Operator`.
    pub is_operator_cred: bool,
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

    /// True iff this session's pid binding (if any) is satisfied by the
    /// kernel-verified `peer_pid` of the calling connection.
    ///
    /// An unbound session (`pid == None`) is not pid-restricted and passes.
    /// A pid-bound session (a webview window token) requires the caller to be
    /// that exact process: `peer_pid` must be `Some` and equal. Fail closed —
    /// a bound token presented with no kernel pid (e.g. over TCP, where
    /// `SO_PEERCRED` can't attest) is rejected, so the binding can't be
    /// stripped by switching transports.
    pub fn pid_authorizes(&self, peer_pid: Option<u32>) -> bool {
        match self.pid {
            None => true,
            Some(bound) => peer_pid == Some(bound),
        }
    }
}

/// Thread-safe session registry shared across the daemon.
pub type SessionRegistry = Arc<Mutex<HashMap<String, DaemonSession>>>;

/// Process-wide handle to the daemon's session registry.
///
/// shipd installs a clone of the runtime's registry `Arc` at boot via
/// [`set_session_registry`]. The host-side spawn path (supervisor) reads
/// it through [`session_registry`] to mint a per-process token for a
/// sandboxed actor — the token is minted in the ephemeral spawn path, so
/// it never crosses the event-persistence boundary. Mirrors the
/// `sandbox::dispatcher()` singleton: the spawn path needs the one
/// process-wide authority without threading a handle through every layer.
static SESSION_REGISTRY: std::sync::OnceLock<SessionRegistry> = std::sync::OnceLock::new();

/// Install the process-wide session registry. Called once by shipd at boot
/// with a clone of the runtime's registry `Arc` (so the global is a *view*
/// of the one registry, not a second source of truth). Subsequent calls
/// are no-ops.
///
/// `OnceLock` rather than `LazyLock` because the value is handed in at
/// boot from a runtime-instantiated `Arc` (it isn't a fixed initializer
/// we can compute at declaration time).
pub fn set_session_registry(registry: SessionRegistry) {
    let _ = SESSION_REGISTRY.set(registry);
}

/// The process-wide session registry, if shipd has installed one. Returns
/// `None` in processes that never booted the daemon (CLI, tests) — callers
/// degrade gracefully rather than panic.
pub fn session_registry() -> Option<SessionRegistry> {
    SESSION_REGISTRY.get().cloned()
}

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

/// The scope set for a space window session — the operator's GUI surface
/// **minus the operator marker**. Deliberately excludes `Scope::Full` (which
/// would flip `is_operator` and clear C3) and `Scope::AdminDaemon` (operator
/// territory). The scopes axis is the dying REST route-auth (#78); the gate's
/// real enforcement is the grant axis + `is_operator`, so the only load-bearing
/// property here is "broad, but never Full".
pub fn space_window_scopes() -> Vec<Scope> {
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
        Scope::StepDone,
        Scope::SuiAuthoring,
    ];
    s.extend(pipeline_agent_artifact_scopes());
    s
}

/// Mint a pid-attested **operator cred** — the SOLE source of operator
/// authority (S6, #124). The operator presents this to act at every tier; the
/// gate's `is_operator` reads ONLY a session carrying `is_operator_cred`.
///
/// Bound to `peer_pid` (kernel-verified via `SO_PEERCRED` at register): the
/// cred is two-factor — the secret AND being that exact process. The token is
/// rejected over a transport that can't attest a pid (TCP) by
/// [`DaemonSession::pid_authorizes`], so the operator binding can't be stripped
/// by switching transports — the same defense the window token relies on.
///
/// This is the shell's pid-attested session today; a cloud-issued operator cred
/// can later plug in through its own verified minter, also setting
/// `is_operator_cred`. `Scope::Full` rides along for the dying REST route-auth
/// axis (#78), but it is the flag — not `Scope::Full` — that confers operator.
///
/// Ephemeral by construction: the operator's shell session lives for its run and
/// is never persisted (a persisted session hydrates with `is_operator_cred:
/// false`; the shell re-registers and re-mints on launch).
pub fn operator_session(peer_pid: u32, token: String) -> DaemonSession {
    DaemonSession {
        token,
        // The operator's own session — not an agent.
        agent_id: "operator".to_string(),
        // Full rides the dying REST axis; the operator flag (not Full) confers
        // operator at the gate.
        scopes: vec![Scope::Full],
        // The operator clears the gate's grant axis — no per-service grants.
        grants: vec![],
        workspace_id: None,
        // The operator session is space-less; emits key None (no home-space
        // fallback, S2). A space the operator works inside arrives via a
        // separate space-window token.
        space_id: None,
        // Pid-bound: the cred is two-factor (secret + this exact process).
        pid: Some(peer_pid),
        created_at: std::time::Instant::now(),
        expires_at_unix: None,
        ephemeral: true,
        // THE elevation source — the one verified operator cred.
        is_operator_cred: true,
    }
}

/// Mint a pid-bound, C2-capped session for a space window.
///
/// The token represents the operator working *inside a space*: it can do
/// anything at C2 (the operator's own elevated set) but is **structurally
/// incapable of C3**. It is never `is_operator` — it carries no operator cred
/// (`is_operator_cred: false`), the SOLE elevation source (S6, #124) — so
/// `gate::decide` denies every C3 op regardless of grants. C3 only ever happens
/// through a verified operator cred; a space token can never reach it.
///
/// Bound to `peer_pid` (kernel-verified via `SO_PEERCRED` at register): the
/// token is two-factor — the secret AND being that exact process. A leaked
/// token replayed from another process fails [`DaemonSession::pid_authorizes`].
///
/// Ephemeral by construction: window tokens live for the window's run and are
/// never persisted (a persisted session hydrates with `pid: None`; a window
/// re-registers and re-mints on launch).
pub fn space_window_session(space_id: &str, peer_pid: u32, token: String) -> DaemonSession {
    DaemonSession {
        token,
        // The space identity. The window is C2-capped because it carries no
        // operator cred (`is_operator_cred: false` below) — operator authority
        // derives SOLELY from a verified operator cred (S6, #124), never from a
        // caller-kind/agent-id shape. ("contributor" grants below are the
        // operator's own profile; the space is the operator's own surface.)
        agent_id: format!("space:{space_id}"),
        scopes: space_window_scopes(),
        // Every C2 elevated service — the operator's own profile. C3 stays
        // denied structurally (is_operator is false).
        grants: grants_for_role(Some("contributor")),
        workspace_id: None,
        // The window's OWN space — the load-bearing line for multi-space. Events
        // this session emits are stamped with this space, not the daemon's home
        // space. See spec `.artifacts/specs/space/128-multi-space-emit.md`.
        space_id: Some(space_id.to_string()),
        pid: Some(peer_pid),
        created_at: std::time::Instant::now(),
        expires_at_unix: None,
        ephemeral: true,
        // A space window is C2-capped — NEVER the operator (the load-bearing
        // invariant). The operator cred is the separate `operator_session`.
        is_operator_cred: false,
    }
}

/// Mint and register a scoped session token programmatically.
///
/// Both the HTTP `session_register` handler and the sandbox supervisor
/// call this. The supervisor mints a **per-process** token at sandbox
/// prepare and pushes it into the actor via `Processes.Create.env` — the
/// root daemon token never enters a container. The session is bound to
/// `workspace_id`, which the daemon's per-project guard enforces.
///
/// `ttl_secs = Some(n)` expires the session n seconds out; `None` = no
/// expiry. Minted sessions are ephemeral (in-memory only): the supervisor
/// mints fresh per process and drops the token on actor exit, so there's
/// nothing to persist.
pub async fn mint_session(
    registry: &SessionRegistry,
    agent_id: String,
    workspace_id: Option<String>,
    role: Option<&str>,
    ttl_secs: Option<i64>,
) -> Result<(String, Vec<Scope>), getrandom::Error> {
    let scopes = scopes_for_role(role);
    let grants = grants_for_role(role);
    let token = generate_session_token()?;
    let session = DaemonSession {
        token: token.clone(),
        agent_id,
        scopes: scopes.clone(),
        grants,
        workspace_id,
        space_id: None,
        pid: None,
        created_at: std::time::Instant::now(),
        expires_at_unix: ttl_secs.map(|s| now_unix() + s),
        ephemeral: true,
        // A per-process agent token is never the operator — it's a sandboxed
        // actor's scoped session.
        is_operator_cred: false,
    };
    registry.lock().await.insert(token.clone(), session);
    Ok((token, scopes))
}

/// Remove a minted session from the registry — called when a sandbox
/// actor exits so its per-process token stops working immediately.
pub async fn drop_session(registry: &SessionRegistry, token: &str) {
    registry.lock().await.remove(token);
}

fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
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

    fn session_bound_to(pid: Option<u32>) -> DaemonSession {
        DaemonSession {
            token: "ship_sess_x".to_string(),
            agent_id: "win".to_string(),
            scopes: vec![],
            grants: vec![],
            workspace_id: None,
            space_id: None,
            pid,
            created_at: std::time::Instant::now(),
            expires_at_unix: None,
            ephemeral: true,
            is_operator_cred: false,
        }
    }

    #[test]
    fn operator_session_is_the_pid_attested_operator_cred() {
        let s = operator_session(4242, "ship_sess_op".to_string());
        // The one elevation source: an operator cred, pid-attested.
        assert!(s.is_operator_cred, "operator_session is THE operator cred");
        assert_eq!(s.pid, Some(4242), "the cred is bound to the shell process");
        assert!(s.pid_authorizes(Some(4242)), "the bound process is authorized");
        assert!(!s.pid_authorizes(Some(1)), "a stolen cred from another process is rejected");
        assert!(!s.pid_authorizes(None), "fails closed over an unattested transport (TCP)");
        // Ephemeral — pid-attested creds are never persisted.
        assert!(s.ephemeral, "operator creds are never persisted");
        // The operator clears the grant axis (no per-service grants needed).
        assert!(s.grants.is_empty(), "operator clears the gate's grant axis");
    }

    #[test]
    fn space_window_session_is_pid_bound_c2_never_operator() {
        let s = space_window_session("acme-space", 4242, "ship_sess_win".to_string());
        // Bound to the registering process.
        assert_eq!(s.pid, Some(4242));
        assert!(s.pid_authorizes(Some(4242)));
        assert!(!s.pid_authorizes(Some(1)));
        // Structurally never the operator: no Full scope (Full would flip
        // is_operator and clear C3).
        assert!(
            !s.scopes.contains(&Scope::Full),
            "a space window token must never carry Scope::Full"
        );
        assert!(
            !s.scopes.contains(&Scope::AdminDaemon),
            "a space window token must not carry AdminDaemon"
        );
        // Has the full C2 elevated set (operator's own profile).
        assert_eq!(s.grants, grants_for_role(Some("contributor")));
        assert!(s.ephemeral, "window tokens are never persisted");
        // NEVER the operator cred — that is the separate `operator_session`.
        assert!(!s.is_operator_cred, "a space window is C2-capped, never the operator cred");
    }

    #[test]
    fn unbound_session_passes_regardless_of_peer_pid() {
        // CLI daemon token, agent peer session, legacy session — not pid-bound.
        let s = session_bound_to(None);
        assert!(s.pid_authorizes(Some(1234)));
        assert!(s.pid_authorizes(None));
    }

    #[test]
    fn bound_session_requires_matching_peer_pid() {
        let s = session_bound_to(Some(4242));
        assert!(s.pid_authorizes(Some(4242)), "the bound process is authorized");
        assert!(!s.pid_authorizes(Some(9999)), "a different process is rejected (stolen token)");
    }

    #[test]
    fn bound_session_fails_closed_without_peer_pid() {
        // A pid-bound window token presented over a transport that can't
        // attest a pid (TCP, no SO_PEERCRED) must be rejected — the binding
        // can't be stripped by switching transports.
        let s = session_bound_to(Some(4242));
        assert!(!s.pid_authorizes(None));
    }

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

    #[tokio::test]
    async fn mint_session_registers_scoped_bound_token() {
        let registry: SessionRegistry =
            Arc::new(Mutex::new(std::collections::HashMap::new()));
        let (token, scopes) = mint_session(
            &registry,
            "rust-lane".into(),
            Some("proj-a".into()),
            Some("worker"),
            Some(3600),
        )
        .await
        .unwrap();

        assert!(is_session_token(&token));
        assert!(Scope::ReadWorkspace.is_granted_by(&scopes));
        assert!(!Scope::Full.is_granted_by(&scopes)); // per-process != root

        let reg = registry.lock().await;
        let session = reg.get(&token).expect("registered");
        assert_eq!(session.workspace_id.as_deref(), Some("proj-a"));
        assert_eq!(session.agent_id, "rust-lane");
        assert!(session.ephemeral);
        assert!(session.expires_at_unix.is_some());
    }

    #[tokio::test]
    async fn global_registry_set_then_mint_visible() {
        // Order-independent across the test binary: whoever wins the
        // OnceLock::set, `session_registry()` returns *a* registry; minting
        // into that handle and reading it back must hold.
        let reg: SessionRegistry = Arc::new(Mutex::new(std::collections::HashMap::new()));
        set_session_registry(reg.clone());
        let installed = session_registry().expect("a registry is installed");
        let (token, _) =
            mint_session(&installed, "a".into(), Some("p".into()), Some("worker"), None)
                .await
                .unwrap();
        assert!(installed.lock().await.contains_key(&token));
    }

    #[tokio::test]
    async fn drop_session_invalidates_token() {
        let registry: SessionRegistry =
            Arc::new(Mutex::new(std::collections::HashMap::new()));
        let (token, _) =
            mint_session(&registry, "a".into(), None, Some("worker"), None)
                .await
                .unwrap();
        assert!(registry.lock().await.contains_key(&token));
        drop_session(&registry, &token).await;
        assert!(!registry.lock().await.contains_key(&token));
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
