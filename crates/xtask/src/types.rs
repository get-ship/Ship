//! Type collection helpers for gen-types.

use specta::TypeCollection;

/// Types whose canonical home is `packages/deck/sdk/src/generated.ts`:
/// project config, permissions, skills, agent profiles, runtime workspace +
/// job/event projections. Consumers (sdk/types.ts) import from this file
/// directly; do not move members out of this set without updating consumers.
pub fn collect_types() -> TypeCollection {
    let mut c = TypeCollection::default();

    // types/config.rs
    c.register::<compiler::HookTrigger>();
    c.register::<compiler::HookConfig>();
    c.register::<compiler::PermissionConfig>();
    c.register::<compiler::AgentConfig>();
    c.register::<compiler::McpServerType>();
    c.register::<compiler::McpServerConfig>();
    c.register::<compiler::AgentLayerConfig>();
    c.register::<compiler::NamespaceConfig>();
    c.register::<compiler::ProjectConfig>();

    // types/permissions.rs
    c.register::<compiler::ToolPermissions>();
    c.register::<compiler::Permissions>();
    c.register::<compiler::NetworkPolicy>();
    c.register::<compiler::NetworkPermissions>();
    c.register::<compiler::FsPermissions>();
    c.register::<compiler::CommandPermissions>();
    c.register::<compiler::AgentLimits>();

    // types/skill.rs
    c.register::<compiler::SkillSource>();
    c.register::<compiler::Skill>();

    // types/rule.rs
    c.register::<compiler::Rule>();

    // types/plugin.rs
    c.register::<compiler::PluginEntry>();
    c.register::<compiler::PluginsManifest>();

    // types/catalog.rs
    c.register::<compiler::CatalogKind>();
    c.register::<compiler::CatalogCategory>();
    c.register::<compiler::CatalogEntry>();

    // types/agent_profile.rs
    c.register::<compiler::AgentProfile>();
    c.register::<compiler::ProfileMeta>();
    c.register::<compiler::SkillRefs>();
    c.register::<compiler::McpRefs>();
    c.register::<compiler::PluginRefs>();
    c.register::<compiler::ProfilePermissions>();
    c.register::<compiler::ProfileRules>();

    // resolve.rs
    c.register::<compiler::ProjectLibrary>();

    // compile/mod.rs
    c.register::<compiler::CompileOutput>();

    // types/transfer.rs — MCP bridge types
    c.register::<compiler::TransferBundle>();
    c.register::<compiler::AgentBundle>();
    c.register::<compiler::SkillBundle>();
    c.register::<compiler::PullResponse>();
    c.register::<compiler::PullAgent>();
    c.register::<compiler::PullProfile>();
    c.register::<compiler::PullSkill>();
    c.register::<compiler::PullMcpServer>();
    c.register::<compiler::PullRule>();
    c.register::<compiler::ListAgentsResponse>();

    // ── Daemon / runtime types ─────────────────────────────────────────────

    // projections/job.rs
    c.register::<ship_pipeline_runtime::projections::pipeline_run::PipelineRun>();
    c.register::<ship_pipeline_runtime::projections::pipeline_run::PipelineRunStatus>();

    // events/job.rs (job event payloads)
    c.register::<ship_substrate::events::pipeline::PipelineRunStartedPayload>();
    c.register::<ship_substrate::events::pipeline::PipelineRunCompletedPayload>();
    c.register::<ship_substrate::events::pipeline::PipelineRunDispatchedPayload>();
    c.register::<ship_substrate::events::pipeline::PipelineGateRequestedPayload>();
    c.register::<ship_substrate::events::pipeline::PipelineGatePassedPayload>();
    c.register::<ship_substrate::events::pipeline::PipelineGateFailedPayload>();
    c.register::<ship_substrate::events::pipeline::PipelineRunBlockedPayload>();
    c.register::<ship_substrate::events::pipeline::PipelineRunMergedPayload>();
    c.register::<ship_substrate::events::pipeline::PipelineRunFailedPayload>();
    c.register::<ship_substrate::events::pipeline::PipelineRunUpdatedPayload>();

    // events/job.rs (step event payloads + evidence types)
    c.register::<ship_substrate::events::pipeline::PipelineStepObservedPayload>();
    c.register::<ship_substrate::events::pipeline::StepTrigger>();
    c.register::<ship_substrate::events::pipeline::StepVerdict>();
    c.register::<ship_substrate::events::pipeline::PipelineStepRetryPayload>();

    // events/job.rs (pipeline types — already exported via PipelineRun, but explicit for clarity)
    c.register::<ship_substrate::events::pipeline::Pipeline>();
    c.register::<ship_substrate::events::pipeline::PipelineStep>();
    c.register::<ship_substrate::events::pipeline::StepType>();

    // events/feedback.rs
    c.register::<ship_substrate::events::feedback::FeedbackEvent>();
    c.register::<ship_substrate::events::feedback::FeedbackKind>();
    c.register::<ship_substrate::events::feedback::FeedbackIntent>();
    c.register::<ship_substrate::events::feedback::FeedbackTarget>();
    c.register::<ship_substrate::events::feedback::Selection>();
    c.register::<ship_substrate::events::feedback::AuthorKind>();

    // events/annotation.rs (AuthorKind/Selection shared with feedback.rs — registered above)
    c.register::<ship_substrate::events::annotation::AnnotationEvent>();
    c.register::<ship_substrate::events::annotation::AnnotationKind>();
    c.register::<ship_substrate::events::annotation::AnnotationIntent>();
    c.register::<ship_substrate::events::annotation::AnnotationTarget>();

    // events/filter.rs
    c.register::<ship_substrate::events::filter::EventFilter>();

    // collection field-definition types are NOT specta-exported in the new
    // ship-collection crate (post-#81 specta removal from domain types); the
    // wire/proto types are the registered surface now (C-2 §3.8).

    // workspace/types_session.rs (the workspace primitive retired; only
    // session-shaped types remain)
    c.register::<ship_agents::sessions::Session>();
    c.register::<ship_agents::sessions::SessionStatus>();

    c
}

