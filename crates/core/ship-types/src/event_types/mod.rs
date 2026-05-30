pub mod mesh;
pub use mesh::*;

// All lines are constant definitions; no logic present. The `ALL` slice is kept for
// backward compat but is derived from `event_registry::registry()` via a consistency
// test. New callers should use `event_registry::all_names()` or `event_registry::registry()`.
pub const WORKSPACE_CREATED: &str = "workspace.created";
pub const WORKSPACE_ACTIVATED: &str = "workspace.activated";
pub const WORKSPACE_ARCHIVED: &str = "workspace.archived";
pub const SESSION_STARTED: &str = "session.started";
pub const SESSION_PROGRESS: &str = "session.progress";
pub const SESSION_ENDED: &str = "session.ended";
pub const SESSION_DRAIN_STARTED: &str = "session.drain_started";
pub const SESSION_DRAIN_COMPLETED: &str = "session.drain_completed";
pub const SESSION_DRAIN_ABORTED: &str = "session.drain_aborted";
pub const SESSION_TOOL_COUNT_INCREMENTED: &str = "session.tool_count_incremented";
pub const SESSION_ATTACHED: &str = "session.attached";
pub const SESSION_HEARTBEAT: &str = "session.heartbeat";
pub const ACTOR_CREATED: &str = "actor.created";
pub const ACTOR_WOKE: &str = "actor.woke";
pub const ACTOR_SLEPT: &str = "actor.slept";
pub const ACTOR_STOPPED: &str = "actor.stopped";
pub const ACTOR_CRASHED: &str = "actor.crashed";
pub const PIPELINE_RUN_STARTED: &str = "pipeline.run.started";
pub const PIPELINE_RUN_COMPLETED: &str = "pipeline.run.completed";
pub const PIPELINE_RUN_FAILED: &str = "pipeline.run.failed";
pub const PIPELINE_RUN_DISPATCHED: &str = "pipeline.run.dispatched";
pub const PIPELINE_RUN_UPDATED: &str = "pipeline.run.updated";
pub const CONFIG_CHANGED: &str = "config.changed";
/// Fine-grained per-file config change (path + content_hash) — see
/// `ConfigFileChanged`. Emitted by the watcher alongside the coarse `*.changed`.
pub const CONFIG_FILE_CHANGED: &str = "config.file_changed";
pub const AGENT_CHANGED: &str = "agent.changed";
pub const SKILL_CHANGED: &str = "skill.changed";
pub const VIEW_CHANGED: &str = "view.changed";
pub const PIPELINE_CHANGED: &str = "pipeline.changed";
pub const RULE_CHANGED: &str = "rule.changed";
pub const PROJECT_CHANGED: &str = "project.changed";
pub const PROJECT_LOG: &str = "project.log";
pub const SKILL_STARTED: &str = "skill.started";
pub const SKILL_COMPLETED: &str = "skill.completed";
pub const SKILL_FAILED: &str = "skill.failed";
/// Emitted after `skill.started` for library-mode invokes. Records the
/// session_id (existing chat session or the synthetic invoke run id)
/// that the skill's reference content is attached to. `entity_id` is the
/// skill id. Payload: `SkillAttachedToSession`.
pub const SKILL_ATTACHED_TO_SESSION: &str = "skill.attached_to_session";
pub const SUPERVISOR_SHUTTING_DOWN: &str = "supervisor.shutting_down";
pub const SUPERVISOR_SWEEP_STARTED: &str = "supervisor.sweep_started";
pub const SUPERVISOR_SWEEP_COMPLETED: &str = "supervisor.sweep_completed";
pub const SUPERVISOR_DRIFT_DETECTED: &str = "supervisor.drift_detected";
pub const SUPERVISOR_COHERENCE_RESTORED: &str = "supervisor.coherence_restored";
pub const PROCESS_SPAWNED: &str = "process.spawned";
pub const PROCESS_EXITED: &str = "process.exited";
pub const PROCESS_SIGNAL_SENT: &str = "process.signal_sent";
pub const PROCESS_ORPHANED: &str = "process.orphaned";
pub const PROCESS_KILLED: &str = "process.killed";
pub const EXECUTOR_SESSION_DESTROYED: &str = "executor.session_destroyed";
pub const EXECUTOR_WINDOW_DESTROYED: &str = "executor.window_destroyed";
pub const DAEMON_STARTUP_COMPLETED: &str = "daemon.startup_completed";
pub const DAEMON_SHUTDOWN_COMPLETED: &str = "daemon.shutdown_completed";
pub const DAEMON_CRASHED: &str = "daemon.crashed";
pub const DAEMON_PORT_BIND_FAILED: &str = "daemon.port_bind_failed";
pub const DAEMON_PIDFILE_ACQUIRED: &str = "daemon.pidfile_acquired";
pub const DAEMON_PIDFILE_STALE_TAKEN: &str = "daemon.pidfile_stale_taken";
pub const PIPELINE_STEP_COMPLETED: &str = "pipeline.step.completed";
pub const PIPELINE_STEP_FAILED: &str = "pipeline.step.failed";
pub const PIPELINE_RUN_CANCELLED: &str = "pipeline.run.cancelled";
pub const PIPELINE_RUN_MERGED: &str = "pipeline.run.merged";
pub const PIPELINE_STEP_SPAWN_FAILED: &str = "pipeline.step.spawn_failed";
pub const ARTIFACT_CREATED: &str = "artifact.created";
pub const ARTIFACT_UPDATED: &str = "artifact.updated";
pub const ARTIFACT_MOVED: &str = "artifact.moved";
pub const ARTIFACT_DELETED: &str = "artifact.deleted";

