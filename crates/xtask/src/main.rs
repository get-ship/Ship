//! Dev-time task runner.
//!
//! Usage:
//!   cargo xtask gen-types          # regenerate packages/ship-set/src/generated.ts
//!   cargo xtask gen-types --check  # verify generated file is up-to-date (CI)

mod gen_sdk;

use std::path::{Path, PathBuf};

use specta::TypeCollection;
use specta_typescript::{BigIntExportBehavior, Typescript};

pub(crate) fn repo_root_pub() -> PathBuf {
    repo_root()
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/
        .and_then(|p| p.parent()) // repo root
        .expect("cannot locate repo root")
        .to_path_buf()
}

fn collect_types() -> TypeCollection {
    let mut c = TypeCollection::default();

    // types/config.rs
    c.register::<compiler::StatusConfig>();
    c.register::<compiler::GitConfig>();
    c.register::<compiler::AiConfig>();
    c.register::<compiler::HookTrigger>();
    c.register::<compiler::HookConfig>();
    c.register::<compiler::PermissionConfig>();
    c.register::<compiler::ModeConfig>();
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
    c.register::<runtime::JobRecord>();
    c.register::<runtime::JobStatus>();

    // events/job.rs (job event payloads)
    c.register::<runtime::events::job::JobCreatedPayload>();
    c.register::<runtime::events::job::JobCompletedPayload>();
    c.register::<runtime::events::job::JobDispatchedPayload>();
    c.register::<runtime::events::job::JobGateRequestedPayload>();
    c.register::<runtime::events::job::JobGatePassedPayload>();
    c.register::<runtime::events::job::JobGateFailedPayload>();
    c.register::<runtime::events::job::JobBlockedPayload>();
    c.register::<runtime::events::job::JobMergedPayload>();
    c.register::<runtime::events::job::JobFailedPayload>();
    c.register::<runtime::events::job::JobUpdatePayload>();

    // events/job.rs (step event payloads + evidence types)
    c.register::<runtime::events::job::StepObservedPayload>();
    c.register::<runtime::events::job::StepTrigger>();
    c.register::<runtime::events::job::StepVerdict>();
    c.register::<runtime::events::job::StepRetryPayload>();

    // events/job.rs (pipeline types — already exported via JobRecord, but explicit for clarity)
    c.register::<runtime::events::job::Pipeline>();
    c.register::<runtime::events::job::PipelineStep>();
    c.register::<runtime::events::job::StepType>();

    // events/feedback.rs
    c.register::<runtime::events::feedback::FeedbackEvent>();
    c.register::<runtime::events::feedback::FeedbackKind>();
    c.register::<runtime::events::feedback::FeedbackIntent>();
    c.register::<runtime::events::feedback::FeedbackTarget>();
    c.register::<runtime::events::feedback::Selection>();
    c.register::<runtime::events::feedback::AuthorKind>();

    // events/annotation.rs (AuthorKind/Selection shared with feedback.rs — registered above)
    c.register::<runtime::events::annotation::AnnotationEvent>();
    c.register::<runtime::events::annotation::AnnotationKind>();
    c.register::<runtime::events::annotation::AnnotationIntent>();
    c.register::<runtime::events::annotation::AnnotationTarget>();

    // events/filter.rs
    c.register::<runtime::events::filter::EventFilter>();

    // collections/schema.rs — collection field-definition types (nested schemas)
    c.register::<runtime::collections::schema::FieldDef>();
    c.register::<runtime::collections::schema::FieldType>();

    // workspace/types.rs
    c.register::<runtime::Workspace>();
    c.register::<runtime::WorkspaceStatus>();
    c.register::<runtime::Environment>();
    c.register::<runtime::Process>();
    c.register::<runtime::ProcessStatus>();

    // workspace/types_session.rs
    c.register::<runtime::WorkspaceSession>();
    c.register::<runtime::WorkspaceSessionStatus>();
    c.register::<runtime::WorkspaceSessionRecord>();
    c.register::<runtime::WorkspaceProviderMatrix>();
    c.register::<runtime::WorkspaceRepairReport>();

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
        "help" | "--help" | "-h" => {
            eprintln!("Usage: cargo xtask <command>");
            eprintln!();
            eprintln!("Commands:");
            eprintln!("  gen-types          Regenerate packages/ship-set/src/generated.ts");
            eprintln!("  gen-types --check  Verify generated file is up-to-date (CI)");
            eprintln!("  gen-sdk            Emit packages/ship-set/sdk/*.json");
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

    let ui_out = repo_root().join("packages/ship-set/src/generated.ts");
    let ui_content = ts
        .export(&collect_types())
        .expect("specta-typescript export failed (ui)");

    let cloud_out = repo_root().join("packages/cloud-types/src/generated.ts");
    let cloud_content = build_cloud_types(&ts);

    if check {
        let mut stale = false;
        for (path, expected) in [(&ui_out, &ui_content), (&cloud_out, &cloud_content)] {
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
