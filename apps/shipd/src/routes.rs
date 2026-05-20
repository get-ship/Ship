//! Canonical Ship route registry.
//!
//! Parsed at build time by `apps/shipd/build.rs` to emit two artifacts:
//!
//! - `$OUT_DIR/routes_generated.rs` — Rust URL builders (`Route::method()`,
//!   `Route::full_path()`) and the `ShipRouterExt::ship_route(Route, handler)`
//!   trait extension used by `lib.rs` to register routes.
//! - `packages/sdk/src/generated/routes.ts` — typed TypeScript route
//!   catalog for SDK / web consumers. Relative URLs (cookie-authenticated).
//!
//! # Variant syntax
//!
//! Each variant MUST carry a `@route` doc annotation. Optional annotations
//! describe the typed request/response surface and are used for codegen
//! only — Axum's own `Handler` trait enforces the Rust handler signature at
//! the `ship_route(Route::X, handler)` call site.
//!
//! ```ignore
//! /// @route METHOD "/path/with/{params}"
//! /// @returns Vec<PipelineRun>     (optional — TS gets typed fetch wrapper)
//! /// @body   CreateJobRequest    (optional — TS gets typed request body)
//! ```
//!
//! Path params (`{id}`) MUST match a String-typed field on the variant.
//! `build.rs` panics with a clear message if these drift.
//!
//! Type allowlist for `@returns` / `@body`: plain identifier (`Spec`),
//! `Vec<T>`, `Option<T>`, where T is a plain identifier. Anything else is a
//! build-time error.

#![allow(dead_code)] // Variants without handlers are still used by codegen.

// ── Route handler submodules ────────────────────────────────────────────
//
// Handler files live at `routes/<x>_api.rs`. Callers import them as
// `crate::routes::<x>_api::*` (or `shipd::routes::<x>_api::*` externally).

pub mod account_api;
pub mod actor_api;
#[cfg(test)]
mod actor_api_tests;
pub mod actors_api;
pub mod agent_sessions_attach;
#[cfg(test)]
mod agent_sessions_attach_tests;
pub mod mcp_registry;
pub mod agents_api;
pub mod analytics_api;
pub mod annotations_api;
pub mod artifacts_api;
pub mod collection_api;
pub mod commands_api;
pub mod create_api;
#[cfg(test)]
mod create_api_tests;
pub mod feedback_api;
pub mod git_api;
pub mod hooks_api;
pub mod human_webhook;
pub mod logs_api;
pub mod mcp_api;
pub mod openapi;
pub mod permissions_api;
pub mod pipelines_api;
pub mod rules_api;
pub mod skills_api;
pub mod openapi_modifier;
pub mod openapi_types;
pub mod projects_api;
pub mod registry_api;
pub mod rest_api;
pub mod runtime_api;
pub mod sdk_manifest;
pub mod search_api;
#[cfg(test)]
mod search_api_tests;
#[cfg(test)]
mod scope_enforcement_tests;
pub mod settings_api;
pub mod specs_api;
pub mod themes_api;
pub mod view_edit_mode_api;
pub mod view_edit_tool_api;
pub mod view_errors_api;
pub mod view_sessions_api;
pub mod webhook_ext;

// The build script writes generated impls into OUT_DIR. We include them here
// so `Route::full_path()`, `Route::method()`, and `ShipRouterExt` are
// visible to the rest of the crate.
include!(concat!(env!("OUT_DIR"), "/routes_generated.rs"));

/// Canonical list of every HTTP route registered by the daemon.
///
/// Variants with path params must keep those params as `String` fields so
/// URL builders can substitute them. Variants without params are unit
/// variants.
pub enum Route {
    // ── First-run / setup ────────────────────────────────────────────────

    /// @route GET "/"
    /// @auth public
    Index,

    /// @route GET "/setup"
    /// @auth public
    Setup,

    // ── Session / auth ───────────────────────────────────────────────────

    /// @route POST "/api/v1/session"
    /// @returns SessionIssueResponse
    /// @auth scope Full
    IssueSession,

    // ── Account — gateway-mediated Ship Cloud identity ────────────────────
    // All routes here require CallerKind::Cli (user-presence). MCP session
    // tokens are categorically denied — see SECURITY.md. Daemon proxies to
    // the gateway via ship-gateway-client; CLI never touches the gateway
    // directly.