/// Emitted when a user sends a turn in a Ship command-bar chat session.
/// Scoped to the originating Ship `session_id` via `EventEnvelope.session_id`
/// so per-session filtering in the MCP `EventRelay` can route the turn only
/// to the intended long-lived agent. Payload: `{ session_id, text, agent_id? }`.
pub const STUDIO_CHAT_TURN: &str = "studio.chat_turn";

/// Emitted when an agent replies into a Ship chat. The `entity_id` carries
/// the target actor's id; the payload is `{ actor_id, text }`. Dashboards
/// and views tail this event to render reply bubbles. Replaces the
/// in-memory event_tx broadcast that only worked for registry-backed
/// PtySessions.
pub const STUDIO_REPLY: &str = "studio.reply";

/// Emitted by `ship compile` at end of a successful run. Records what was
/// compiled (active agent, harness, plugins, ship version) for the
/// `project_compile_state` projection. `entity_id` is the space_id.
pub const COMPILE_COMPLETED: &str = "compile.completed";

/// Project registry — first time a project path is seen by the daemon.
/// `entity_id` is the space_id (stable hash of the canonical path).
pub const PROJECT_REGISTERED: &str = "project.registered";
/// Project registry — `last_seen_at` bump (re-open, switch, reattach).
pub const PROJECT_SEEN: &str = "project.seen";
/// Project registry — user removed the project from their registry.
pub const PROJECT_ARCHIVED: &str = "project.archived";
/// Active-project switch. `entity_id` is the newly-active space_id.
/// Payload carries the new + previous ids so chrome subscribers can
/// distinguish a no-op set from a true context change.
pub const PROJECT_SWITCHED: &str = "project.switched";

/// Auth — a daemon session (CLI / MCP / Studio-local) was issued.
/// `entity_id` is the session token. The auth_sessions projection
/// materializes one row per session.
pub const AUTH_SESSION_CREATED: &str = "auth.session.created";
/// Auth — a session was explicitly revoked (e.g. `ship auth revoke`).
pub const AUTH_SESSION_REVOKED: &str = "auth.session.revoked";
/// Auth — a session passed its `expires_at_unix`.
pub const AUTH_SESSION_EXPIRED: &str = "auth.session.expired";