/// Framework data types that cross the daemon ↔ ui-runtime boundary.
/// Output lands at `packages/sui/runtime/src/generated.ts` and is
/// re-exported from `@ship/sui-runtime`. Add new entries here whenever a
/// daemon HTTP/SSE response or a runtime payload struct gains a
/// `#[derive(Type)]`. Drift between Rust and these registrations is
/// caught by `cargo xtask gen-types --check` in CI.
pub fn collect_framework_types() -> TypeCollection {
    let mut c = TypeCollection::default();

    // deck — view payload + resolver.
    c.register::<deck::view_payload::ViewPayload>();
    c.register::<deck::resolver::ResolvedView>();
    c.register::<deck::resolver::ResolveError>();

    // shipd::views — view list / manifest / source / write APIs.
    c.register::<shipd::views::ViewManifest>();
    c.register::<shipd::views::ViewListEntry>();
    c.register::<shipd::views::RailMeta>();
    c.register::<shipd::views::write_api::WriteTemplateRequest>();
    c.register::<shipd::views::write_api::WriteManifestRequest>();
    c.register::<shipd::views::write_api::WriteResponse>();
    c.register::<shipd::views::write_api::WriteDiagnostics>();
    c.register::<shipd::views::write_api::LintFinding>();
    c.register::<shipd::views::resolve::ManifestRail>();

    // view-authoring — contract types shared with the chrome edit-mode UI.
    c.register::<ship_views::view_authoring::CompileError>();
    c.register::<ship_views::view_authoring::SnapshotMeta>();
    c.register::<ship_views::view_authoring::ViewReadSourceRequest>();
    c.register::<ship_views::view_authoring::ViewReadSourceResponse>();
    c.register::<ship_views::view_authoring::ViewWriteSourceRequest>();
    c.register::<ship_views::view_authoring::ViewWriteSourceResponse>();
    c.register::<ship_views::view_authoring::ViewCompileCheckRequest>();
    c.register::<ship_views::view_authoring::ViewCompileCheckResponse>();
    c.register::<ship_views::view_authoring::ViewCollectionsSchemaRequest>();
    c.register::<ship_views::view_authoring::ViewListSiblingsRequest>();
    c.register::<ship_views::view_authoring::SiblingView>();
    c.register::<ship_views::view_authoring::ViewListSiblingsResponse>();
    c.register::<ship_views::view_authoring::ViewThemeTokensRequest>();
    c.register::<ship_views::view_authoring::ViewThemeTokensResponse>();
    c.register::<ship_views::view_authoring::ViewPreviewSnapshotRequest>();
    c.register::<ship_views::view_authoring::ViewPreviewSnapshotResponse>();
    c.register::<ship_views::view_authoring::AnnotationKind>();
    c.register::<ship_views::view_authoring::ViewAnnotateRequest>();
    c.register::<ship_views::view_authoring::ViewAnnotateResponse>();
    c.register::<ship_views::view_authoring::ViewListSnapshotsRequest>();
    c.register::<ship_views::view_authoring::ViewListSnapshotsResponse>();
    c.register::<ship_views::view_authoring::ViewRestoreSnapshotRequest>();
    c.register::<ship_views::view_authoring::ViewRestoreSnapshotResponse>();
    c.register::<ship_views::view_authoring::ViewEditModeEnterRequest>();
    c.register::<ship_views::view_authoring::ViewEditModeExitRequest>();
    c.register::<ship_views::view_authoring::ViewEditModeAck>();

    c
}