    /// @route POST "/api/v1/account/login"
    /// @auth cli
    StartAccountLogin,

    /// @route POST "/api/v1/account/login/complete"
    /// @auth cli
    CompleteAccountLogin,

    /// @route GET "/api/v1/account/state"
    /// @auth cli
    GetAccountState,

    /// @route POST "/api/v1/account/logout"
    /// @auth cli
    LogoutAccount,

    /// Transitional bridge — returns the stored API key for legacy
    /// CLI call sites (publish, convert) that still hit the gateway
    /// directly. Remove once those route through the daemon.
    /// @route GET "/api/v1/account/key"
    /// @auth cli
    GetAccountKey,

    // ── Shared assets (unauthenticated) ──────────────────────────────────

    /// @route GET "/api/assets/view-tokens.css"
    /// @auth public
    AssetViewTokensCss,

    /// @route GET "/api/assets/theme.css"
    /// @auth public
    AssetThemeCss,

    /// @route GET "/api/v1/themes"
    /// @auth public
    ListThemes,

    // ── SUI framework hosting (ship-sui-server handlers) ──

    /// @route GET "/api/sui/catalog/workspace"
    /// @auth public
    SuiCatalogWorkspace,

    /// @route GET "/api/sui/catalog/published"
    /// @auth public
    SuiCatalogPublished,

    /// @route GET "/_sui/packs/{pack_id}/{file}"
    /// @auth public
    SuiPackAsset { pack_id: String, file: String },

    /// @route GET "/_sui/runtime.js"
    /// @auth public
    SuiRuntimeJs,

    /// @route GET "/_sui/dom.js"
    /// @auth public
    SuiDomJs,

    /// @route GET "/_sui/sdk-client.js"
    /// @auth public
    SuiSdkClientJs,

    /// @route GET "/_sui/components.js"
    /// @auth public
    SuiComponentsJs,

    /// @route GET "/_sui/lucide-icons.json"
    /// @auth public
    SuiLucideIconsJson,

    /// @route GET "/_sui/sui-components/{file}"
    /// @auth public
    SuiComponentBundle { file: String },

    /// @route GET "/_sui/views/{name}/view.sui"
    /// @auth public
    SuiViewTemplate { name: String },

    /// @route GET "/_sui/views/{name}/view.js"
    /// @auth public
    SuiViewSetup { name: String },

    /// @route GET "/_sui/views/{name}"
    /// @auth public
    SuiViewPayload { name: String },

    /// @route GET "/_sui/views/{name}/view.css"
    /// @auth public
    SuiViewCss { name: String },

    /// @route GET "/_sui/views/{name}/sse"
    /// @auth public
    SuiViewSse { name: String },

    /// @route GET "/view-host"
    /// @auth public
    SuiViewHost,

    /// @route GET "/sui/{name}"
    /// @auth public
    SuiViewHostForName { name: String },

    /// @route GET "/_sui/hmr"
    /// @auth public
    SuiHmr,

    /// @route GET "/_sui/hmr-client.js"
    /// @auth public
    SuiHmrClientJs,

    /// @route GET "/_sui/dev/hmr"
    /// @auth public
    SuiDevHmr,

    /// @route GET "/_sui/data"
    /// @auth public
    SuiData,

    /// @route POST "/_sui/mutation/{name}"
    /// @auth public
    SuiMutation { name: String },

    /// @route POST "/_sui/views/{name}/patch-elements"
    /// @auth scope Full
    SuiPatchElements { name: String },

    /// @route GET "/_sui/source/{tag}"
    /// @auth public
    SuiSource { tag: String },

    // ── AgentService — service-shaped CRUD + lifecycle ──

    /// @route GET "/api/v1/agents"
    /// @auth scope ReadWorkspace
    AgentsList,

    /// @route GET "/api/v1/agents/{id}"
    /// @auth scope ReadWorkspace
    AgentsGet { id: String },

    /// @route PUT "/api/v1/agents/{id}"
    /// @auth scope WriteWorkspace
    AgentsPut { id: String },

    /// @route PATCH "/api/v1/agents/{id}"
    /// @auth scope WriteWorkspace
    AgentsPatch { id: String },

    /// @route DELETE "/api/v1/agents/{id}"
    /// @auth scope WriteWorkspace
    AgentsDelete { id: String },

    /// @route POST "/api/v1/agents"
    /// @auth scope CreateJobs
    AgentsCreate,

