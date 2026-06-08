//! Session type, registry, token formatters, and session constructors.
//!
//! This is the L0 session surface: pure data, the in-memory registry, and
//! the minters the L1 platform reuses (operator cred, space-window token,
//! per-process agent token). The L1 policy — grant resolution, session
//! persistence/hydration, the sweeper, the pairing flow — lives in
//! `ship_platform::auth::scopes`.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::roles::{grants_for_role, scopes_for_role};
use super::types::Scope;

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
    s.extend(super::roles::pipeline_agent_artifact_scopes());
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
