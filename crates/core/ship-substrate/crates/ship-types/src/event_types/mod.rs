pub mod canonical_session;
pub mod daemon;
pub mod mesh;
pub mod pipeline;
pub mod skill;
pub mod supervisor;
pub use mesh::*;

// All lines below are constant definitions; no logic present. The constants are
// NAME REFERENCES only — the vocabulary's source of truth is the inventory
// registry (`event_registry::registry()`, collected at link time from every
// crate's `register_event!` lines). There is deliberately NO master `ALL`
// slice: a hand-maintained mirror of the registry drifts (it did), and under
// keystone A each domain owns its events. Closed family lists live with their
// domain (`canonical_session::ALL_CANONICAL_SESSION` / `ALL_CANONICAL_OP`,
// `ship_collection::events::ALL_COLLECTION`). Full-linkage vocabulary guards
// live in shipd (`tests/event_vocabulary.rs`): a pinned registry count plus a
// live-validator sweep, with NO exception lists.
pub const WORKSPACE_CREATED: &str = "workspace.created";
pub const WORKSPACE_ACTIVATED: &str = "workspace.activated";
pub const WORKSPACE_ARCHIVED: &str = "workspace.archived";
pub const ACTOR_CREATED: &str = "actor.created";
pub const ACTOR_WOKE: &str = "actor.woke";
pub const ACTOR_SLEPT: &str = "actor.slept";
pub const ACTOR_STOPPED: &str = "actor.stopped";
pub const ACTOR_CRASHED: &str = "actor.crashed";
pub const CONFIG_CHANGED: &str = "config.changed";
/// Fine-grained per-file config change (path + content_hash) — see
/// `ConfigFileChanged`. Emitted by the watcher alongside the coarse `*.changed`.
pub const CONFIG_FILE_CHANGED: &str = "config.file_changed";
pub const AGENT_CHANGED: &str = "agent.changed";
/// Fine-grained agent def change notification (Format-2). Emitted per def file
/// when `.ship/agents/<id>.jsonc` changes; carries `{path, content_hash}` (see
/// `AgentDefUpdated`) and rides the Space scope (the def lives in a space's
/// `.ship`). NEVER carries file content — the read-only def index re-reads the
/// file on the ping. The fine-grained, space-stamped successor to the coarse
/// `agent.changed` for the agents domain.
pub const AGENT_DEF_UPDATED: &str = "agent.def.updated";
pub const VIEW_CHANGED: &str = "view.changed";
pub const RULE_CHANGED: &str = "rule.changed";
pub const COLLECTION_CHANGED: &str = "collection.changed";

pub const PROCESS_SPAWNED: &str = "process.spawned";
pub const PROCESS_EXITED: &str = "process.exited";
pub const PROCESS_SIGNAL_SENT: &str = "process.signal_sent";
pub const PROCESS_ORPHANED: &str = "process.orphaned";
pub const PROCESS_KILLED: &str = "process.killed";
pub const EXECUTOR_SESSION_DESTROYED: &str = "executor.session_destroyed";
pub const EXECUTOR_WINDOW_DESTROYED: &str = "executor.window_destroyed";
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

/// Per-space settings override — a single (space, key) value was set. The
/// space scope rides the envelope (`space_id`, stamped via `with_context`),
/// NOT the payload — mirror `CONFIG_FILE_CHANGED`. Layered OVER the global
/// daemon settings (`settings.state`); space overrides win per-key. `entity_id`
/// is the setting key. Payload: `SpaceSettingChanged`.
pub const SPACE_SETTING_CHANGED: &str = "space.setting.changed";

/// Unified scope-discriminated settings change — the keystone settings event.
/// One type across SPACE / ACCOUNT / RUNTIME (`SettingsService.Set` authors it);
/// the chosen `EventScope` rides the envelope and routes to the matching
/// per-scope db. `entity_id` is the setting key. Payload: `SettingChanged`.
pub const SETTINGS_CHANGED: &str = "settings.changed";

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

/// Studio — a user annotation was created on a session or artifact.
/// `entity_id` is the annotation id.
/// A1: space, durable.
pub const STUDIO_ANNOTATION_CREATED: &str = "studio.annotation.created";

/// Studio — a feedback record was created (thumbs-up/down, rating, etc.).
/// `entity_id` is the feedback id.
/// A1: space, durable.
pub const STUDIO_FEEDBACK_CREATED: &str = "studio.feedback.created";

/// Sync — a push to the remote sync target failed.
/// A1: device, durable.
pub const SYNC_PUSH_FAILED: &str = "sync.push.failed";

/// Sync — the sync process started.
/// A1: device, durable.
pub const SYNC_STARTED: &str = "sync.started";

/// Sync — the sync process stopped (clean shutdown or error).
/// A1: device, durable.
pub const SYNC_STOPPED: &str = "sync.stopped";