    /// @route POST "/api/v1/agents/{id}/rename"
    /// @auth scope WriteWorkspace
    AgentsRename { id: String },

    /// @route POST "/api/v1/agents/{id}/duplicate"
    /// @auth scope CreateJobs
    AgentsDuplicate { id: String },

    /// @route POST "/api/v1/agents/{id}/move"
    /// @auth scope WriteWorkspace
    AgentsMove { id: String },

    // ── PipelineService — service-shaped CRUD + run ops ──

    /// @route GET "/api/v1/pipelines"
    /// @auth scope ReadJobs
    PipelinesList,

    /// @route GET "/api/v1/pipelines/{name}"
    /// @auth scope ReadJobs
    PipelinesGet { name: String },

    /// @route PUT "/api/v1/pipelines/{name}"
    /// @auth scope WriteWorkspace
    PipelinesPut { name: String },

    /// @route PATCH "/api/v1/pipelines/{name}"
    /// @auth scope WriteWorkspace
    PipelinesPatch { name: String },

    /// @route DELETE "/api/v1/pipelines/{name}"
    /// @auth scope WriteWorkspace
    PipelinesDelete { name: String },

    /// @route POST "/api/v1/pipelines/{name}/dispatch"
    /// @auth scope CreateJobs
    PipelinesDispatch { name: String },

    /// @route POST "/api/v1/pipelines/run"
    /// @auth scope CreateJobs
    PipelinesRun,

    /// @route POST "/api/v1/pipelines/runs/{slug}/cancel"
    /// @auth scope ManageJobs
    PipelineRunCancel { slug: String },

    /// @route GET "/api/v1/pipelines/runs"
    /// @auth scope ReadJobs
    PipelineRunsList,

    /// @route GET "/api/v1/pipelines/runs/{slug}"
    /// @auth scope ReadJobs
    PipelineRunGet { slug: String },

    /// @route GET "/api/v1/pipelines/runs/{slug}/log/stream"
    /// @auth scope ReadJobs
    PipelineRunLogStream { slug: String },

    // ── RuleService — markdown rules at <scope>/rules/<id>.md ──

    /// @route GET "/api/v1/rules"
    /// @auth scope ReadWorkspace
    RulesList,

    /// @route GET "/api/v1/rules/{id}"
    /// @auth scope ReadWorkspace
    RulesGet { id: String },

    /// @route PUT "/api/v1/rules/{id}"
    /// @auth scope WriteWorkspace
    RulesPut { id: String },

    /// @route DELETE "/api/v1/rules/{id}"
    /// @auth scope WriteWorkspace
    RulesDelete { id: String },

    /// @route POST "/api/v1/rules"
    /// @auth scope CreateJobs
    RulesCreate,

    // ── McpService — entries in <scope>/mcp.jsonc ──

    /// @route GET "/api/v1/mcp"
    /// @auth scope ReadWorkspace
    McpList,

    /// @route GET "/api/v1/mcp/{id}"
    /// @auth scope ReadWorkspace
    McpGet { id: String },

    /// @route PUT "/api/v1/mcp/{id}"
    /// @auth scope WriteWorkspace
    McpPut { id: String },

    /// @route PATCH "/api/v1/mcp/{id}"
    /// @auth scope WriteWorkspace
    McpPatch { id: String },

    /// @route DELETE "/api/v1/mcp/{id}"
    /// @auth scope WriteWorkspace
    McpDelete { id: String },

    /// @route POST "/api/v1/mcp"
    /// @auth scope CreateJobs
    McpCreate,

    // ── PermissionService — preset entries in <scope>/permissions.jsonc ──

    /// @route GET "/api/v1/permissions"
    /// @auth scope ReadWorkspace
    PermissionsList,

    /// @route GET "/api/v1/permissions/tool-patterns"
    /// @auth scope ReadWorkspace
    PermissionsToolPatterns,

    /// @route GET "/api/v1/permissions/{id}"
    /// @auth scope ReadWorkspace
    PermissionsGet { id: String },

    /// @route PUT "/api/v1/permissions/{id}"
    /// @auth scope WriteWorkspace
    PermissionsPut { id: String },

    /// @route PATCH "/api/v1/permissions/{id}"
    /// @auth scope WriteWorkspace
    PermissionsPatch { id: String },

