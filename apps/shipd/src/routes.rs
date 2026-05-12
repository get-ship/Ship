//! Canonical Ship route registry.
//!
//! Parsed at build time by `apps/shipd/build.rs` to emit three artifacts:
//!
//! - `$OUT_DIR/routes_generated.rs` — Rust URL builders (`Route::method()`,
//!   `Route::full_path()`) and the `ShipRouterExt::ship_route(Route, handler)`
//!   trait extension used by `lib.rs` to register routes.
//! - `packages/sdk/src/generated/routes.ts` — typed TypeScript route
//!   catalog for SDK / web consumers. Relative URLs (cookie-authenticated).
//! - `apps/shipd/templates/_urls.html.j2` — minijinja macros (one per route
//!   variant). Templates import the ones they need:
//!   `{% from '_urls.html.j2' import view_pipelines, list_jobs %}`.
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
//! /// @returns Vec<JobRecord>     (optional — TS gets typed fetch wrapper)
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

pub mod actor_api;
pub mod actors_api;
pub mod annotations_api;
pub mod artifacts_api;
pub mod collection_api;
pub mod create_api;
pub mod feedback_api;
pub mod hooks_api;
pub mod lint_api;
pub mod logs_api;
pub mod human_webhook;
pub mod openapi;
pub mod openapi_modifier;
pub mod openapi_types;
pub mod projects_api;
pub mod rest_api;
pub mod runtime_api;
pub mod sdk_manifest;
pub mod settings_api;
pub mod shell_assets;
pub mod specs_api;
pub mod themes;
pub mod themes_api;
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
    Index,

    /// @route GET "/setup"
    Setup,

    // ── Session / auth ───────────────────────────────────────────────────

    /// @route POST "/api/v1/session"
    /// @returns SessionIssueResponse
    IssueSession,

    // ── Shared assets (unauthenticated) ──────────────────────────────────

    /// @route GET "/api/assets/datastar.js"
    AssetDatastarJs,

    /// @route GET "/api/assets/view-tokens.css"
    AssetViewTokensCss,

    /// @route GET "/api/assets/ship-elements.js"
    AssetShipElementsJs,

    /// @route GET "/api/assets/ship-code-editor.js"
    AssetCodeEditorJs,

    /// @route GET "/api/assets/ship-markdown-editor.js"
    AssetMarkdownEditorJs,

    /// @route GET "/api/assets/ship-terminal.js"
    AssetTerminalJs,

    /// @route GET "/api/assets/ship-git.js"
    AssetGitJs,

    /// @route GET "/api/assets/custom-elements.json"
    AssetElementsManifest,

    /// @route GET "/api/assets/ship-elements.md"
    AssetElementsCatalog,

    /// @route GET "/api/assets/jinja-macros.json"
    AssetJinjaMacrosJson,

    /// @route GET "/api/assets/jinja-macros.md"
    AssetJinjaMacrosMd,

    /// @route GET "/api/assets/theme.css"
    AssetThemeCss,

    /// @route GET "/api/v1/themes"
    ListThemes,

    // ── SUI framework hosting (ship-sui-server handlers) ──

    /// @route GET "/api/assets/sui-custom-elements.json"
    AssetSuiCustomElementsManifest,

    /// @route GET "/_sui/runtime.js"
    SuiRuntimeJs,

    /// @route GET "/_sui/dom.js"
    SuiDomJs,

    /// @route GET "/_sui/sdk-client.js"
    SuiSdkClientJs,

    /// @route GET "/_sui/components.js"
    SuiComponentsJs,

    /// @route GET "/_sui/lucide-icons.json"
    SuiLucideIconsJson,

    /// @route GET "/_sui/islands/{tag}/island.js"
    SuiIsland { tag: String },

    /// @route GET "/_sui/views/{name}/view.sui"
    SuiViewTemplate { name: String },

    /// @route GET "/_sui/views/{name}/view.js"
    SuiViewSetup { name: String },

    /// @route GET "/_sui/views/{name}"
    SuiViewPayload { name: String },

    /// @route GET "/_sui/views/{name}/view.css"
    SuiViewCss { name: String },

    /// @route GET "/_sui/views/{name}/sse"
    SuiViewSse { name: String },

    /// @route GET "/view-host"
    SuiViewHost,

    /// @route GET "/sui/{name}"
    SuiViewHostForName { name: String },

    /// @route GET "/_sui/hmr"
    SuiHmr,

    /// @route GET "/_sui/hmr-client.js"
    SuiHmrClientJs,

    /// @route GET "/_sui/dev/hmr"
    SuiDevHmr,

    /// @route GET "/_sui/data"
    SuiData,

    /// @route POST "/_sui/mutation/{name}"
    SuiMutation { name: String },

    /// @route POST "/_sui/views/{name}/patch-elements"
    SuiPatchElements { name: String },

    // ── AgentService — service-shaped CRUD + lifecycle ──

    /// @route GET "/api/v1/agents"
    AgentsList,

    /// @route GET "/api/v1/agents/{id}"
    AgentsGet { id: String },

    /// @route PUT "/api/v1/agents/{id}"
    AgentsPut { id: String },

    /// @route PATCH "/api/v1/agents/{id}"
    AgentsPatch { id: String },

    /// @route DELETE "/api/v1/agents/{id}"
    AgentsDelete { id: String },

    /// @route POST "/api/v1/agents"
    AgentsCreate,

    /// @route POST "/api/v1/agents/{id}/rename"
    AgentsRename { id: String },

    /// @route POST "/api/v1/agents/{id}/duplicate"
    AgentsDuplicate { id: String },

    /// @route POST "/api/v1/agents/{id}/move"
    AgentsMove { id: String },

    // ── PipelineService — service-shaped CRUD + run ops ──

    /// @route GET "/api/v1/pipelines"
    PipelinesList,

    /// @route GET "/api/v1/pipelines/{name}"
    PipelinesGet { name: String },

    /// @route PUT "/api/v1/pipelines/{name}"
    PipelinesPut { name: String },

    /// @route PATCH "/api/v1/pipelines/{name}"
    PipelinesPatch { name: String },

    /// @route DELETE "/api/v1/pipelines/{name}"
    PipelinesDelete { name: String },

    /// @route POST "/api/v1/pipelines/{name}/dispatch"
    PipelinesDispatch { name: String },

    /// @route POST "/api/v1/pipelines/runs/{slug}/cancel"
    PipelineRunCancel { slug: String },

    /// @route GET "/api/v1/pipelines/runs"
    PipelineRunsList,

    /// @route GET "/api/v1/pipelines/runs/{slug}"
    PipelineRunGet { slug: String },

    /// @route GET "/api/v1/pipelines/runs/{slug}/log/stream"
    PipelineRunLogStream { slug: String },

    // ── RuleService — markdown rules at <scope>/rules/<id>.md ──

    /// @route GET "/api/v1/rules"
    RulesList,

    /// @route GET "/api/v1/rules/{id}"
    RulesGet { id: String },

    /// @route PUT "/api/v1/rules/{id}"
    RulesPut { id: String },

    /// @route DELETE "/api/v1/rules/{id}"
    RulesDelete { id: String },

    /// @route POST "/api/v1/rules"
    RulesCreate,

    // ── McpService — entries in <scope>/mcp.jsonc ──

    /// @route GET "/api/v1/mcp"
    McpList,

    /// @route GET "/api/v1/mcp/{id}"
    McpGet { id: String },

    /// @route PUT "/api/v1/mcp/{id}"
    McpPut { id: String },

    /// @route PATCH "/api/v1/mcp/{id}"
    McpPatch { id: String },

    /// @route DELETE "/api/v1/mcp/{id}"
    McpDelete { id: String },

    /// @route POST "/api/v1/mcp"
    McpCreate,

    // ── PermissionService — preset entries in <scope>/permissions.jsonc ──

    /// @route GET "/api/v1/permissions"
    PermissionsList,

    /// @route GET "/api/v1/permissions/tool-patterns"
    PermissionsToolPatterns,

    /// @route GET "/api/v1/permissions/{id}"
    PermissionsGet { id: String },

    /// @route PUT "/api/v1/permissions/{id}"
    PermissionsPut { id: String },

    /// @route PATCH "/api/v1/permissions/{id}"
    PermissionsPatch { id: String },

    /// @route DELETE "/api/v1/permissions/{id}"
    PermissionsDelete { id: String },

    /// @route POST "/api/v1/permissions"
    PermissionsCreate,

    // ── SkillService — service-shaped CRUD + vars ──

    /// @route GET "/api/v1/skills"
    SkillsList,

    /// @route GET "/api/v1/skills/{id}"
    SkillsGet { id: String },

    /// @route POST "/api/v1/skills"
    SkillsCreate,

    /// @route DELETE "/api/v1/skills/{id}"
    SkillsDelete { id: String },

    /// @route PUT "/api/v1/skills/{id}/frontmatter"
    SkillsPutFrontmatter { id: String },

    /// @route PUT "/api/v1/skills/{id}/body"
    SkillsPutBody { id: String },

    /// @route POST "/api/v1/skills/{id}/rename"
    SkillsRename { id: String },

    /// @route POST "/api/v1/skills/{id}/duplicate"
    SkillsDuplicate { id: String },

    /// @route POST "/api/v1/skills/{id}/move"
    SkillsMove { id: String },

    /// @route GET "/api/v1/skills/{id}/vars"
    SkillsGetVars { id: String },

    /// @route PUT "/api/v1/skills/{id}/vars/{key}"
    SkillsPutVar { id: String, key: String },

    /// @route DELETE "/api/v1/skills/{id}/vars/{key}"
    SkillsDeleteVar { id: String, key: String },

    /// @route POST "/api/v1/chat/send"
    ChatSend,

    // ── Generic view serving ─────────────────────────────────────────────

    /// @route GET "/api/v1/views"
    ListViews,

    /// @route GET "/api/v1/ship/schema"
    ShipSchema,

    /// @route GET "/api/v1/v/{*path}"
    ServeViewRouted { path: String },

    /// @route GET "/api/v1/sse/{*path}"
    ServeViewSseRouted { path: String },

    /// @route GET "/api/v1/view-source/{*path}"
    ServeViewSource { path: String },

    /// @route POST "/api/v1/create/{surface}"
    CreateSurface { surface: String },

    /// @route PUT "/api/v1/views/{name}/template"
    WriteViewTemplate { name: String },

    /// @route PUT "/api/v1/views/{name}/manifest"
    WriteViewManifestApi { name: String },

    /// @route POST "/api/v1/views/{name}/fork"
    ForkView { name: String },

    /// @route POST "/api/v1/themes/contrast"
    ThemeContrast,

    /// @route POST "/api/v1/sessions/spawn"
    SpawnPtySession,

    /// @route GET "/api/v1/pty-sessions"
    ListPtySessions,

    /// @route GET "/api/v1/sessions/{id}"
    GetPtySession { id: String },

    /// @route DELETE "/api/v1/sessions/{id}"
    StopPtySession { id: String },

    /// @route POST "/api/v1/sessions/{id}/input"
    PtySessionInput { id: String },

    /// @route POST "/api/v1/sessions/{id}/resize"
    PtySessionResize { id: String },

    /// @route POST "/api/v1/sessions/{id}/message"
    PtySessionMessage { id: String },

    /// @route POST "/api/v1/sessions/{id}/reply"
    PtySessionReply { id: String },

    /// @route GET "/api/v1/sessions/{id}/stream"
    StreamPtySession { id: String },

    /// @route GET "/api/v1/sessions/{id}/events"
    StreamPtySessionEvents { id: String },

    /// @route GET "/api/v1/providers/installed"
    InstalledProviders,

    /// @route GET "/api/v1/projects"
    ListProjects,

    /// @route POST "/api/v1/projects"
    AddProject,

    /// @route POST "/api/v1/projects/switch"
    SwitchProject,

    /// @route GET "/api/v1/view/{name}/manifest"
    ServeViewManifest { name: String },

    /// @route GET "/api/v1/view/{name}/bundle"
    ServeViewBundle { name: String },

    /// @route GET "/api/v1/view/{name}/raw"
    ServeViewRaw { name: String },

    /// @route GET "/api/v1/view/{name}/sse"
    ServeViewSse { name: String },

    // ── Agent-owned view sessions (Phase 1 canvas framework) ─────────────

    /// @route POST "/api/v1/view-sessions"
    OpenViewSession,

    /// @route POST "/api/v1/view-sessions/{id}/patch"
    PatchViewSession { id: String },

    /// @route POST "/api/v1/view-sessions/{id}/append"
    AppendViewSession { id: String },

    /// @route POST "/api/v1/view-sessions/{id}/signal"
    SignalViewSession { id: String },

    /// @route DELETE "/api/v1/view-sessions/{id}"
    CloseViewSession { id: String },

    /// @route GET "/api/v1/view-sessions/{id}"
    SubscribeViewSession { id: String },

    /// @route GET "/api/v1/view-sessions/{id}/raw"
    ViewSessionRaw { id: String },

    // ── Feedback (agent-designed feedback ingress) ────────────────────────

    /// @route POST "/api/v1/feedback"
    PostFeedback,

    // ── Annotations (canvas overlay wire format) ─────────────────────────

    /// @route POST "/api/v1/annotations"
    PostAnnotation,

    // ── Runtime listings ─────────────────────────────────────────────────

    /// @route GET "/api/v1/sessions"
    ListSessions,

    /// @route GET "/api/v1/agents/active"
    ListAgents,

    /// @route POST "/api/v1/agents/{slug}/stop"
    StopAgent { slug: String },

    /// @route POST "/api/v1/agents/{slug}/steer"
    SteerAgent { slug: String },

    // ── Jobs ─────────────────────────────────────────────────────────────

    /// @route GET "/api/v1/jobs"
    ListJobs,

    /// @route POST "/api/v1/jobs/dispatch"
    DispatchPipeline,

    /// @route GET "/api/v1/jobs/{job_id}"
    GetJob { job_id: String },

    /// @route POST "/api/v1/jobs/end"
    EndJob,

    /// @route POST "/api/v1/jobs/{slug}/cancel"
    CancelJob { slug: String },

    /// @route POST "/api/v1/jobs/{job_id}/status"
    JobStatusUpdate { job_id: String },

    /// @route POST "/api/v1/jobs/{slug}/step-done"
    JobStepDone { slug: String },

    /// @route GET "/api/v1/jobs/{slug}/log/stream"
    StreamJobLog { slug: String },

    // ── Process projection / events polling / daemon status (slice 6) ────

    /// @route GET "/api/v1/procs"
    ListProcs,

    /// @route GET "/api/v1/events/poll"
    PollEvents,

    /// @route GET "/api/v1/daemon/status"
    DaemonStatus,

    // ── Specs ────────────────────────────────────────────────────────────

    /// @route GET "/api/v1/specs"
    ListSpecs,

    /// @route POST "/api/v1/specs"
    CreateSpec,

    /// @route GET "/api/v1/specs/{id}"
    GetSpec { id: String },

    /// @route PUT "/api/v1/specs/{id}"
    UpdateSpec { id: String },

    /// @route DELETE "/api/v1/specs/{id}"
    DeleteSpec { id: String },

    /// @route POST "/api/v1/specs/{id}/launch"
    LaunchSpec { id: String },

    // ── Events / projections ─────────────────────────────────────────────

    /// @route GET "/api/v1/events/stream"
    EventStream,

    /// @route POST "/api/v1/rebuild-projections"
    RebuildProjections,

    /// @route GET "/api/v1/projects/{id}/pty"
    ProjectPty { id: String },

    // ── Settings / providers ─────────────────────────────────────────────

    /// @route GET "/api/v1/settings"
    ListSettings,

    /// @route PUT "/api/v1/settings"
    UpdateSetting,

    /// @route GET "/api/v1/settings/{key}"
    GetSetting { key: String },

    /// @route GET "/api/v1/providers"
    ListProviders,

    // ── Supervisor ───────────────────────────────────────────────────────

    // ── Project-scoped ───────────────────────────────────────────────────

    /// @route GET "/api/v1/projects/{id}/session-files"
    ListSessionFiles { id: String },

    /// @route GET "/api/v1/projects/{id}/agents"
    ListProjectAgents { id: String },

    /// @route POST "/api/v1/projects/{id}/agents"
    WriteProjectAgent { id: String },

    // ── GitService — canonical `/api/git/*` surface ──────────────────────

    /// @route GET "/api/v1/git/status"
    GitStatus,

    /// @route GET "/api/v1/git/diff"
    GitDiff,

    /// @route GET "/api/v1/git/log"
    GitLog,

    /// @route GET "/api/v1/git/branches"
    GitBranches,

    /// @route POST "/api/v1/git/stage"
    GitStage,

    /// @route POST "/api/v1/git/unstage"
    GitUnstage,

    /// @route POST "/api/v1/git/commit"
    GitCommit,

    /// @route POST "/api/v1/git/push"
    GitPush,

    /// @route POST "/api/v1/git/fetch"
    GitFetch,

    /// @route POST "/api/v1/git/pull"
    GitPull,

    /// @route POST "/api/v1/git/checkout"
    GitCheckout,

    /// @route GET "/api/v1/git/sse"
    GitSse,

    /// @route POST "/api/v1/events/emit"
    EmitEvent,

    // Workspace-scoped artifact status/promote removed — callers get 404.
    // Use the unified /api/artifacts/* surface instead.

    // ── Mesh / actors ────────────────────────────────────────────────────

    /// @route POST "/api/v1/mesh/register"
    MeshRegister,

    /// @route POST "/api/v1/mesh/send"
    MeshSend,

    /// @route POST "/api/v1/mesh/broadcast"
    MeshBroadcast,

    /// @route GET "/api/v1/mesh/discover"
    MeshDiscover,

    /// @route POST "/api/v1/mesh/status"
    MeshStatusUpdate,

    /// @route GET "/api/v1/mesh/events/{agent_id}"
    MeshEvents { agent_id: String },

    /// @route POST "/api/v1/actor/spawn"
    ActorSpawn,

    /// @route POST "/api/v1/events/route"
    EventRoute,

    /// @route GET "/api/v1/actors"
    ListActors,

    /// @route GET "/api/v1/mesh"
    ListMesh,

    /// @route GET "/api/v1/actors/{actor_id}"
    GetActor { actor_id: String },

    /// @route GET "/api/v1/actors/{actor_id}/events"
    ActorEvents { actor_id: String },

    /// @route POST "/api/v1/actors/{actor_id}/input"
    ActorInput { actor_id: String },

    /// @route GET "/api/v1/mesh/snapshot"
    MeshSnapshot,

    /// @route GET "/api/v1/mesh/events"
    MeshEventsStream,

    /// @route POST "/api/v1/actors/{actor_id}/kill-tree"
    KillTree { actor_id: String },

    // ── Collections ──────────────────────────────────────────────────────

    /// @route GET "/api/v1/collections"
    ListCollections,

    /// @route GET "/api/v1/collections/{collection}/schema"
    GetCollectionSchema { collection: String },

    /// @route PUT "/api/v1/collections/{collection}/schema"
    UpdateCollectionSchema { collection: String },

    /// @route POST "/api/v1/collections/{collection}/items"
    CreateCollectionItem { collection: String },

    /// @route GET "/api/v1/collections/{collection}/items/{item_id}"
    GetCollectionItem { collection: String, item_id: String },

    /// @route PUT "/api/v1/collections/{collection}/items/{item_id}"
    UpdateCollectionItem { collection: String, item_id: String },

    /// @route DELETE "/api/v1/collections/{collection}/items/{item_id}"
    DeleteCollectionItem { collection: String, item_id: String },

    /// @route POST "/api/v1/collections/{collection}/query"
    QueryCollection { collection: String },

    // ── Artifacts ────────────────────────────────────────────────────────

    /// @route POST "/api/v1/artifacts"
    ArtifactsUpload,

    /// @route GET "/api/v1/artifacts"
    ArtifactsList,

    /// @route GET "/api/v1/artifacts/{id}"
    ArtifactDownload { id: String },


    /// @route GET "/api/v1/artifacts/{id}/metadata"
    ArtifactMetadata { id: String },

    /// @route PATCH "/api/v1/artifacts/{id}"
    ArtifactPatch { id: String },

    /// @route DELETE "/api/v1/artifacts/{id}"
    ArtifactDelete { id: String },

    /// @route POST "/api/v1/artifacts/{id}/promote"
    ArtifactPromote { id: String },

    // ── Capabilities ─────────────────────────────────────────────────────

    /// @route GET "/api/v1/capabilities"
    Capabilities,

    // ── Lint ─────────────────────────────────────────────────────────────

    /// @route POST "/api/v1/lint/view"
    LintView,

    // ── SDK manifest (read-side; generated by `just gen-sdk`) ────────────

    /// @route GET "/api/v1/views/{name}/meta"
    ViewMeta { name: String },

    /// @route GET "/api/v1/sdk"
    SdkRoot,

    /// @route GET "/api/v1/sdk/query"
    SdkQuery,

    // ── View client-error capture ────────────────────────────────────────

    /// @route POST "/api/v1/view-errors"
    PostViewError,

    /// @route GET "/api/v1/view-errors"
    ListViewErrors,

    /// @route DELETE "/api/v1/view-errors"
    ClearViewErrors,

    // ── MCP registry proxy ────────────────────────────────────────────────

    /// @route GET "/api/v1/mcp/servers"
    McpServers,
}

#[cfg(test)]
mod tests;
