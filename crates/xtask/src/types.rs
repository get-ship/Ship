//! Type collection helpers for gen-types.

use specta::TypeCollection;

/// Types whose canonical home is `packages/sdk/src/generated.ts`:
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

    // definition.rs — the unified PipelineDefinition authoring contract (P0.2).
    c.register::<ship_substrate::events::pipeline::PipelineDefinition>();
    c.register::<ship_substrate::events::pipeline::PipelineStepDef>();
    c.register::<ship_substrate::events::pipeline::ProviderOverride>();
    c.register::<ship_substrate::events::pipeline::Invocation>();

    // ship-agents/src/feedback.rs (moved from ship-substrate, keystone-A de-bucket 2026-06-10)
    c.register::<ship_agents::feedback::FeedbackEvent>();
    c.register::<ship_agents::feedback::FeedbackKind>();
    c.register::<ship_agents::feedback::FeedbackIntent>();
    c.register::<ship_agents::feedback::FeedbackTarget>();
    c.register::<ship_agents::feedback::Selection>();
    c.register::<ship_agents::feedback::AuthorKind>();

    // ship-agents/src/annotation.rs (AuthorKind/Selection shared with feedback.rs — registered above)
    c.register::<ship_agents::annotation::AnnotationEvent>();
    c.register::<ship_agents::annotation::AnnotationKind>();
    c.register::<ship_agents::annotation::AnnotationIntent>();
    c.register::<ship_agents::annotation::AnnotationTarget>();

    // events/filter.rs
    c.register::<ship_substrate::events::filter::EventFilter>();

    // collection field-definition types are NOT specta-exported in the new
    // ship-collection crate (post-#81 specta removal from domain types); the
    // wire/proto types are the registered surface now (C-2 §3.8).

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

    // shipd::views — manifest rail metadata read by search_context. The
    // ViewManifest/ViewListEntry/RailMeta REST shapes were deleted with
    // their zero-caller handlers (shipd deletion map 1.12).
    c.register::<shipd::views::resolve::ManifestRail>();

    // view-authoring — contract types shared with the chrome edit-mode UI.
    c.register::<ship_ui::view_authoring::CompileError>();
    c.register::<ship_ui::view_authoring::SnapshotMeta>();
    c.register::<ship_ui::view_authoring::ViewReadSourceRequest>();
    c.register::<ship_ui::view_authoring::ViewReadSourceResponse>();
    c.register::<ship_ui::view_authoring::ViewWriteSourceRequest>();
    c.register::<ship_ui::view_authoring::ViewWriteSourceResponse>();
    c.register::<ship_ui::view_authoring::ViewCompileCheckRequest>();
    c.register::<ship_ui::view_authoring::ViewCompileCheckResponse>();
    c.register::<ship_ui::view_authoring::ViewCollectionsSchemaRequest>();
    c.register::<ship_ui::view_authoring::ViewListSiblingsRequest>();
    c.register::<ship_ui::view_authoring::SiblingView>();
    c.register::<ship_ui::view_authoring::ViewListSiblingsResponse>();
    c.register::<ship_ui::view_authoring::ViewThemeTokensRequest>();
    c.register::<ship_ui::view_authoring::ViewThemeTokensResponse>();
    c.register::<ship_ui::view_authoring::ViewPreviewSnapshotRequest>();
    c.register::<ship_ui::view_authoring::ViewPreviewSnapshotResponse>();
    c.register::<ship_ui::view_authoring::AnnotationKind>();
    c.register::<ship_ui::view_authoring::ViewAnnotateRequest>();
    c.register::<ship_ui::view_authoring::ViewAnnotateResponse>();
    c.register::<ship_ui::view_authoring::ViewListSnapshotsRequest>();
    c.register::<ship_ui::view_authoring::ViewListSnapshotsResponse>();
    c.register::<ship_ui::view_authoring::ViewRestoreSnapshotRequest>();
    c.register::<ship_ui::view_authoring::ViewRestoreSnapshotResponse>();
    c.register::<ship_ui::view_authoring::ViewEditModeEnterRequest>();
    c.register::<ship_ui::view_authoring::ViewEditModeExitRequest>();
    c.register::<ship_ui::view_authoring::ViewEditModeAck>();

    c
}

/// Ship daemon-domain data types (agents / account / skills). These belong to the
/// SDK layer, NOT the framework — `@ship/sui-runtime` names no host concept.
/// Output: `packages/sdk-client/src/generated.ts`.
pub fn collect_ship_data_types() -> TypeCollection {
    let mut c = TypeCollection::default();

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

    // collections — proto-only since #81. `CollectionSummary` and friends
    // ship from the buf-generated `ship_ops_rpc::collections` client; the TS
    // shape lives in `packages/sdk/src/gen/ship/ops/v1/collection_pb.ts`.
    // Do not re-add a specta registration here without first confirming the
    // Rust definition is still specta-derived (it isn't).

    c
}