    /// @route DELETE "/api/v1/permissions/{id}"
    /// @auth scope WriteWorkspace
    PermissionsDelete { id: String },

    /// @route POST "/api/v1/permissions"
    /// @auth scope CreateJobs
    PermissionsCreate,

    // ── SkillService — service-shaped CRUD + vars ──

    /// @route GET "/api/v1/skills"
    /// @auth scope ReadWorkspace
    SkillsList,

    /// @route GET "/api/v1/skills/{id}"
    /// @auth scope ReadWorkspace
    SkillsGet { id: String },

    /// @route POST "/api/v1/skills"
    /// @auth scope CreateJobs
    SkillsCreate,

    /// @route DELETE "/api/v1/skills/{id}"
    /// @auth scope WriteWorkspace
    SkillsDelete { id: String },

    /// @route PUT "/api/v1/skills/{id}/frontmatter"
    /// @auth scope WriteWorkspace
    SkillsPutFrontmatter { id: String },

    /// @route PUT "/api/v1/skills/{id}/body"
    /// @auth scope WriteWorkspace
    SkillsPutBody { id: String },

    /// @route POST "/api/v1/skills/{id}/rename"
    /// @auth scope WriteWorkspace
    SkillsRename { id: String },

    /// @route POST "/api/v1/skills/{id}/duplicate"
    /// @auth scope CreateJobs
    SkillsDuplicate { id: String },

    /// @route POST "/api/v1/skills/{id}/move"
    /// @auth scope WriteWorkspace
    SkillsMove { id: String },

    /// @route GET "/api/v1/skills/{id}/vars"
    /// @auth scope ReadWorkspace
    SkillsGetVars { id: String },

    /// @route PUT "/api/v1/skills/{id}/vars/{key}"
    /// @auth scope WriteWorkspace
    SkillsPutVar { id: String, key: String },

    /// @route DELETE "/api/v1/skills/{id}/vars/{key}"
    /// @auth scope WriteWorkspace
    SkillsDeleteVar { id: String, key: String },

    /// @route POST "/api/v1/skills/{id}/invoke"
    /// @auth scope ManageJobs
    SkillsInvoke { id: String },

    /// @route POST "/api/v1/chat/send"
    /// @auth scope ManageJobs
    ChatSend,

    // ── Generic view serving ─────────────────────────────────────────────

    /// @route GET "/api/v1/views"
    /// @auth scope ReadWorkspace
    ListViews,

    /// @route GET "/api/v1/ship/schema"
    /// @auth scope ReadWorkspace
    ShipSchema,

    /// @route GET "/api/v1/view-source/{*path}"
    /// @auth scope ReadWorkspace
    ServeViewSource { path: String },

    /// @route GET "/api/v1/views/{name}/files"
    /// @auth scope ReadWorkspace
    ListViewFiles { name: String },

    /// @route POST "/api/v1/create/{surface}"
    /// @auth scope CreateJobs
    CreateSurface { surface: String },

    /// @route PUT "/api/v1/views/{name}/template"
    /// @auth scope WriteWorkspace
    WriteViewTemplate { name: String },

    /// @route PUT "/api/v1/views/{name}/manifest"
    /// @auth scope WriteWorkspace
    WriteViewManifestApi { name: String },

    /// @route POST "/api/v1/views/{name}/fork"
    /// @auth scope WriteWorkspace
    ForkView { name: String },

    /// @route POST "/api/v1/themes/contrast"
    /// @auth scope ReadWorkspace
    ThemeContrast,

    /// @route POST "/api/v1/sessions/spawn"
    /// @auth scope ManageJobs
    SpawnPtySession,

    /// @route GET "/api/v1/pty-sessions"
    /// @auth scope ReadJobs
    ListPtySessions,

    /// @route GET "/api/v1/sessions/{id}"
    /// @auth scope ReadJobs
    GetPtySession { id: String },

    /// @route DELETE "/api/v1/sessions/{id}"
    /// @auth scope ManageJobs
    StopPtySession { id: String },

    /// @route POST "/api/v1/sessions/{id}/input"
    /// @auth scope ManageJobs
    PtySessionInput { id: String },

    /// @route POST "/api/v1/sessions/{id}/resize"
    /// @auth scope ManageJobs
    PtySessionResize { id: String },

    /// @route POST "/api/v1/sessions/{id}/message"
    /// @auth scope ManageJobs
    PtySessionMessage { id: String },

