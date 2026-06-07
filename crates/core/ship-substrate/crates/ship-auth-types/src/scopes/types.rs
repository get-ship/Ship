//! Core scope/required-auth value types — the `Scope` enum, the MCP
//! identity projection, and the route-level `RequiredAuth` tag.
//!
//! Pure data + a single helper (`Scope::is_granted_by`). The role→scope
//! and role→grant tables live in `super::roles`; the session type and
//! registry live in `super::session`.

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
