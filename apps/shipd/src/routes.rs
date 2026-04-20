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

    /// @route POST "/api/runtime/session"
    /// @returns SessionIssueResponse
    IssueSession,

    // ── Shared assets (unauthenticated) ──────────────────────────────────

    /// @route GET "/api/runtime/assets/datastar.js"
    AssetDatastarJs,

    /// @route GET "/api/runtime/assets/view-tokens.css"
    AssetViewTokensCss,

    /// @route GET "/api/runtime/assets/ship-elements.js"
    AssetShipElementsJs,

    /// @route GET "/api/runtime/assets/ship-code-editor.js"
    AssetCodeEditorJs,

    /// @route GET "/api/runtime/assets/ship-markdown-editor.js"
    AssetMarkdownEditorJs,

    /// @route GET "/api/runtime/assets/ship-terminal.js"
    AssetTerminalJs,

    /// @route GET "/api/runtime/assets/custom-elements.json"
    AssetElementsManifest,

    /// @route GET "/api/runtime/assets/ship-elements.md"
    AssetElementsCatalog,

    /// @route GET "/api/runtime/assets/theme.css"
    AssetThemeCss,

    /// @route GET "/api/runtime/themes"
    ListThemes,

    // ── Views: HTML shells + SSE streams ─────────────────────────────────

    /// @route GET "/api/runtime/view/pipelines"
    ViewPipelines,

    /// @route GET "/api/runtime/view/pipelines/sse"
    ViewPipelinesSse,

    /// @route GET "/api/runtime/view/specs"
    ViewSpecs,

    /// @route GET "/api/runtime/view/specs/sse"
    ViewSpecsSse,

    /// @route GET "/api/runtime/view/git"
    ViewGit,

    /// @route GET "/api/runtime/view/git/sse"
    ViewGitSse,

    /// @route GET "/api/runtime/view/settings"
    ViewSettings,

    /// @route GET "/api/runtime/view/settings/sse"
    ViewSettingsSse,

    /// @route GET "/api/runtime/view/agents"
    ViewAgents,

    /// @route GET "/api/runtime/view/agents/detail/{id}"
    ViewAgentDetail { id: String },

    /// @route POST "/api/runtime/view/agents/new"
    ViewAgentsCreate,

    /// @route GET "/api/runtime/view/skills"
    ViewSkills,

    /// @route GET "/api/runtime/view/skills/rail"
    ViewSkillsRail,

    /// @route GET "/api/runtime/view/skills/detail/{id}"
    ViewSkillDetail { id: String },

    /// @route GET "/api/runtime/view/chat"
    ViewChat,

    /// @route GET "/api/runtime/view/chat/sse"
    ViewChatSse,

    /// @route POST "/api/runtime/view/chat/send"
    ViewChatSend,

    /// @route GET "/api/runtime/view/events"
    ViewEvents,

    /// @route GET "/api/runtime/view/events/sse"
    ViewEventsSse,

    // ── Generic view serving ─────────────────────────────────────────────

    /// @route GET "/api/runtime/views"
    ListViews,

    /// @route GET "/api/runtime/view/{name}"
    ServeView { name: String },

    /// @route GET "/api/runtime/view/{name}/manifest"
    ServeViewManifest { name: String },

    /// @route GET "/api/runtime/view/{name}/bundle"
    ServeViewBundle { name: String },

    /// @route GET "/api/runtime/view/{name}/raw"
    ServeViewRaw { name: String },

    /// @route GET "/api/runtime/view/{name}/sse"
    ServeViewSse { name: String },

    // ── Agent-owned view sessions (Phase 1 canvas framework) ─────────────

    /// @route POST "/api/runtime/view-sessions"
    OpenViewSession,

    /// @route POST "/api/runtime/view-sessions/{id}/patch"
    PatchViewSession { id: String },

    /// @route POST "/api/runtime/view-sessions/{id}/append"
    AppendViewSession { id: String },

    /// @route POST "/api/runtime/view-sessions/{id}/signal"
    SignalViewSession { id: String },

    /// @route DELETE "/api/runtime/view-sessions/{id}"
    CloseViewSession { id: String },

    /// @route GET "/api/runtime/view-sessions/{id}"
    SubscribeViewSession { id: String },

    /// @route GET "/api/runtime/view-sessions/{id}/raw"
    ViewSessionRaw { id: String },

    // ── Feedback (agent-designed feedback ingress) ────────────────────────

    /// @route POST "/api/runtime/feedback"
    PostFeedback,

    // ── Annotations (canvas overlay wire format) ─────────────────────────

    /// @route POST "/api/runtime/annotations"
    PostAnnotation,

    // ── Runtime listings ─────────────────────────────────────────────────

    /// @route GET "/api/runtime/workspaces"
    ListWorkspaces,

    /// @route GET "/api/runtime/sessions"
    ListSessions,

    /// @route GET "/api/runtime/agents"
    ListAgents,

    /// @route POST "/api/runtime/agents/{slug}/stop"
    StopAgent { slug: String },

    /// @route POST "/api/runtime/agents/{slug}/steer"
    SteerAgent { slug: String },

    // ── Jobs ─────────────────────────────────────────────────────────────

    /// @route GET "/api/runtime/jobs"
    ListJobs,

    /// @route POST "/api/runtime/jobs"
    CreateJob,

    /// @route GET "/api/runtime/jobs/{job_id}"
    GetJob { job_id: String },

    /// @route POST "/api/runtime/jobs/end"
    EndJob,

    /// @route POST "/api/runtime/jobs/{slug}/cancel"
    CancelJob { slug: String },

    // ── Process projection / events polling / daemon status (slice 6) ────

    /// @route GET "/api/runtime/procs"
    ListProcs,

    /// @route GET "/api/runtime/events/poll"
    PollEvents,

    /// @route GET "/api/runtime/daemon/status"
    DaemonStatus,

    // ── Specs ────────────────────────────────────────────────────────────

    /// @route GET "/api/runtime/specs"
    ListSpecs,

    /// @route POST "/api/runtime/specs"
    CreateSpec,

    /// @route GET "/api/runtime/specs/{id}"
    GetSpec { id: String },

    /// @route PUT "/api/runtime/specs/{id}"
    UpdateSpec { id: String },

    /// @route DELETE "/api/runtime/specs/{id}"
    DeleteSpec { id: String },

    /// @route POST "/api/runtime/specs/{id}/launch"
    LaunchSpec { id: String },

    // ── Events / projections ─────────────────────────────────────────────

    /// @route GET "/api/runtime/events"
    EventStream,

    /// @route POST "/api/runtime/rebuild-projections"
    RebuildProjections,

    /// @route GET "/api/runtime/workspaces/{id}/pty"
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

    /// @route GET "/api/workspaces/{id}/git/status"
    WorkspaceGitStatus { id: String },

    /// @route GET "/api/workspaces/{id}/git/diff"
    WorkspaceGitDiff { id: String },

    /// @route GET "/api/workspaces/{id}/git/log"
    WorkspaceGitLog { id: String },

    /// @route GET "/api/workspaces/{id}/agents"
    ListWorkspaceAgents { id: String },

    /// @route POST "/api/workspaces/{id}/agents"
    WriteWorkspaceAgent { id: String },

    /// @route GET "/api/workspaces/{id}/skills"
    ListWorkspaceSkills { id: String },

    /// @route GET "/api/workspaces/{id}/skills/{skillId}/vars"
    #[allow(non_snake_case)]
    GetSkillVars { id: String, skillId: String },

    /// @route PUT "/api/workspaces/{id}/skills/{skillId}/vars/{key}"
    #[allow(non_snake_case)]
    SetSkillVar { id: String, skillId: String, key: String },

    /// @route DELETE "/api/workspaces/{id}"
    DeleteWorkspace { id: String },

    /// @route POST "/api/workspaces/{id}/activate"
    ActivateWorkspace { id: String },

    /// @route GET "/api/workspaces/{id}/pipelines"
    ListWorkspacePipelines { id: String },

    /// @route PUT "/api/workspaces/{id}/pipelines/{name}"
    WritePipeline { id: String, name: String },

    /// @route DELETE "/api/workspaces/{id}/pipelines/{name}"
    DeletePipeline { id: String, name: String },

    /// @route GET "/api/project/git/status"
    ProjectGitStatus,

    /// @route GET "/api/project/git/diff"
    ProjectGitDiff,

    /// @route GET "/api/project/git/log"
    ProjectGitLog,

    /// @route POST "/api/events/emit"
    EmitEvent,

    /// @route GET "/api/workspaces/{id}/artifacts/status"
    ArtifactsStatus { id: String },

    /// @route POST "/api/workspaces/{id}/artifacts/promote"
    PromoteArtifact { id: String },

    // `GET /api/artifacts` is owned by the artifacts API below — it lists
    // DB-backed artifact metadata. The prior FS-scanning endpoint on this
    // path has been replaced.

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

    /// @route DELETE "/api/artifacts/{id}"
    ArtifactDelete { id: String },

    // ── Capabilities ─────────────────────────────────────────────────────

    /// @route GET "/api/capabilities"
    Capabilities,
}

#[cfg(test)]
#[path = "routes_tests.rs"]
mod tests;