    /// @route POST "/api/v1/sessions/{id}/reply"
    /// @auth scope ManageJobs
    PtySessionReply { id: String },

    /// @route GET "/api/v1/sessions/{id}/stream"
    /// @auth scope ManageJobs
    StreamPtySession { id: String },

    /// @route GET "/api/v1/sessions/{id}/events"
    /// @auth scope ReadJobs
    StreamPtySessionEvents { id: String },

    /// @route GET "/api/v1/harnesses/installed"
    /// @auth scope ReadWorkspace
    InstalledProviders,

    /// @route GET "/api/v1/projects"
    /// @auth scope ReadWorkspace
    ListProjects,

    /// @route POST "/api/v1/projects"
    /// @auth scope WriteWorkspace
    AddProject,

    /// @route POST "/api/v1/projects/switch"
    /// @auth scope WriteWorkspace
    SwitchProject,

    /// @route POST "/api/v1/projects/{id}/activate"
    /// @auth scope WriteWorkspace
    ActivateProject { id: String },

    /// @route GET "/api/v1/projects/active"
    /// @auth scope ReadWorkspace
    GetActiveProject,

    /// @route GET "/api/v1/sessions/active-across-projects"
    /// @auth scope ReadWorkspace
    ListSessionsAcrossProjects,

    /// @route GET "/api/v1/view/{name}/manifest"
    /// @auth scope ReadWorkspace
    ServeViewManifest { name: String },

    /// @route GET "/api/v1/view/{name}/bundle"
    /// @auth scope ReadWorkspace
    ServeViewBundle { name: String },

    // ── Agent-owned view sessions (Phase 1 canvas framework) ─────────────

    /// @route POST "/api/v1/view-sessions"
    /// @auth scope ManageJobs
    OpenViewSession,

    /// @route POST "/api/v1/view-sessions/{id}/patch"
    /// @auth scope ManageJobs
    PatchViewSession { id: String },

    /// @route POST "/api/v1/view-sessions/{id}/append"
    /// @auth scope ManageJobs
    AppendViewSession { id: String },

    /// @route POST "/api/v1/view-sessions/{id}/signal"
    /// @auth scope ManageJobs
    SignalViewSession { id: String },

    /// @route DELETE "/api/v1/view-sessions/{id}"
    /// @auth scope ManageJobs
    CloseViewSession { id: String },

    /// @route GET "/api/v1/view-sessions/{id}"
    /// @auth scope ReadWorkspace
    SubscribeViewSession { id: String },

    // ── Feedback (agent-designed feedback ingress) ────────────────────────

    /// @route POST "/api/v1/feedback"
    /// @auth scope LogProgress
    PostFeedback,

    // ── Annotations (canvas overlay wire format) ─────────────────────────

    /// @route POST "/api/v1/annotations"
    /// @auth scope LogProgress
    PostAnnotation,

    // ── Runtime listings ─────────────────────────────────────────────────

    /// @route GET "/api/v1/sessions"
    /// @auth scope ReadWorkspace
    ListSessions,

    /// @route GET "/api/v1/agents/active"
    /// @auth scope ReadWorkspace
    ListAgents,

    /// @route POST "/api/v1/agents/{slug}/stop"
    /// @auth scope ManageJobs
    StopAgent { slug: String },

    /// @route POST "/api/v1/agents/{slug}/steer"
    /// @auth scope ManageJobs
    SteerAgent { slug: String },

    // ── Jobs ─────────────────────────────────────────────────────────────

    /// @route GET "/api/v1/agent_sessions"
    /// @auth scope ReadJobs
    ListJobs,

    /// @route POST "/api/v1/agent_sessions/dispatch"
    /// @auth scope CreateJobs
    DispatchPipeline,

    /// @route GET "/api/v1/agent_sessions/{pipeline_run_id}"
    /// @auth scope ReadJobs
    GetJob { pipeline_run_id: String },

    /// @route POST "/api/v1/agent_sessions/end"
    /// @auth scope ManageJobs
    EndJob,

    /// @route POST "/api/v1/agent_sessions/{slug}/cancel"
    /// @auth scope ManageJobs
    CancelJob { slug: String },

    /// @route POST "/api/v1/agent_sessions/{pipeline_run_id}/status"
    /// @auth scope EmitEvents
    JobStatusUpdate { pipeline_run_id: String },

