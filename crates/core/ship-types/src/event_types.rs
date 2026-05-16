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
pub const GATE_PASSED: &str = "gate.passed";
pub const GATE_FAILED: &str = "gate.failed";
pub const JOB_CREATED: &str = "job.created";
pub const JOB_COMPLETED: &str = "job.completed";
pub const JOB_FAILED: &str = "job.failed";
pub const JOB_DISPATCHED: &str = "job.dispatched";
pub const JOB_UPDATE: &str = "job.update";
pub const CONFIG_CHANGED: &str = "config.changed";
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
pub const JOB_STEP_COMPLETED: &str = "job.step_completed";
pub const JOB_STEP_FAILED: &str = "job.step_failed";
pub const JOB_CANCELLED: &str = "job.cancelled";
pub const JOB_MERGED: &str = "job.merged";
pub const JOB_SPAWN_FAILED: &str = "job.spawn_failed";
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
/// compiled (active agent, provider, plugins, ship version) for the
/// `project_compile_state` projection. `entity_id` is the project_id.
pub const COMPILE_COMPLETED: &str = "compile.completed";

/// Project registry — first time a project path is seen by the daemon.
/// `entity_id` is the project_id (stable hash of the canonical path).
pub const PROJECT_REGISTERED: &str = "project.registered";
/// Project registry — `last_seen_at` bump (re-open, switch, reattach).
pub const PROJECT_SEEN: &str = "project.seen";
/// Project registry — user removed the project from their registry.
pub const PROJECT_ARCHIVED: &str = "project.archived";
/// Active-project switch. `entity_id` is the newly-active project_id.
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
/// is the project_id (or `"global"` for user-scope writes).
pub const PERMISSIONS_UPDATED: &str = "permissions.updated";

/// Session status transition (e.g. Idle → Thinking). `entity_id` is
/// the session id. Useful for "what's the agent doing right now?" UIs.
pub const SESSION_STATUS_CHANGED: &str = "session.status_changed";
/// A turn failed to spawn or run end-to-end. `entity_id` is the session
/// id. Distinct from `session.ended` — the session stays alive, the
/// individual turn is the unit of failure.
pub const SESSION_TURN_FAILED: &str = "session.turn_failed";

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
    GATE_PASSED,
    GATE_FAILED,
    JOB_CREATED,
    JOB_COMPLETED,
    JOB_FAILED,
    JOB_DISPATCHED,
    JOB_UPDATE,
    CONFIG_CHANGED,
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
    JOB_STEP_COMPLETED,
    JOB_STEP_FAILED,
    JOB_CANCELLED,
    JOB_MERGED,
    JOB_SPAWN_FAILED,
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
    SKILL_CREATED,
    SKILL_UPDATED,
    SKILL_DELETED,
    RULE_CREATED,
    RULE_UPDATED,
    RULE_DELETED,
    PERMISSIONS_UPDATED,
    SESSION_STATUS_CHANGED,
    SESSION_TURN_FAILED,
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
