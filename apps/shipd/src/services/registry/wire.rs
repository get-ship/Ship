//! Named wire shapes for shipd-orchestrated registry ops — the daemon-side
//! counterpart to each domain's ship-platform `wire_types.rs`. Used where a
//! response is assembled in the daemon (joining projections, PTY state, view
//! helpers) rather than returned by a pure ship-platform state fn.
//!
//! These satisfy the typed-op single-source contract: every return is a named
//! `Serialize` type, never an anonymous `json!`, and dispatch is byte-identical.
//! Irreducibly-dynamic sub-parts stay `serde_json::Value` (passthrough → exact
//! bytes). Their `data.ts` handles stay `unknown` until specta-registered
//! (incremental and additive — see `gen_sdk::data_ts::MIGRATED_RETURN_TYPES`).

use serde::Serialize;
use serde_json::Value;

use ship_artifacts::types::ArtifactRecord;
use ship_platform::services::git::diff::ParsedDiff;
use ship_platform::services::git::state::{BranchSummary, CommitSummary, StatusFile};
use ship_pipeline_runtime::pipelines::state as pipelines_state;
use ship_platform::services::sessions::types::SessionMeta;
use ship_platform::services::skills::state as skills_state;
use ship_platform::services::skills::view_state::SkillSummary;
use ship_substrate::projections::actor::ActorRow;

// ── sessions / events / artifacts ──────────────────────────────────────

/// Wire shape of `ship.agents.sessions()`.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SessionsListResponse {
    pub items: Vec<SessionMeta>,
}

/// Wire shape of `ship.artifacts.list()`. Unused by the typed `ArtifactsService`
/// handler (it builds the proto `ArtifactsListResponse` directly), but kept
/// here as a reference shape for the specta TS surface.
#[derive(Serialize)]
#[allow(dead_code)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ArtifactsListResponse {
    pub items: Vec<ArtifactRecord>,
    pub count: usize,
    pub scope: String,
}

// ── skills ────────────────────────────────────────────────────────────

/// Wire shape of `ship.skills.list()`.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SkillsIndexResponse {
    pub items: Vec<skills_state::IndexEntry>,
}

/// Wire shape of `ship.skills.summaries()` — filtered + grouped rail data.
#[derive(Serialize)]
pub struct SkillsSummariesResponse {
    pub items: Vec<SkillSummary>,
    pub filtered: Vec<SkillSummary>,
    pub groups: Vec<Value>,
    pub kind_counts: Vec<Value>,
    pub tags: std::collections::BTreeSet<String>,
    pub total: usize,
    pub filtered_count: usize,
    pub q: String,
    pub active_kind: String,
    pub active_tag: String,
}

/// Wire shape of `ship.skills.file()` — HTML-escaped file content.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SkillFileResponse {
    pub path: String,
    pub name: String,
    pub ext: String,
    pub content: String,
}

// ── pipelines ─────────────────────────────────────────────────────────

/// Wire shape of `ship.pipelines.list()`.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PipelinesIndexResponse {
    pub items: Vec<pipelines_state::IndexEntry>,
}

/// Wire shape of `ship.pipelines.get()` — raw pipeline doc at a scope.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PipelineGetResponse {
    pub name: String,
    pub scope: pipelines_state::ScopeTag,
    pub format: String,
    pub doc: Value,
}

// ── git ───────────────────────────────────────────────────────────────

/// Wire shape of `ship.git.log()`.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GitLogResponse {
    pub commits: Vec<CommitSummary>,
}

/// Wire shape of `ship.git.branches()`.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GitBranchesResponse {
    pub branches: Vec<BranchSummary>,
    pub ahead: usize,
    pub behind: usize,
}

/// Wire shape of `ship.git.overview()` — one-call default-route payload.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GitOverviewResponse {
    pub workspace_id: Option<String>,
    pub branch: Option<String>,
    pub head_short: Option<String>,
    pub clean: bool,
    pub total_changed: usize,
    pub staged: Vec<StatusFile>,
    pub modified: Vec<StatusFile>,
    pub untracked: Vec<StatusFile>,
    pub commits: Vec<CommitSummary>,
    pub branches: Vec<BranchSummary>,
    pub ahead: usize,
    pub behind: usize,
}

/// Wire shape of `ship.git.diff()`.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GitDiffResponse {
    pub sha: String,
    pub short_sha: String,
    pub is_working: bool,
    pub diff_text: String,
    pub parsed: ParsedDiff,
}

// ── collections ───────────────────────────────────────────────────────

/// One field row in a collection summary. Single Rust source for the TS
/// `CollectionFieldRecord` (was hand-written in @ship/sdk-client). Serde output
/// is byte-identical to the prior `json!` in registry/collections.rs.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CollectionFieldRecord {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub field_type: String,
    pub required: bool,
    pub values: Option<Vec<String>>,
}

/// One row of `ship.collections.list()` — schema metadata + per-collection
/// stats. Single Rust source for the TS `CollectionSummary` (was hand-written
/// AND incomplete — missing the 5 stats fields). Byte-identical to the json!.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CollectionSummary {
    pub id: String,
    pub name: Option<String>,
    pub field_count: usize,
    pub fields: Vec<CollectionFieldRecord>,
    pub builtin: bool,
    pub item_count: usize,
    pub status_field: Option<String>,
    pub status_values: Vec<String>,
    #[cfg_attr(feature = "specta", specta(type = std::collections::HashMap<String, i64>))]
    pub status_counts: serde_json::Map<String, Value>,
    pub last_activity: Option<String>,
}

/// Wire shape of `ship.collections.list()`.
#[derive(Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CollectionsListResponse {
    pub items: Vec<CollectionSummary>,
}

/// Wire shape of `ship.collections.detail()`.
#[derive(Serialize)]
pub struct CollectionDetailResponse {
    pub schema: Value,
    pub items: Vec<Value>,
    pub total_items: usize,
}

/// Wire shape of `ship.collections.items()`.
#[derive(Serialize)]
pub struct CollectionItemsResponse {
    pub items: Vec<Value>,
    pub total_items: usize,
}

/// Wire shape of `ship.collections.state()`.
#[derive(Serialize)]
pub struct CollectionsStateResponse {
    pub collections: Vec<Value>,
    pub active_collection: String,
    pub schema: Value,
    pub items: Vec<Value>,
    pub total_items: usize,
}

// ── actors / mesh (dispatchable but intentionally absent from OPS) ─────

/// Wire shape of `ship.actors.list()`.
#[derive(Serialize)]
pub struct ActorsListResponse {
    pub items: Vec<ActorRow>,
}

/// Wire shape of `ship.mesh.snapshot()` — kernel actors + processes +
/// integrations. The rows are heterogeneous, daemon-assembled objects, kept as
/// `Value` so the payload is byte-identical to the prior hand-built `json!`.
#[derive(Serialize)]
pub struct MeshSnapshotResponse {
    pub actors: Vec<Value>,
    pub processes: Vec<Value>,
    pub integrations: Vec<Value>,
    pub counts: Value,
}