    /// @route POST "/api/v1/agent_sessions/{slug}/step-done"
    /// @auth scope EmitEvents
    JobStepDone { slug: String },

    /// @route GET "/api/v1/agent_sessions/{slug}/log/stream"
    /// @auth scope ReadJobs
    StreamJobLog { slug: String },

    /// @route GET "/api/v1/agent_sessions/{slug}/attach"
    /// @auth scope WriteWorkspace
    AttachAgentSession { slug: String },

    // ── Process projection / events polling / daemon status (slice 6) ────

    /// @route GET "/api/v1/procs"
    /// @auth scope ReadWorkspace
    ListProcs,

    /// @route GET "/api/v1/events/poll"
    /// @auth scope ReadWorkspace
    PollEvents,

    /// @route GET "/api/v1/daemon/status"
    /// @auth scope ReadWorkspace
    DaemonStatus,

    // ── Specs ────────────────────────────────────────────────────────────

    /// @route GET "/api/v1/specs"
    /// @auth scope ReadJobs
    ListSpecs,

    /// @route POST "/api/v1/specs"
    /// @auth scope CreateJobs
    CreateSpec,

    /// @route GET "/api/v1/specs/{id}"
    /// @auth scope ReadJobs
    GetSpec { id: String },

    /// @route PUT "/api/v1/specs/{id}"
    /// @auth scope WriteWorkspace
    UpdateSpec { id: String },

    /// @route DELETE "/api/v1/specs/{id}"
    /// @auth scope WriteWorkspace
    DeleteSpec { id: String },

    /// @route POST "/api/v1/specs/{id}/launch"
    /// @auth scope CreateJobs
    LaunchSpec { id: String },

    // ── Events / projections ─────────────────────────────────────────────

    /// @route GET "/api/v1/events/stream"
    /// @auth scope ReadWorkspace
    EventStream,

    /// @route POST "/api/v1/rebuild-projections"
    /// @auth scope AdminDaemon
    RebuildProjections,

    /// @route GET "/api/v1/projects/{id}/pty"
    /// @auth scope ManageJobs
    ProjectPty { id: String },

    // ── Settings / harnesses ─────────────────────────────────────────────

    /// @route GET "/api/v1/settings"
    /// @auth scope ReadWorkspace
    ListSettings,

    /// @route PUT "/api/v1/settings"
    /// @auth scope AdminDaemon
    UpdateSetting,

    /// @route GET "/api/v1/settings/{key}"
    /// @auth scope ReadWorkspace
    GetSetting { key: String },

    /// @route GET "/api/v1/harnesses"
    /// @auth scope ReadWorkspace
    ListProviders,

    // ── Supervisor ───────────────────────────────────────────────────────

    // ── Project-scoped ───────────────────────────────────────────────────

    /// @route GET "/api/v1/projects/{id}/session-files"
    /// @auth scope ReadWorkspace
    ListSessionFiles { id: String },

    /// @route GET "/api/v1/projects/{id}/agents"
    /// @auth scope ReadWorkspace
    ListProjectAgents { id: String },

    // ── GitService — canonical `/api/git/*` surface ──────────────────────

    /// @route GET "/api/v1/git/status"
    /// @auth scope ReadWorkspace
    GitStatus,

    /// @route GET "/api/v1/git/diff"
    /// @auth scope ReadWorkspace
    GitDiff,

    /// @route GET "/api/v1/git/log"
    /// @auth scope ReadWorkspace
    GitLog,

    /// @route GET "/api/v1/git/branches"
    /// @auth scope ReadWorkspace
    GitBranches,

    /// @route POST "/api/v1/git/stage"
    /// @auth scope WriteWorkspace
    GitStage,

    /// @route POST "/api/v1/git/unstage"
    /// @auth scope WriteWorkspace
    GitUnstage,

    /// @route POST "/api/v1/git/commit"
    /// @auth scope WriteWorkspace
    GitCommit,

    /// @route POST "/api/v1/git/push"
    /// @auth scope WriteWorkspace
    GitPush,

    /// @route POST "/api/v1/git/fetch"
    /// @auth scope ReadWorkspace
    GitFetch,

    /// @route POST "/api/v1/git/pull"
    /// @auth scope WriteWorkspace
    GitPull,

    /// @route POST "/api/v1/git/checkout"
    /// @auth scope WriteWorkspace
    GitCheckout,