/// Ship daemon-domain data types (git / agents / account). These belong to the
/// SDK layer, NOT the framework — `@ship/sui-runtime` names no host concept.
/// Output: `packages/deck/sdk-client/src/generated.ts`.
pub fn collect_ship_data_types() -> TypeCollection {
    let mut c = TypeCollection::default();

    // git — overview, diff, status buckets.
    c.register::<ship_platform::services::git::state::CommitSummary>();
    c.register::<ship_platform::services::git::state::StatusFile>();
    c.register::<ship_platform::services::git::state::StatusBuckets>();
    c.register::<ship_platform::services::git::state::BranchSummary>();
    c.register::<ship_platform::services::git::state::GitOverview>();
    c.register::<ship_platform::services::git::state::GitDiff>();
    c.register::<ship_platform::services::git::state::GitDiffArgs>();
    c.register::<ship_platform::services::git::diff::DiffLine>();
    c.register::<ship_platform::services::git::diff::DiffHunk>();
    c.register::<ship_platform::services::git::diff::DiffFileView>();
    c.register::<ship_platform::services::git::diff::ParsedDiff>();

    // agents — summary list payload.
    c.register::<ship_agents::services::agents::view_state::AgentSummary>();
    c.register::<ship_agents::services::agents::view_state::AgentsSummariesResponse>();

    // account — Ship Cloud account state for the chrome chip.
    c.register::<ship_account::types::AccountState>();
    c.register::<ship_account::types::MeView>();

    // artifacts — the new `ship_artifacts::types::ArtifactRecord` is not
    // specta-exported (post-#81); the proto wire type is the registered surface
    // (C-2 §3.8). The daemon `ArtifactsListResponse` carries the proto shape.

    // skills — index row. `#[specta(rename = "SkillIndexEntry")]` on the Rust
    // struct disambiguates it from the other domains' IndexEntry.
    c.register::<ship_agents::services::skills::state::IndexEntry>();

    // collections — typed row/Database codegen moved to `gen_collection_types`
    // (schema→TS via the ship_collection SSOT). The old specta registration of
    // shipd::services::registry::wire::CollectionSummary is gone with the
    // services::registry cull; nothing specta-derived replaces it here.

    c
}