/// Daemon-owner binding — a user paired with this daemon. `entity_id`
/// is the user_id. The daemon_owner projection materializes the latest.
pub const DAEMON_OWNER_BOUND: &str = "daemon.owner.bound";
/// Daemon-owner binding — the user unpaired (`ship logout`).
pub const DAEMON_OWNER_UNBOUND: &str = "daemon.owner.unbound";

/// Account — PKCE login flow opened. `entity_id` is the opaque state_id
/// returned to the CLI; payload carries the redirect_uri the CLI bound.
/// No credential material is emitted.
pub const ACCOUNT_LOGIN_STARTED: &str = "account.login_started";
/// Account — PKCE login flow finished: token stored + daemon-owner bound.
/// `entity_id` is the user_id.
pub const ACCOUNT_LOGIN_COMPLETED: &str = "account.login_completed";
/// Account — local token cleared. `entity_id` is the user_id of the
/// previously-bound owner, or `"none"` when the call was idempotent.
pub const ACCOUNT_LOGOUT: &str = "account.logout";
/// Account — `/me` refreshed against the gateway with the stored bearer.
/// `entity_id` is the user_id.
pub const ACCOUNT_KEY_REFRESHED: &str = "account.key_refreshed";

/// Skill CRUD — a skill SKILL.md (project- or user-scoped) was written
/// for the first time. `entity_id` is the skill id.
pub const SKILL_CREATED: &str = "skill.created";
/// Skill CRUD — content or front-matter of an existing skill changed.
/// `entity_id` is the skill id.
pub const SKILL_UPDATED: &str = "skill.updated";
/// Skill CRUD — a skill was removed from disk. `entity_id` is the
/// skill id.
pub const SKILL_DELETED: &str = "skill.deleted";

/// Rule CRUD — a `.ship/rules/<file>.md` was created. `entity_id` is
/// the canonical rule id (file name minus the optional `NNNN-` prefix
/// and `.md` suffix).
pub const RULE_CREATED: &str = "rule.created";
/// Rule CRUD — content of an existing rule changed.
pub const RULE_UPDATED: &str = "rule.updated";
/// Rule CRUD — rule file was removed.
pub const RULE_DELETED: &str = "rule.deleted";

/// Permissions — `.ship/permissions.jsonc` was rewritten. `entity_id`
/// is the space_id (or `"global"` for user-scope writes).
pub const PERMISSIONS_UPDATED: &str = "permissions.updated";

/// Capability provisioning — an agent's approved grant baseline was written.
/// `entity_id` is the agent_id; the `provisioned_grants` projection
/// materializes one row per agent (latest write wins). Emitted at install
/// (source="install", seeded from the agent's resolved skill grants) and on
/// operator approval (source="operator"). The mint path reads this baseline;
/// `requires:`/`grants:` in skills are intent, never authority.
pub const AGENT_GRANTS_PROVISIONED: &str = "agent.grants.provisioned";

/// Session status transition (e.g. Idle → Thinking). `entity_id` is
/// the session id. Useful for "what's the agent doing right now?" UIs.
pub const SESSION_STATUS_CHANGED: &str = "session.status_changed";
/// A turn failed to spawn or run end-to-end. `entity_id` is the session
/// id. Distinct from `session.ended` — the session stays alive, the
/// individual turn is the unit of failure.
pub const SESSION_TURN_FAILED: &str = "session.turn_failed";

/// Per-space settings override — a single (space, key) value was set. The
/// space scope rides the envelope (`space_id`, stamped via `with_context`),
/// NOT the payload — mirror `CONFIG_FILE_CHANGED`. Layered OVER the global
/// daemon settings (`settings.state`); space overrides win per-key. `entity_id`
/// is the setting key. Payload: `SpaceSettingChanged`.
pub const SPACE_SETTING_CHANGED: &str = "space.setting.changed";

/// Actor — mailbox rejected a message because the channel was full.
/// `entity_id` is the actor_id.
/// A1: device, durable.
pub const ACTOR_MAILBOX_FULL: &str = "actor.mailbox_full";