    /// @route POST "/api/v1/git/branch"
    /// @auth scope WriteWorkspace
    GitBranchCreate,

    /// @route DELETE "/api/v1/git/branch"
    /// @auth scope WriteWorkspace
    GitBranchDelete,

    /// @route POST "/api/v1/git/init"
    /// @auth scope WriteWorkspace
    GitInit,

    /// @route GET "/api/v1/git/sse"
    /// @auth scope ReadWorkspace
    GitSse,

    /// @route POST "/api/v1/events/emit"
    /// @auth scope EmitEvents
    EmitEvent,

    // Workspace-scoped artifact status/promote removed — callers get 404.
    // Use the unified /api/artifacts/* surface instead.

    // ── Mesh / actors ────────────────────────────────────────────────────

    /// @route POST "/api/v1/mesh/register"
    /// @auth scope MeshSend
    MeshRegister,

    /// @route POST "/api/v1/mesh/send"
    /// @auth scope MeshSend
    MeshSend,

    /// @route POST "/api/v1/mesh/broadcast"
    /// @auth scope MeshBroadcast
    MeshBroadcast,

    /// @route GET "/api/v1/mesh/discover"
    /// @auth scope ReadWorkspace
    MeshDiscover,

    /// @route POST "/api/v1/mesh/status"
    /// @auth scope MeshSend
    MeshStatusUpdate,

    /// @route GET "/api/v1/mesh/events/{agent_id}"
    /// @auth scope ReadWorkspace
    MeshEvents { agent_id: String },

    /// @route POST "/api/v1/actor/spawn"
    /// @auth scope MeshSend
    ActorSpawn,

    /// @route POST "/api/v1/events/route"
    /// @auth scope EmitEvents
    EventRoute,

    /// @route GET "/api/v1/actors"
    /// @auth scope ReadWorkspace
    ListActors,

    /// @route GET "/api/v1/mesh"
    /// @auth scope ReadWorkspace
    ListMesh,

    /// @route GET "/api/v1/actors/{actor_id}"
    /// @auth scope ReadWorkspace
    GetActor { actor_id: String },

    /// @route GET "/api/v1/actors/{actor_id}/events"
    /// @auth scope ReadWorkspace
    ActorEvents { actor_id: String },

    /// @route POST "/api/v1/actors/{actor_id}/input"
    /// @auth scope MeshSend
    ActorInput { actor_id: String },

    /// @route GET "/api/v1/mesh/snapshot"
    /// @auth scope ReadWorkspace
    MeshSnapshot,

    /// @route GET "/api/v1/mesh/events"
    /// @auth scope ReadWorkspace
    MeshEventsStream,

    /// @route POST "/api/v1/actors/{actor_id}/kill-tree"
    /// @auth scope MeshSend
    KillTree { actor_id: String },

    // ── Analytics (ship-analytics projection) ────────────────────────────

    /// @route GET "/api/v1/analytics/sessions"
    /// @auth scope ReadWorkspace
    ListAnalyticsSessions,

    /// @route GET "/api/v1/analytics/sessions/{session_key}"
    /// @auth scope ReadWorkspace
    GetAnalyticsSession { session_key: String },

    /// @route GET "/api/v1/analytics/tool-usage"
    /// @auth scope ReadWorkspace
    AnalyticsToolUsage,

    /// @route GET "/api/v1/analytics/sessions/{session_key}/turns"
    /// @auth scope ReadWorkspace
    AnalyticsSessionTurns { session_key: String },

    // ── Collections ──────────────────────────────────────────────────────

    /// @route GET "/api/v1/collections"
    /// @auth scope ReadWorkspace
    ListCollections,

    /// @route GET "/api/v1/collections/{collection}/schema"
    /// @auth scope ReadWorkspace
    GetCollectionSchema { collection: String },

    /// @route PUT "/api/v1/collections/{collection}/schema"
    /// @auth scope WriteWorkspace
    UpdateCollectionSchema { collection: String },

    /// @route POST "/api/v1/collections/{collection}/items"
    /// @auth scope WriteWorkspace
    CreateCollectionItem { collection: String },

    /// @route GET "/api/v1/collections/{collection}/items/{item_id}"
    /// @auth scope ReadWorkspace
    GetCollectionItem { collection: String, item_id: String },

