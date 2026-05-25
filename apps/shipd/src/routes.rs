//! Canonical Ship route registry.
//!
//! Parsed at build time by `apps/shipd/build.rs` to emit two artifacts:
//!
//! - `$OUT_DIR/routes_generated.rs` — Rust URL builders (`Route::method()`,
//!   `Route::full_path()`) and the `ShipRouterExt::ship_route(Route, handler)`
//!   trait extension used by `lib.rs` to register routes.
//! - `packages/deck/sdk/src/generated/routes.ts` — typed TypeScript route
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
pub mod annotations_api;
pub mod artifacts_api;
pub mod commands_api;
pub mod create_api;
#[cfg(test)]
mod create_api_tests;
pub mod feedback_api;
pub mod git_api;
pub mod hooks_api;
pub mod human_webhook;
pub mod logs_api;
pub mod openapi;
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

    // ── SUI framework hosting (deck-server handlers) ──

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

    /// @route GET "/_sui/live-reload.js"
    /// @auth public
    SuiLiveReloadJs,

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

    // ── PipelineService — service-shaped CRUD + run ops ──

    /// @route POST "/api/v1/pipelines/run"
    /// @auth scope CreateRuns
    PipelinesRun,

    /// @route GET "/api/v1/pipelines/runs/{slug}/log/stream"
    /// @auth scope ReadRuns
    PipelineRunLogStream { slug: String },

    // ── Chat (command-bar send) ──────────────────────────────────────────

    /// @route POST "/api/v1/chat/send"
    /// @auth scope ManageRuns
    ChatSend,

    // ── Generic view serving ─────────────────────────────────────────────

    /// @route GET "/api/v1/ship/schema"
    /// @auth scope ReadWorkspace
    ShipSchema,

    /// @route GET "/api/v1/views/{name}/files"
    /// @auth scope ReadWorkspace
    ListViewFiles { name: String },

    /// @route POST "/api/v1/create/{surface}"
    /// @auth scope CreateRuns
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
    /// @auth scope ManageRuns
    SpawnPtySession,

    /// @route GET "/api/v1/pty-sessions"
    /// @auth scope ReadRuns
    ListPtySessions,

    /// @route GET "/api/v1/sessions/{id}"
    /// @auth scope ReadRuns
    GetPtySession { id: String },

    /// @route DELETE "/api/v1/sessions/{id}"
    /// @auth scope ManageRuns
    StopPtySession { id: String },

    /// @route POST "/api/v1/sessions/{id}/input"
    /// @auth scope ManageRuns
    PtySessionInput { id: String },

    /// @route POST "/api/v1/sessions/{id}/resize"
    /// @auth scope ManageRuns
    PtySessionResize { id: String },

    /// @route POST "/api/v1/sessions/{id}/message"
    /// @auth scope ManageRuns
    PtySessionMessage { id: String },

    /// @route POST "/api/v1/sessions/{id}/reply"
    /// @auth scope ManageRuns
    PtySessionReply { id: String },

    /// @route GET "/api/v1/sessions/{id}/stream"
    /// @auth scope ManageRuns
    StreamPtySession { id: String },

    /// @route GET "/api/v1/sessions/{id}/events"
    /// @auth scope ReadRuns
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
    /// @auth scope ManageRuns
    OpenViewSession,

    /// @route POST "/api/v1/view-sessions/{id}/patch"
    /// @auth scope ManageRuns
    PatchViewSession { id: String },

    /// @route POST "/api/v1/view-sessions/{id}/append"
    /// @auth scope ManageRuns
    AppendViewSession { id: String },

    /// @route POST "/api/v1/view-sessions/{id}/signal"
    /// @auth scope ManageRuns
    SignalViewSession { id: String },

    /// @route DELETE "/api/v1/view-sessions/{id}"
    /// @auth scope ManageRuns
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
    /// @auth scope ManageRuns
    StopAgent { slug: String },

    /// @route POST "/api/v1/agents/{slug}/steer"
    /// @auth scope ManageRuns
    SteerAgent { slug: String },

    // ── Jobs ─────────────────────────────────────────────────────────────

    /// @route GET "/api/v1/agent_sessions"
    /// @auth scope ReadRuns
    ListJobs,

    /// @route POST "/api/v1/agent_sessions/dispatch"
    /// @auth scope CreateRuns
    DispatchPipeline,

    /// @route GET "/api/v1/agent_sessions/{pipeline_run_id}"
    /// @auth scope ReadRuns
    GetJob { pipeline_run_id: String },

    /// @route POST "/api/v1/agent_sessions/end"
    /// @auth scope ManageRuns
    EndJob,

    /// @route POST "/api/v1/agent_sessions/{slug}/cancel"
    /// @auth scope ManageRuns
    CancelJob { slug: String },

    /// @route POST "/api/v1/agent_sessions/{pipeline_run_id}/status"
    /// @auth scope EmitEvents
    RunStatusUpdate { pipeline_run_id: String },

    /// @route POST "/api/v1/agent_sessions/{slug}/step-done"
    /// @auth scope EmitEvents
    StepDoneSignal { slug: String },

    /// @route GET "/api/v1/agent_sessions/{slug}/log/stream"
    /// @auth scope ReadRuns
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

    /// @route GET "/daemon/status"
    /// @auth scope ReadWorkspace
    DaemonStatus,

    // ── Events / projections ─────────────────────────────────────────────

    /// @route GET "/api/v1/events/stream"
    /// @auth scope ReadWorkspace
    EventStream,

    /// @route POST "/api/v1/rebuild-projections"
    /// @auth scope AdminDaemon
    RebuildProjections,

    /// @route GET "/api/v1/projects/{id}/pty"
    /// @auth scope ManageRuns
    ProjectPty { id: String },

    // ── Settings / harnesses ─────────────────────────────────────────────

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
    /// @auth scope ManageRuns
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
