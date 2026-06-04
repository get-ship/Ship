// FILE_CAP_OVERRIDE: flat payload module holds all canonical event payload structs.
// These 70+ structs are alphabetically organized and all logically co-located
// (they share one re-export surface, one serde dependency, and one registry
// identity in `ship_substrate::events::payload_registry`). Splitting into
// sub-modules would require consumers to import from sub-paths and fragment the
// re-export in ship-substrate's events::types. Split deferred to Spec 7b when a
// second major payload family (e.g. typed MCP tool payloads) warrants it.
use serde::{Deserialize, Serialize};

// -- Workspace aggregate -------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceCreated {
    pub workspace_id: String,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceActivated {
    pub agent_id: Option<String>,
    pub harnesses: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceArchived {}

// -- Session aggregate ---------------------------------------------------------

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionStarted {
    pub goal: Option<String>,
    #[serde(default)]
    pub workspace_id: String,
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub primary_harness: Option<String>,
    /// Project this session belongs to. Looked up at start from the
    /// actor's space_id or the workspace path. None for sessions that
    /// have no associated project (daemon services, ephemeral runs).
    #[serde(default)]
    pub space_id: Option<String>,
    /// Entry-point discriminator: `pipeline_step | chat_thread |
    /// direct_attach`. Defaults to `direct_attach` when not specified.
    #[serde(default)]
    pub kind: Option<String>,
    /// Set when this session is a step within a pipeline run.
    #[serde(default)]
    pub pipeline_run_id: Option<String>,
    /// Set when this session is a chat-thread invocation.
    #[serde(default)]
    pub chat_thread_id: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionProgress {
    #[serde(default)]
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalation: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionEnded {
    pub summary: Option<String>,
    pub duration_secs: Option<u64>,
    pub gate_result: Option<String>,
    #[serde(default)]
    pub updated_workspace_ids: Vec<String>,
    // Telemetry fields — populated at session end
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub harness: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub files_changed: Option<i64>,
    #[serde(default)]
    pub lines_added: Option<i64>,
    #[serde(default)]
    pub lines_removed: Option<i64>,
    #[serde(default)]
    pub tool_calls: Option<i64>,
    #[serde(default)]
    pub exit_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionDrainStarted {
    pub drained_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionDrainCompleted {
    pub ended_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionDrainAborted {
    pub resumed_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionToolCountIncremented {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionHeartbeat {
    pub last_activity_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionAttached {
    pub existing_session_id: String,
    pub workspace_id: String,
    pub branch: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStatusChanged {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionTurnFailed {
    /// Free-text error message from the spawn / run path.
    pub error: String,
}

// -- Skill aggregate -----------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillStarted {
    pub skill_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillCompleted {
    pub skill_id: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillFailed {
    pub skill_id: String,
    pub error: String,
}

/// Payload for `skill.attached_to_session`.
#[derive(Debug, Serialize, Deserialize)]
pub struct SkillAttachedToSession {
    pub skill_id: String,
    pub run_id: String,
    pub session_id: String,
    /// `"library"` today; `"operator"` / `"mode"` once those paths land.
    pub mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// `true` when an existing agent session was reused; `false` when
    /// the run_id was used as a synthetic session.
    pub reused: bool,
}

// -- Actor aggregate (v0.2.0 kernel) -------------------------------------------

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ActorCreated {
    pub kind: String,
    pub environment_type: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub parent_actor_id: Option<String>,
    #[serde(default)]
    pub restart_count: u32,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub harness: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub transport: Option<String>,
    #[serde(default)]
    pub persistence: Option<String>,
    #[serde(default)]
    pub system_prompt_hash: Option<String>,
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorWoke {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorSlept {
    pub idle_secs: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ActorStopped {
    pub reason: String,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorCrashed {
    pub error: String,
    pub restart_count: u32,
}

/// Payload for `actor.mailbox_full`. `entity_id` is the actor_id.
#[derive(Debug, Serialize, Deserialize)]
pub struct ActorMailboxFull {
    pub actor_id: String,
    pub dropped_event_type: String,
    pub dropped_event_id: String,
}

// Job event payloads (job.created, job.completed, job.failed, job.dispatched,
// step.*, gate.*) carry the runtime pipeline shape and live in substrate at
// `ship_substrate::events::job` since they reference `ship-pipelines` types.
// ship-types is a leaf crate and cannot pull in `ship-pipelines`.

// -- Artifact events -----------------------------------------------------------
//
// Artifact METADATA is event-sourced (ADR D6/D7): the write path is emit-only
// and `ship_substrate::projections::ArtifactProjection` materializes the
// `collection_artifacts` table from these events. BLOBS stay on the
// content-addressed blob plane (not event-sourced). These payloads are
// projection-complete: every metadata column the read model needs rides the
// event so the projection never has to read prior state.

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactCreated {
    pub id: String,
    pub scope: String,
    pub name: String,
    pub mime: String,
    pub size: u64,
    /// Relative blob path — the projection materializes it so reads can locate
    /// bytes without consulting the blob backend.
    pub path: String,
    /// Row creation timestamp (RFC3339) — the projection writes it verbatim.
    pub created_at: String,
    #[serde(default)]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub owner_actor: Option<String>,
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub collection_id: Option<String>,
    #[serde(default)]
    pub item_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactUpdated {
    pub id: String,
    pub scope: String,
    /// Map of changed metadata columns → new values. The projection applies a
    /// partial column UPDATE of exactly these keys (`name`/`owner_actor`/
    /// `expires_at`), leaving untouched columns alone.
    pub changed: serde_json::Value,
}

/// `artifact.moved` — a promote: insert the new metadata row (`new`) and delete
/// the old row (`old_id`) in one projection step. Both rows live in the same
/// space.db (promote changes the scope KIND, not the scope space), so this is a
/// single same-transaction insert+delete.
#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactMoved {
    /// Old metadata row id to delete.
    pub old_id: String,
    pub from_scope: String,
    pub to_scope: String,
    /// The full new metadata row to insert — projection-complete.
    pub new: ArtifactCreated,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactDeleted {
    pub id: String,
    pub scope: String,
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
}

// -- Config / Log aggregates ---------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigChanged {
    pub subject: String,
    pub detail: Option<String>,
}

/// Fine-grained, per-file config change (Format-2 reactivity).
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFileChanged {
    pub domain: String,
    pub path: String,
    pub content_hash: String,
}

/// Per-space settings override (`space.setting.changed`).
#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceSettingChanged {
    pub key: String,
    pub value_json: String,
}

/// Unified scope-discriminated settings change (`settings.changed`).
///
/// The keystone settings event: one type across all three scopes (SPACE /
/// ACCOUNT / RUNTIME). The scope rides the ENVELOPE — `EventScope` is chosen at
/// emit and the kernel routes to the matching per-scope db — so `scope` here is
/// the human-readable echo for read-model projections and audit, not the routing
/// key. `entity_id` is the setting key; `value_json` is the canonical JSON value.
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingChanged {
    pub key: String,
    pub value_json: String,
    /// The scope this setting was written in (`space` / `account` / `runtime`),
    /// matching `EventScope::as_str`.
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectLog {
    pub action: String,
    pub details: String,
}

// -- Supervisor aggregate ------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct SupervisorShuttingDown {
    #[serde(default)]
    pub reason: Option<String>,
}

// -- Auth aggregate ----------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSessionCreated {
    pub agent_id: String,
    pub scopes_json: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    pub created_at_unix: i64,
    pub expires_at_unix: i64,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSessionRevoked {
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSessionExpired {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGrantsProvisioned {
    pub grants: Vec<String>,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
}

// -- Daemon owner ------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonOwnerBound {
    pub source: String,
    pub bound_at_unix: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonOwnerUnbound {}

// -- Account lifecycle ------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountLoginStarted {
    pub redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountLoginCompleted {}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountLogout {
    pub had_token: bool,
    #[serde(default)]
    pub key_revoked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountKeyRefreshed {}

// -- Project registry --------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectRegistered {
    pub path: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSeen {
    pub seen_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectArchived {
    pub archived_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSwitched {
    pub space_id: String,
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous: Option<String>,
    pub at: String,
}

// -- Compile aggregate --------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct CompileCompleted {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_agent: Option<String>,
    pub compiled_at: String,
    pub ship_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins_installed: Vec<String>,
}

// -- Skill / Rule / Permission CRUD -----------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillCreated {
    pub scope: String,
    pub name: String,
    pub content_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillUpdated {
    pub scope: String,
    pub content_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillDeleted {
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleCreated {
    pub file_name: String,
    pub content_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleUpdated {
    pub file_name: String,
    pub content_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleDeleted {
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionsUpdated {
    pub scope: String,
    pub content_hash: String,
    pub permissions_json: String,
}

// -- Watcher change events (space, ephemeral) --------------------------------

/// `agent.changed` — watcher ping when agent profile shape changes.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentChanged {
    pub path: String,
    pub content_hash: String,
}

/// `agent.def.updated` — Format-2 agent-def change notification.
///
/// Carries ONLY the def file's rel `path` (under `.ship/agents/`) and its
/// `content_hash`. The space rides the ENVELOPE scope_key (Space-scoped) — NOT
/// the payload — mirroring [`ConfigFileChanged`]. This is a Bucket-2
/// notification: the file is truth, so it NEVER carries content; the read-only
/// def index re-reads the file on the ping. An empty `content_hash` is the
/// deleted-file marker (read failed) → the index drops the row.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentDefUpdated {
    pub path: String,
    pub content_hash: String,
}

/// `skill.changed` — watcher ping when skill shape changes.
#[derive(Debug, Serialize, Deserialize)]
pub struct SkillChanged {
    pub path: String,
    pub content_hash: String,
}

/// `view.changed` — watcher ping when view shape changes.
#[derive(Debug, Serialize, Deserialize)]
pub struct ViewChanged {
    pub path: String,
    pub content_hash: String,
}

/// `pipeline.changed` — watcher ping when pipeline definition changes.
#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineChanged {
    pub path: String,
    pub content_hash: String,
}

/// `rule.changed` — watcher ping when rule shape changes.
#[derive(Debug, Serialize, Deserialize)]
pub struct RuleChanged {
    pub path: String,
    pub content_hash: String,
}

/// `project.changed` — legacy; kept for backward compat.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectChanged {
    pub path: String,
    pub content_hash: String,
}

// -- Daemon lifecycle --------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonStartupCompleted {
    pub pid: u32,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonShutdownCompleted {
    pub pid: u32,
    #[serde(default)]
    pub uptime_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonPortBindFailed {
    pub port: u16,
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonCrashed {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonPidfileAcquired {
    pub pid: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonPidfileStaleTaken {
    pub stale_pid: u32,
    pub new_pid: u32,
}

// -- Mesh events (all device) ------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshBroadcast {
    pub message_type: String,
    #[serde(default)]
    pub sender: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshDeregister {
    pub actor_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshDeregistered {
    pub actor_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshDiscoverResponse {
    pub peers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshMessage {
    pub from: String,
    pub to: String,
    pub message_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshRegister {
    pub actor_id: String,
    pub namespace: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshRegistered {
    pub actor_id: String,
    pub namespace: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshSend {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshSendFailed {
    pub from: String,
    pub to: String,
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshStatus {
    pub actor_id: String,
    pub status: String,
}

// -- Spec-6 new events -------------------------------------------------------

/// `agent.task.complete` — task-creation completion in space history.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentTaskComplete {
    pub task_id: String,
    pub task_type: String,
    #[serde(default)]
    pub space_id: Option<String>,
}

/// `human.response` — inbound human reply (telegram webhook) tied to space conversation.
#[derive(Debug, Serialize, Deserialize)]
pub struct HumanResponse {
    pub conversation_id: String,
    pub text: String,
    #[serde(default)]
    pub sender: Option<String>,
}

// -- Kernel actor events -----------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct KernelActorRestored {
    pub actor_id: String,
    pub event_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KernelActorSnapshot {
    pub actor_id: String,
    pub event_count: u64,
    pub last_event_id: Option<String>,
    pub size_bytes: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KernelActorSuspended {
    pub actor_id: String,
    pub reason: String,
}

// -- Pipeline step events added in Spec 6 ------------------------------------

/// `pipeline.step.retry` — a step was retried after a transient failure.
#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineStepRetry {
    pub pipeline_run_id: String,
    pub step_id: String,
    pub retry_count: u32,
    pub reason: String,
}

// -- Studio events added in Spec 6 -------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct StudioAnnotationCreated {
    pub annotation_id: String,
    pub session_id: String,
    pub text: String,
    #[serde(default)]
    pub artifact_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudioFeedbackCreated {
    pub feedback_id: String,
    pub rating: Option<i32>,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudioChatTurn {
    pub session_id: String,
    pub text: String,
    #[serde(default)]
    pub agent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudioReply {
    pub actor_id: String,
    pub text: String,
}

// -- Sync events added in Spec 6 ---------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPushFailed {
    pub error: String,
    pub target: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStarted {
    pub target: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStopped {
    pub target: String,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPushCompleted {
    pub events_pushed: u64,
    pub duration_ms: u64,
}

// -- Process events ----------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessOrphaned {
    pub pid: u32,
    pub cmd: String,
    pub last_known_slug: Option<String>,
    pub detected_by: String,
}

// -- Collection events -------------------------------------------------------

/// `collection.item_created` — projection-complete: carries the full validated
/// item so `CollectionProjection` can materialize the row from the event alone
/// (event-sourced, no read-back). `data` is the validated item object (schema
/// fields only); `created_at`/`updated_at` are the row timestamps.
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionItemCreated {
    pub collection_id: String,
    pub item_id: String,
    #[serde(default)]
    pub item_type: Option<String>,
    /// The full validated item data (schema fields → values). The projection
    /// writes exactly these columns into the collection table.
    pub data: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// `collection.item_updated` — projection-complete: carries the patch (the
/// fields to set) plus the new `updated_at`. The projection applies a partial
/// column update; untouched columns keep their values.
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionItemUpdated {
    pub collection_id: String,
    pub item_id: String,
    /// The patch — only the fields provided on update. The projection writes
    /// exactly these columns (plus `updated_at`); other columns are untouched.
    pub data: serde_json::Value,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionItemDeleted {
    pub collection_id: String,
    pub item_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionCreated {
    pub collection_id: String,
    pub schema_json: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionSchemaChanged {
    pub collection_id: String,
    pub schema_json: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_created_payload_deserializes() {
        let original = WorkspaceCreated {
            workspace_id: "ws-1".to_string(),
            path: None,
        };
        let json = serde_json::to_string(&original).unwrap();
        let decoded: WorkspaceCreated = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.workspace_id, "ws-1");
    }

    #[test]
    fn session_started_payload_has_defaults() {
        let decoded: SessionStarted = serde_json::from_str(r#"{"goal": "test"}"#).unwrap();
        assert_eq!(decoded.workspace_id, "");
        assert!(decoded.agent_id.is_none());
    }
}