    /// @route PUT "/api/v1/collections/{collection}/items/{item_id}"
    /// @auth scope WriteWorkspace
    UpdateCollectionItem { collection: String, item_id: String },

    /// @route DELETE "/api/v1/collections/{collection}/items/{item_id}"
    /// @auth scope WriteWorkspace
    DeleteCollectionItem { collection: String, item_id: String },

    /// @route POST "/api/v1/collections/{collection}/query"
    /// @auth scope ReadWorkspace
    QueryCollection { collection: String },

    // ── Artifacts ────────────────────────────────────────────────────────
    // The list/upload/download/metadata/patch/delete handlers compute the
    // required scope dynamically from the artifact's stored `scope` field
    // ("run" / "project" / "user") via can_*_artifacts. The router can't
    // model that statically, so these declare `@auth handler` — the
    // scope_enforcement_tests assert that each handler still calls
    // can_*_artifacts (or require_scope) so the Handler tag can't silently
    // disable enforcement.

    /// @route POST "/api/v1/artifacts"
    /// @auth handler
    ArtifactsUpload,

    /// @route GET "/api/v1/artifacts"
    /// @auth handler
    ArtifactsList,

    /// @route GET "/api/v1/artifacts/{id}"
    /// @auth handler
    ArtifactDownload { id: String },


    /// @route GET "/api/v1/artifacts/{id}/metadata"
    /// @auth handler
    ArtifactMetadata { id: String },

    /// @route PATCH "/api/v1/artifacts/{id}"
    /// @auth handler
    ArtifactPatch { id: String },

    /// @route DELETE "/api/v1/artifacts/{id}"
    /// @auth handler
    ArtifactDelete { id: String },

    /// @route POST "/api/v1/artifacts/{id}/promote"
    /// @auth scope PromoteArtifact
    ArtifactPromote { id: String },

    // ── Capabilities ─────────────────────────────────────────────────────

    /// @route GET "/api/v1/capabilities"
    /// @auth scope ReadWorkspace
    Capabilities,

    // ── Command bar registry ─────────────────────────────────────────────

    /// @route GET "/api/v1/commands"
    /// @auth scope ReadWorkspace
    ListCommands,

    // ── Unified search ───────────────────────────────────────────────────

    /// @route GET "/api/v1/search"
    /// @auth scope ReadWorkspace
    SearchAll,

    // ── SDK manifest (read-side; generated by `just gen-sdk`) ────────────

    /// @route GET "/api/v1/views/{name}/meta"
    /// @auth scope ReadWorkspace
    ViewMeta { name: String },

    /// @route GET "/api/v1/sdk"
    /// @auth scope ReadWorkspace
    SdkRoot,

    /// @route GET "/api/v1/sdk/query"
    /// @auth scope ReadWorkspace
    SdkQuery,

    // ── View client-error capture ────────────────────────────────────────

    /// @route POST "/api/v1/view-errors"
    /// @auth scope ReadWorkspace
    PostViewError,

    /// @route GET "/api/v1/view-errors"
    /// @auth scope ReadWorkspace
    ListViewErrors,

    /// @route DELETE "/api/v1/view-errors"
    /// @auth scope ReadWorkspace
    ClearViewErrors,

    // ── MCP registry proxy ────────────────────────────────────────────────

    /// @route GET "/api/v1/mcp/servers"
    /// @auth public
    McpServers,

    // ── View authoring (chrome edit-mode + agent tool dispatch) ──────────

    /// @route POST "/api/v1/views/{name}/edit-mode/enter"
    /// @auth scope WriteWorkspace
    EnterViewEditMode { name: String },

    /// @route POST "/api/v1/views/{name}/edit-mode/exit"
    /// @auth scope WriteWorkspace
    ExitViewEditMode { name: String },

    /// @route POST "/api/v1/views/{name}/edit/tool"
    /// @auth scope ManageJobs
    DispatchViewEditTool { name: String },

    // ── Registry (gateway-forwarded) ─────────────────────────────────────
    //
    // The daemon is the single hop to the gateway: CLI hits the daemon
    // here, the daemon attaches the user's stored API key and forwards
    // to `<cloud.base_url>/api/v1/registry/publish`. `ship publish` no
    // longer talks to the gateway directly.

    /// @route POST "/api/v1/registry/publish"
    /// @auth cli
    Publish,
}

#[cfg(test)]
mod tests;