/// Agent — a dispatched task ran to completion. `entity_id` is the task id.
/// A1: space, durable.
pub const AGENT_TASK_COMPLETE: &str = "agent.task.complete";

/// Human gateway — a human sent a response into the system (e.g. Telegram
/// webhook reply). `entity_id` is the session or conversation id.
/// A1: space, durable.
pub const HUMAN_RESPONSE: &str = "human.response";

/// Kernel — an actor was restored from a persisted snapshot.
/// `entity_id` is the actor_id.
/// A1: device, durable.
pub const KERNEL_ACTOR_RESTORED: &str = "kernel.actor.restored";

/// Kernel — a snapshot of actor state was written to durable storage.
/// `entity_id` is the actor_id.
/// A1: device, durable.
pub const KERNEL_ACTOR_SNAPSHOT: &str = "kernel.actor.snapshot";

/// Kernel — an actor was suspended (paused, not stopped).
/// `entity_id` is the actor_id.
/// A1: device, durable.
pub const KERNEL_ACTOR_SUSPENDED: &str = "kernel.actor.suspended";

/// Pipeline — a step was retried after a transient failure.
/// `entity_id` is the pipeline run id.
/// A1: space, durable.
pub const PIPELINE_STEP_RETRY: &str = "pipeline.step.retry";

/// Studio — a user annotation was created on a session or artifact.
/// `entity_id` is the annotation id.
/// A1: space, durable.
pub const STUDIO_ANNOTATION_CREATED: &str = "studio.annotation.created";

/// Studio — a feedback record was created (thumbs-up/down, rating, etc.).
/// `entity_id` is the feedback id.
/// A1: space, durable.
pub const STUDIO_FEEDBACK_CREATED: &str = "studio.feedback.created";

/// Studio — a generic message was emitted from a studio session.
/// `entity_id` is the session id.
/// A1: space, durable.
pub const STUDIO_MESSAGE: &str = "studio.message";

/// Sync — a push to the remote sync target failed.
/// A1: device, durable.
pub const SYNC_PUSH_FAILED: &str = "sync.push.failed";

/// Sync — the sync process started.
/// A1: device, durable.
pub const SYNC_STARTED: &str = "sync.started";

/// Sync — the sync process stopped (clean shutdown or error).
/// A1: device, durable.
pub const SYNC_STOPPED: &str = "sync.stopped";

