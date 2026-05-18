//! Dev-time task runner.
//!
//! Usage:
//!   cargo xtask gen-types          # regenerate packages/sdk/src/generated.ts
//!   cargo xtask gen-types --check  # verify generated file is up-to-date (CI)

mod gen_sdk;
mod lint_layers;

use std::path::{Path, PathBuf};

use specta::TypeCollection;
use specta_typescript::{BigIntExportBehavior, Typescript};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/
        .and_then(|p| p.parent()) // repo root
        .expect("cannot locate repo root")
        .to_path_buf()
}

/// Types whose canonical home is `packages/sdk/src/generated.ts`:
/// project config, permissions, skills, agent profiles, runtime workspace +
/// job/event projections. Consumers (sdk/types.ts) import from this file
/// directly; do not move members out of this set without updating consumers.
fn collect_types() -> TypeCollection {
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
    c.register::<ship_platform::projections::job::PipelineRun>();
    c.register::<ship_platform::projections::job::JobStatus>();

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

    // collections/schema.rs — collection field-definition types (nested schemas)
    c.register::<ship_platform::collections::schema::FieldDef>();
    c.register::<ship_platform::collections::schema::FieldType>();

    // workspace/types_session.rs (the workspace primitive retired; only
    // session-shaped types remain)
    c.register::<ship_platform::sessions::Session>();
    c.register::<ship_platform::sessions::SessionStatus>();
    c.register::<ship_platform::sessions::ProviderMatrix>();
    c.register::<ship_platform::sessions::ProviderMatrixRepairReport>();

    c
}

/// Framework data types that cross the daemon ↔ ui-runtime boundary.
/// Output lands at `packages/sui-runtime/src/generated.ts` and is
/// re-exported from `@ship/sui-runtime`. Add new entries here whenever a
/// daemon HTTP/SSE response or a runtime payload struct gains a
/// `#[derive(Type)]`. Drift between Rust and these registrations is
/// caught by `cargo xtask gen-types --check` in CI.
fn collect_framework_types() -> TypeCollection {
    let mut c = TypeCollection::default();

    // ship-sui — view payload + resolver.
    c.register::<ship_sui::view_payload::ViewPayload>();
    c.register::<ship_sui::resolver::ResolvedView>();
    c.register::<ship_sui::resolver::ResolveError>();

    // shipd::views — view list / manifest / source / write APIs.
    c.register::<shipd::views::ViewManifest>();
    c.register::<shipd::views::ViewListEntry>();
    c.register::<shipd::views::RailMeta>();
    c.register::<shipd::views::source_api::ViewSourceResponse>();
    c.register::<shipd::views::write_api::WriteTemplateRequest>();
    c.register::<shipd::views::write_api::WriteManifestRequest>();
    c.register::<shipd::views::write_api::WriteResponse>();
    c.register::<shipd::views::write_api::WriteDiagnostics>();
    c.register::<shipd::views::write_api::LintFinding>();
    c.register::<shipd::views::resolve::ManifestRail>();

    // shipd::services::git — overview, diff, status buckets.
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

    // shipd::services::agents — summary list payload.
    c.register::<ship_platform::services::agents::view_state::AgentSummary>();
    c.register::<ship_platform::services::agents::view_state::AgentsSummariesResponse>();

    // ship-platform::account — Ship Cloud account state for the chrome chip.
    c.register::<ship_platform::account::types::AccountState>();
    c.register::<ship_platform::account::types::MeView>();

    // view-authoring — contract types shared with the chrome edit-mode UI.
    c.register::<ship_platform::view_authoring::CompileError>();
    c.register::<ship_platform::view_authoring::SnapshotMeta>();
    c.register::<ship_platform::view_authoring::ViewReadSourceRequest>();
    c.register::<ship_platform::view_authoring::ViewReadSourceResponse>();
    c.register::<ship_platform::view_authoring::ViewWriteSourceRequest>();
    c.register::<ship_platform::view_authoring::ViewWriteSourceResponse>();
    c.register::<ship_platform::view_authoring::ViewCompileCheckRequest>();
    c.register::<ship_platform::view_authoring::ViewCompileCheckResponse>();
    c.register::<ship_platform::view_authoring::ViewCollectionsSchemaRequest>();
    c.register::<ship_platform::view_authoring::ViewListSiblingsRequest>();
    c.register::<ship_platform::view_authoring::SiblingView>();
    c.register::<ship_platform::view_authoring::ViewListSiblingsResponse>();
    c.register::<ship_platform::view_authoring::ViewThemeTokensRequest>();
    c.register::<ship_platform::view_authoring::ViewThemeTokensResponse>();
    c.register::<ship_platform::view_authoring::ViewPreviewSnapshotRequest>();
    c.register::<ship_platform::view_authoring::ViewPreviewSnapshotResponse>();
    c.register::<ship_platform::view_authoring::AnnotationKind>();
    c.register::<ship_platform::view_authoring::ViewAnnotateRequest>();
    c.register::<ship_platform::view_authoring::ViewAnnotateResponse>();
    c.register::<ship_platform::view_authoring::ViewListSnapshotsRequest>();
    c.register::<ship_platform::view_authoring::ViewListSnapshotsResponse>();
    c.register::<ship_platform::view_authoring::ViewRestoreSnapshotRequest>();
    c.register::<ship_platform::view_authoring::ViewRestoreSnapshotResponse>();
    c.register::<ship_platform::view_authoring::ViewEditModeEnterRequest>();
    c.register::<ship_platform::view_authoring::ViewEditModeExitRequest>();
    c.register::<ship_platform::view_authoring::ViewEditModeAck>();

    c
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "gen-types" => {
            let check = args.iter().any(|a| a == "--check");
            gen_types(check);
        }
        "gen-sdk" => {
            if let Err(e) = gen_sdk::run(&repo_root()) {
                eprintln!("gen-sdk failed: {e:#}");
                std::process::exit(1);
            }
        }
        "lint-layers" => {
            if let Err(e) = lint_layers::run(&repo_root()) {
                eprintln!("lint-layers failed: {e}");
                std::process::exit(1);
            }
        }
        "help" | "--help" | "-h" => {
            eprintln!("Usage: cargo xtask <command>");
            eprintln!();
            eprintln!("Commands:");
            eprintln!("  gen-types          Regenerate packages/sdk/src/generated.ts");
            eprintln!("  gen-types --check  Verify generated file is up-to-date (CI)");
            eprintln!("  gen-sdk            Emit packages/sdk/src/generated/sdk-*.json + TS bindings");
            eprintln!("  lint-layers        Verify substrate/platform/application boundaries");
        }
        other => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}

