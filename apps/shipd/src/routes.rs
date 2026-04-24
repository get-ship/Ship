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

    /// @route POST "/api/session"
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

    /// @route GET "/api/themes"
    ListThemes,

    // ── AgentService — service-shaped CRUD + lifecycle ──

    /// @route GET "/api/agents"
    AgentsList,

    /// @route GET "/api/agents/{id}"
    AgentsGet { id: String },

    /// @route PUT "/api/agents/{id}"
    AgentsPut { id: String },

    /// @route PATCH "/api/agents/{id}"
    AgentsPatch { id: String },

    /// @route DELETE "/api/agents/{id}"
    AgentsDelete { id: String },

    /// @route POST "/api/agents"
    AgentsCreate,

    /// @route POST "/api/agents/{id}/rename"
    AgentsRename { id: String },

    /// @route POST "/api/agents/{id}/duplicate"
    AgentsDuplicate { id: String },

    /// @route POST "/api/agents/{id}/move"
    AgentsMove { id: String },

    // ── PipelineService — service-shaped CRUD + run ops ──

    /// @route GET "/api/pipelines"
    PipelinesList,

    /// @route GET "/api/pipelines/{name}"
    PipelinesGet { name: String },

    /// @route PUT "/api/pipelines/{name}"
    PipelinesPut { name: String },

    /// @route PATCH "/api/pipelines/{name}"
    PipelinesPatch { name: String },

    /// @route DELETE "/api/pipelines/{name}"
    PipelinesDelete { name: String },

    /// @route POST "/api/pipelines/{name}/dispatch"
    PipelinesDispatch { name: String },

    /// @route POST "/api/pipelines/runs/{slug}/cancel"
    PipelineRunCancel { slug: String },

    /// @route GET "/api/pipelines/runs"
    PipelineRunsList,

    /// @route GET "/api/pipelines/runs/{slug}"
    PipelineRunGet { slug: String },

    /// @route GET "/api/pipelines/runs/{slug}/log/stream"
    PipelineRunLogStream { slug: String },

    // ── RuleService — markdown rules at <scope>/rules/<id>.md ──

    /// @route GET "/api/rules"
    RulesList,

    /// @route GET "/api/rules/{id}"
    RulesGet { id: String },

    /// @route PUT "/api/rules/{id}"
    RulesPut { id: String },

    /// @route DELETE "/api/rules/{id}"
    RulesDelete { id: String },

    /// @route POST "/api/rules"
    RulesCreate,

    // ── McpService — entries in <scope>/mcp.jsonc ──

    /// @route GET "/api/mcp"
    McpList,

    /// @route GET "/api/mcp/{id}"
    McpGet { id: String },

    /// @route PUT "/api/mcp/{id}"
    McpPut { id: String },

    /// @route PATCH "/api/mcp/{id}"
    McpPatch { id: String },

    /// @route DELETE "/api/mcp/{id}"
    McpDelete { id: String },

    /// @route POST "/api/mcp"
    McpCreate,

    // ── PermissionService — preset entries in <scope>/permissions.jsonc ──

    /// @route GET "/api/permissions"
    PermissionsList,

    /// @route GET "/api/permissions/tool-patterns"
    PermissionsToolPatterns,

    /// @route GET "/api/permissions/{id}"
    PermissionsGet { id: String },

    /// @route PUT "/api/permissions/{id}"
    PermissionsPut { id: String },

    /// @route PATCH "/api/permissions/{id}"
    PermissionsPatch { id: String },

    /// @route DELETE "/api/permissions/{id}"
    PermissionsDelete { id: String },

    /// @route POST "/api/permissions"
    PermissionsCreate,

    // ── SkillService — service-shaped CRUD + vars ──

    /// @route GET "/api/skills"
    SkillsList,

    /// @route GET "/api/skills/{id}"
    SkillsGet { id: String },

    /// @route POST "/api/skills"
    SkillsCreate,

    /// @route DELETE "/api/skills/{id}"
    SkillsDelete { id: String },

    /// @route PUT "/api/skills/{id}/frontmatter"
    SkillsPutFrontmatter { id: String },

    /// @route PUT "/api/skills/{id}/body"
    SkillsPutBody { id: String },

    /// @route POST "/api/skills/{id}/rename"
    SkillsRename { id: String },

    /// @route POST "/api/skills/{id}/duplicate"
    SkillsDuplicate { id: String },

    /// @route POST "/api/skills/{id}/move"
    SkillsMove { id: String },

    /// @route GET "/api/skills/{id}/vars"
    SkillsGetVars { id: String },

    /// @route PUT "/api/skills/{id}/vars/{key}"
    SkillsPutVar { id: String, key: String },

    /// @route DELETE "/api/skills/{id}/vars/{key}"
    SkillsDeleteVar { id: String, key: String },

    /// @route POST "/api/chat/send"
    ChatSend,

    // ── Generic view serving ─────────────────────────────────────────────

    /// @route GET "/api/views"
    ListViews,

    /// @route GET "/api/ship/schema"
    ShipSchema,

    /// @route GET "/api/v/{*path}"
    ServeViewRouted { path: String },

    /// @route GET "/api/sse/{*path}"
    ServeViewSseRouted { path: String },

    /// @route GET "/api/view-source/{*path}"
    ServeViewSource { path: String },

    /// @route POST "/api/create/{surface}"
    CreateSurface { surface: String },

    /// @route PUT "/api/views/{name}/template"
    WriteViewTemplate { name: String },

    /// @route PUT "/api/views/{name}/manifest"
    WriteViewManifestApi { name: String },

    /// @route POST "/api/views/{name}/fork"
    ForkView { name: String },

    /// @route GET "/api/view/{name}/manifest"
    ServeViewManifest { name: String },

    /// @route GET "/api/view/{name}/bundle"
    ServeViewBundle { name: String },

    /// @route GET "/api/view/{name}/raw"
    ServeViewRaw { name: String },

    /// @route GET "/api/view/{name}/sse"
    ServeViewSse { name: String },

    // ── Agent-owned view sessions (Phase 1 canvas framework) ─────────────

    /// @route POST "/api/view-sessions"
    OpenViewSession,

    /// @route POST "/api/view-sessions/{id}/patch"
    PatchViewSession { id: String },

    /// @route POST "/api/view-sessions/{id}/append"
    AppendViewSession { id: String },

    /// @route POST "/api/view-sessions/{id}/signal"
    SignalViewSession { id: String },

    /// @route DELETE "/api/view-sessions/{id}"
    CloseViewSession { id: String },

    /// @route GET "/api/view-sessions/{id}"
    SubscribeViewSession { id: String },

    /// @route GET "/api/view-sessions/{id}/raw"
    ViewSessionRaw { id: String },

    // ── Feedback (agent-designed feedback ingress) ────────────────────────

    /// @route POST "/api/feedback"
    PostFeedback,

    // ── Annotations (canvas overlay wire format) ─────────────────────────

    /// @route POST "/api/annotations"
    PostAnnotation,

    // ── Runtime listings ─────────────────────────────────────────────────

    /// @route GET "/api/workspaces"
    ListWorkspaces,

    /// @route GET "/api/sessions"
    ListSessions,

    /// @route GET "/api/agents/active"
    ListAgents,

    /// @route POST "/api/agents/{slug}/stop"
    StopAgent { slug: String },

    /// @route POST "/api/agents/{slug}/steer"
    SteerAgent { slug: String },

    // ── Jobs ─────────────────────────────────────────────────────────────

    /// @route GET "/api/jobs"
    ListJobs,

    /// @route POST "/api/jobs/dispatch"
    DispatchPipeline,

    /// @route GET "/api/jobs/{job_id}"
    GetJob { job_id: String },

    /// @route POST "/api/jobs/end"
    EndJob,

    /// @route POST "/api/jobs/{slug}/cancel"
    CancelJob { slug: String },

    /// @route GET "/api/jobs/{slug}/log/stream"
    StreamJobLog { slug: String },

    // ── Process projection / events polling / daemon status (slice 6) ────

    /// @route GET "/api/procs"
    ListProcs,

    /// @route GET "/api/events/poll"
    PollEvents,

    /// @route GET "/api/daemon/status"
    DaemonStatus,

    // ── Specs ────────────────────────────────────────────────────────────

    /// @route GET "/api/specs"
    ListSpecs,

    /// @route POST "/api/specs"
    CreateSpec,

    /// @route GET "/api/specs/{id}"
    GetSpec { id: String },

    /// @route PUT "/api/specs/{id}"
    UpdateSpec { id: String },

    /// @route DELETE "/api/specs/{id}"
    DeleteSpec { id: String },

    /// @route POST "/api/specs/{id}/launch"
    LaunchSpec { id: String },

    // ── Events / projections ─────────────────────────────────────────────

    /// @route GET "/api/events/stream"
    EventStream,

    /// @route POST "/api/rebuild-projections"
    RebuildProjections,

    /// @route GET "/api/workspaces/{id}/pty"
    WorkspacePty { id: String },

    // ── Settings / providers ─────────────────────────────────────────────

    /// @route GET "/api/settings"
    ListSettings,

    /// @route PUT "/api/settings"
    UpdateSetting,

    /// @route GET "/api/settings/{key}"
    GetSetting { key: String },

    /// @route GET "/api/providers"
    ListProviders,

    // ── Supervisor ───────────────────────────────────────────────────────

    /// @route POST "/api/supervisor/workspaces/{id}/start"
    StartWorkspace { id: String },

    // ── Project / workspace scoped ───────────────────────────────────────

    /// @route GET "/api/workspaces/{id}/session-files"
    ListSessionFiles { id: String },

    /// @route GET "/api/workspaces/{id}/agents"
    ListWorkspaceAgents { id: String },

    /// @route POST "/api/workspaces/{id}/agents"
    WriteWorkspaceAgent { id: String },

    /// @route DELETE "/api/workspaces/{id}"
    DeleteWorkspace { id: String },

    /// @route POST "/api/workspaces/{id}/activate"
    ActivateWorkspace { id: String },

    // ── GitService — canonical `/api/git/*` surface ──────────────────────

    /// @route GET "/api/git/status"
    GitStatus,

    /// @route GET "/api/git/diff"
    GitDiff,

    /// @route GET "/api/git/log"
    GitLog,

    /// @route GET "/api/git/branches"
    GitBranches,

    /// @route POST "/api/git/stage"
    GitStage,

    /// @route POST "/api/git/unstage"
    GitUnstage,

    /// @route POST "/api/git/commit"
    GitCommit,

    /// @route POST "/api/git/push"
    GitPush,

    /// @route POST "/api/git/fetch"
    GitFetch,

    /// @route POST "/api/git/pull"
    GitPull,

    /// @route POST "/api/git/checkout"
    GitCheckout,

    /// @route GET "/api/git/commit-draft"
    GitCommitDraftGet,

    /// @route PUT "/api/git/commit-draft"
    GitCommitDraftPut,

    /// @route DELETE "/api/git/commit-draft"
    GitCommitDraftDelete,

    /// @route GET "/api/git/sse"
    GitSse,

    /// @route POST "/api/events/emit"
    EmitEvent,

    // Workspace-scoped artifact status/promote removed — callers get 404.
    // Use the unified /api/artifacts/* surface instead.

    // ── Mesh / actors ────────────────────────────────────────────────────

    /// @route POST "/api/mesh/register"
    MeshRegister,

    /// @route POST "/api/mesh/send"
    MeshSend,

    /// @route POST "/api/mesh/broadcast"
    MeshBroadcast,

    /// @route GET "/api/mesh/discover"
    MeshDiscover,

    /// @route POST "/api/mesh/status"
    MeshStatusUpdate,

    /// @route GET "/api/mesh/events/{agent_id}"
    MeshEvents { agent_id: String },

    /// @route POST "/api/actor/spawn"
    ActorSpawn,

    /// @route POST "/api/events/route"
    EventRoute,

    // ── Collections ──────────────────────────────────────────────────────

    /// @route GET "/api/collections"
    ListCollections,

    /// @route GET "/api/collections/{collection}/schema"
    GetCollectionSchema { collection: String },

    /// @route PUT "/api/collections/{collection}/schema"
    UpdateCollectionSchema { collection: String },

    /// @route POST "/api/collections/{collection}/items"
    CreateCollectionItem { collection: String },

    /// @route GET "/api/collections/{collection}/items/{item_id}"
    GetCollectionItem { collection: String, item_id: String },

    /// @route PUT "/api/collections/{collection}/items/{item_id}"
    UpdateCollectionItem { collection: String, item_id: String },

    /// @route DELETE "/api/collections/{collection}/items/{item_id}"
    DeleteCollectionItem { collection: String, item_id: String },

    /// @route POST "/api/collections/{collection}/query"
    QueryCollection { collection: String },

    // ── Artifacts ────────────────────────────────────────────────────────

    /// @route POST "/api/artifacts"
    ArtifactsUpload,

    /// @route GET "/api/artifacts"
    ArtifactsList,

    /// @route GET "/api/artifacts/{id}"
    ArtifactDownload { id: String },


    /// @route GET "/api/artifacts/{id}/metadata"
    ArtifactMetadata { id: String },

    /// @route PATCH "/api/artifacts/{id}"
    ArtifactPatch { id: String },

    /// @route DELETE "/api/artifacts/{id}"
    ArtifactDelete { id: String },

    /// @route POST "/api/artifacts/{id}/promote"
    ArtifactPromote { id: String },

    // ── Capabilities ─────────────────────────────────────────────────────

    /// @route GET "/api/capabilities"
    Capabilities,

    // ── Lint ─────────────────────────────────────────────────────────────

    /// @route POST "/api/lint/view"
    LintView,

    // ── SDK manifest (read-side; generated by `just gen-sdk`) ────────────

    /// @route GET "/api/views/{name}/meta"
    ViewMeta { name: String },

    /// @route GET "/api/sdk"
    SdkRoot,

    /// @route GET "/api/sdk/query"
    SdkQuery,

    // ── View client-error capture ────────────────────────────────────────

    /// @route POST "/api/view-errors"
    PostViewError,

    /// @route GET "/api/view-errors"
    ListViewErrors,

    /// @route DELETE "/api/view-errors"
    ClearViewErrors,

    // ── MCP registry proxy ────────────────────────────────────────────────

    /// @route GET "/api/mcp/servers"
    McpServers,
}

#[cfg(test)]
#[path = "routes_tests.rs"]
mod tests;