// Aliases for backward compatibility (moved constants now come via pub use)
// Aliases for backward compatibility (moved constants now come via pub use)
pub const PIPELINE_RUN_STARTED: &str = pipeline::RUN_STARTED;
pub const PIPELINE_RUN_COMPLETED: &str = pipeline::RUN_COMPLETED;
pub const PIPELINE_RUN_FAILED: &str = pipeline::RUN_FAILED;
pub const PIPELINE_RUN_DISPATCHED: &str = pipeline::RUN_DISPATCHED;
pub const PIPELINE_RUN_UPDATED: &str = pipeline::RUN_UPDATED;
pub const PIPELINE_CHANGED: &str = pipeline::CHANGED;
pub const PIPELINE_STEP_COMPLETED: &str = pipeline::STEP_COMPLETED;
pub const PIPELINE_STEP_FAILED: &str = pipeline::STEP_FAILED;
pub const PIPELINE_RUN_CANCELLED: &str = pipeline::RUN_CANCELLED;
pub const PIPELINE_RUN_MERGED: &str = pipeline::RUN_MERGED;
pub const PIPELINE_STEP_SPAWN_FAILED: &str = pipeline::STEP_SPAWN_FAILED;
pub const PIPELINE_STEP_RETRY: &str = pipeline::STEP_RETRY;
pub const SKILL_CHANGED: &str = skill::CHANGED;
pub const SKILL_STARTED: &str = skill::STARTED;
pub const SKILL_COMPLETED: &str = skill::COMPLETED;
pub const SKILL_FAILED: &str = skill::FAILED;
pub const SKILL_CREATED: &str = skill::CREATED;
pub const SKILL_UPDATED: &str = skill::UPDATED;
pub const SKILL_DELETED: &str = skill::DELETED;
pub const DAEMON_STARTUP_COMPLETED: &str = daemon::STARTUP_COMPLETED;
pub const DAEMON_SHUTDOWN_COMPLETED: &str = daemon::SHUTDOWN_COMPLETED;
pub const DAEMON_CRASHED: &str = daemon::CRASHED;
pub const DAEMON_PORT_BIND_FAILED: &str = daemon::PORT_BIND_FAILED;
pub const DAEMON_PIDFILE_ACQUIRED: &str = daemon::PIDFILE_ACQUIRED;
pub const DAEMON_PIDFILE_STALE_TAKEN: &str = daemon::PIDFILE_STALE_TAKEN;
pub const DAEMON_OWNER_BOUND: &str = daemon::OWNER_BOUND;
pub const DAEMON_OWNER_UNBOUND: &str = daemon::OWNER_UNBOUND;
pub const SUPERVISOR_SHUTTING_DOWN: &str = supervisor::SHUTTING_DOWN;
pub const SUPERVISOR_SWEEP_STARTED: &str = supervisor::SWEEP_STARTED;
pub const SUPERVISOR_SWEEP_COMPLETED: &str = supervisor::SWEEP_COMPLETED;
pub const SUPERVISOR_DRIFT_DETECTED: &str = supervisor::DRIFT_DETECTED;
pub const SUPERVISOR_COHERENCE_RESTORED: &str = supervisor::COHERENCE_RESTORED;

// Canonical ship-shaped agent-session family (jobs: canonical-session-events,
// workbench-teardown). 5 coarse-grain lifecycle events (started + 4
// terminals) + 2 medium-grain turn events. The workbench family is retired.
// The family also has a fine-grained `op.*` tier (see
// `canonical_session::ALL_CANONICAL_OP`), registered in the event registry
// but not aliased here and not in `ALL`. See `canonical_session` for docs.
pub const SESSION_STARTED: &str = canonical_session::SESSION_STARTED;
pub const SESSION_COMPLETED: &str = canonical_session::SESSION_COMPLETED;
pub const SESSION_FAILED: &str = canonical_session::SESSION_FAILED;
pub const SESSION_CANCELLED: &str = canonical_session::SESSION_CANCELLED;
pub const SESSION_SUSPENDED: &str = canonical_session::SESSION_SUSPENDED;
pub const TURN_STARTED: &str = canonical_session::TURN_STARTED;
pub const TURN_ENDED: &str = canonical_session::TURN_ENDED;



#[cfg(test)]
mod tests {
    /// Registered event names are unique. (The former const-level uniqueness
    /// test iterated the retired hand-maintained `ALL` mirror; the registry
    /// is the vocabulary SSOT, and at THIS crate's linkage the check also
    /// runs in `event_registry::tests::registry_names_are_unique` — kept
    /// here too so the invariant is asserted next to the name constants.)
    #[test]
    fn registered_event_names_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for def in crate::event_registry::registry() {
            assert!(seen.insert(def.name), "duplicate event registration: {}", def.name);
        }
    }
}