fn gen_types(check: bool) {
    let ts = Typescript::default()
        .bigint(BigIntExportBehavior::Number)
        .header("// This file is auto-generated by `cargo xtask gen-types`. Do not edit.")
        .framework_header("");

    let ui_out = repo_root().join("packages/sdk/src/generated.ts");
    let ui_content = ts
        .export(&collect_types())
        .expect("specta-typescript export failed (ui)");

    let cloud_out = repo_root().join("packages/cloud-types/src/generated.ts");
    let cloud_content = build_cloud_types(&ts);

    let framework_out = repo_root().join("packages/sui-runtime/src/generated.ts");
    let framework_content = ts
        .export(&collect_framework_types())
        .expect("specta-typescript export failed (framework)");

    if check {
        let mut stale = false;
        for (path, expected) in [
            (&ui_out, &ui_content),
            (&cloud_out, &cloud_content),
            (&framework_out, &framework_content),
        ] {
            let existing = std::fs::read_to_string(path).unwrap_or_default();
            if existing != *expected {
                eprintln!("Out of date: {}", path.display());
                stale = true;
            }
        }
        if stale {
            eprintln!("Run `cargo xtask gen-types` and commit the result.");
            std::process::exit(1);
        }
        eprintln!("Types are up-to-date.");
        return;
    }

    std::fs::write(&ui_out, &ui_content).expect("failed to write ui generated.ts");
    eprintln!("Wrote {}", ui_out.display());
    std::fs::write(&cloud_out, &cloud_content).expect("failed to write cloud generated.ts");
    eprintln!("Wrote {}", cloud_out.display());
    std::fs::write(&framework_out, &framework_content)
        .expect("failed to write framework generated.ts");
    eprintln!("Wrote {}", framework_out.display());
}

/// Build the cloud-side generated.ts: EventEnvelope struct + all event-type
/// string constants read from `ship-types/src/event_types.rs`. Cloud's TS
/// surface must mirror Rust's event taxonomy exactly.
fn build_cloud_types(ts: &Typescript) -> String {
    let mut c = TypeCollection::default();
    c.register::<ship_types::EventEnvelope>();
    let struct_ts = ts
        .export(&c)
        .expect("specta-typescript export failed (cloud)");

    let event_types_src = repo_root().join("crates/core/ship-types/src/event_types.rs");
    let src = std::fs::read_to_string(&event_types_src)
        .unwrap_or_else(|e| panic!("read {}: {e}", event_types_src.display()));
    let mut consts = String::new();
    for line in src.lines() {
        let trimmed = line.trim();
        // pub const NAME: &str = "value";
        let Some(rest) = trimmed.strip_prefix("pub const ") else { continue };
        let Some((name, rest)) = rest.split_once(": &str =") else { continue };
        let rest = rest.trim().trim_end_matches(';').trim();
        if !rest.starts_with('"') || !rest.ends_with('"') {
            continue;
        }
        consts.push_str(&format!(
            "export const {name} = {value} as const;\n",
            name = name.trim(),
            value = rest,
        ));
    }

    format!("{struct_ts}\n// ── Event-type constants (mirrored from crates/core/ship-types/src/event_types.rs) ──\n\n{consts}")
}