/// `ALL` — the canonical set of known event type strings.
///
/// **Deprecated**: new callers should use `ship_types::event_registry::all_names()`
/// or `ship_types::event_registry::registry()` directly. This const is kept as a
/// backward-compat shim for callers that require `&'static [&str]` shape
/// (payload_schema.rs test, substrate tests). After Spec 7b all event-subscription
/// validation goes through `event_registry::all_names()` via `triggers/loader.rs`.
///
/// Note: `event_registry::all_names()` returns the registry-derived set (sorted,
/// deduplicated). This const includes the same names but with legacy aliases that
/// predate the registry (workspace.*, project.changed, etc.), which remain valid
/// event types emitted by deprecated code paths.
pub const ALL: &[&str] = &[
    WORKSPACE_CREATED,
    WORKSPACE_ACTIVATED,
    WORKSPACE_ARCHIVED,
    SESSION_STARTED,
    SESSION_PROGRESS,
    SESSION_ENDED,
    SESSION_DRAIN_STARTED,
    SESSION_DRAIN_COMPLETED,
    SESSION_DRAIN_ABORTED,
    SESSION_TOOL_COUNT_INCREMENTED,
    SESSION_ATTACHED,
    SESSION_HEARTBEAT,
    ACTOR_CREATED,
    ACTOR_WOKE,
    ACTOR_SLEPT,
    ACTOR_STOPPED,
    ACTOR_CRASHED,
    PIPELINE_RUN_STARTED,
    PIPELINE_RUN_COMPLETED,
    PIPELINE_RUN_FAILED,
    PIPELINE_RUN_DISPATCHED,
    PIPELINE_RUN_UPDATED,
    CONFIG_CHANGED,
    CONFIG_FILE_CHANGED,
    SPACE_SETTING_CHANGED,
    AGENT_CHANGED,
    SKILL_CHANGED,
    VIEW_CHANGED,
    PIPELINE_CHANGED,
    RULE_CHANGED,
    PROJECT_CHANGED,
    PROJECT_LOG,
    SKILL_STARTED,
    SKILL_COMPLETED,
    SKILL_FAILED,
    SKILL_ATTACHED_TO_SESSION,
    SUPERVISOR_SHUTTING_DOWN,
    SUPERVISOR_SWEEP_STARTED,
    SUPERVISOR_SWEEP_COMPLETED,
    SUPERVISOR_DRIFT_DETECTED,
    SUPERVISOR_COHERENCE_RESTORED,
    PROCESS_SPAWNED,
    PROCESS_EXITED,
    PROCESS_SIGNAL_SENT,
    PROCESS_ORPHANED,
    PROCESS_KILLED,
    EXECUTOR_SESSION_DESTROYED,
    EXECUTOR_WINDOW_DESTROYED,
    DAEMON_STARTUP_COMPLETED,
    DAEMON_SHUTDOWN_COMPLETED,
    DAEMON_CRASHED,
    DAEMON_PORT_BIND_FAILED,
    DAEMON_PIDFILE_ACQUIRED,
    DAEMON_PIDFILE_STALE_TAKEN,
    PIPELINE_STEP_COMPLETED,
    PIPELINE_STEP_FAILED,
    PIPELINE_RUN_CANCELLED,
    PIPELINE_RUN_MERGED,
    PIPELINE_STEP_SPAWN_FAILED,
    ARTIFACT_CREATED,
    ARTIFACT_UPDATED,
    ARTIFACT_MOVED,
    ARTIFACT_DELETED,
    STUDIO_CHAT_TURN,
    STUDIO_REPLY,
    COMPILE_COMPLETED,
    PROJECT_REGISTERED,
    PROJECT_SEEN,
    PROJECT_ARCHIVED,
    PROJECT_SWITCHED,
    AUTH_SESSION_CREATED,
    AUTH_SESSION_REVOKED,
    AUTH_SESSION_EXPIRED,
    DAEMON_OWNER_BOUND,
    DAEMON_OWNER_UNBOUND,
    ACCOUNT_LOGIN_STARTED,
    ACCOUNT_LOGIN_COMPLETED,
    ACCOUNT_LOGOUT,
    ACCOUNT_KEY_REFRESHED,
    SKILL_CREATED,
    SKILL_UPDATED,
    SKILL_DELETED,
    RULE_CREATED,
    RULE_UPDATED,
    RULE_DELETED,
    PERMISSIONS_UPDATED,
    AGENT_GRANTS_PROVISIONED,
    SESSION_STATUS_CHANGED,
    SESSION_TURN_FAILED,
    // actor
    ACTOR_MAILBOX_FULL,
    // agent
    AGENT_TASK_COMPLETE,
    // human
    HUMAN_RESPONSE,
    // kernel
    KERNEL_ACTOR_RESTORED,
    KERNEL_ACTOR_SNAPSHOT,
    KERNEL_ACTOR_SUSPENDED,
    // mesh
    MESH_BROADCAST,
    MESH_DEREGISTER,
    MESH_DEREGISTERED,
    MESH_DISCOVER_RESPONSE,
    MESH_MESSAGE,
    MESH_REGISTER,
    MESH_REGISTERED,
    MESH_SEND,
    MESH_SEND_FAILED,
    MESH_STATUS,
    // pipeline
    PIPELINE_STEP_RETRY,
    // studio
    STUDIO_ANNOTATION_CREATED,
    STUDIO_FEEDBACK_CREATED,
    STUDIO_MESSAGE,
    // sync
    SYNC_PUSH_FAILED,
    SYNC_STARTED,
    SYNC_STOPPED,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_constants_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for &t in ALL {
            assert!(seen.insert(t), "duplicate event type constant: {t}");
        }
    }
}
