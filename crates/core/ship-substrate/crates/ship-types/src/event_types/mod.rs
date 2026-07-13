pub mod canonical_session;
pub mod daemon;
pub mod pipeline;
pub mod skill;
pub mod supervisor;

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

/// Emitted by `ship compile` at end of a successful run. Records what was
/// compiled (active agent, harness, plugins, ship version) for the
/// `project_compile_state` projection. `entity_id` is the space_id.
pub const COMPILE_COMPLETED: &str = "compile.completed";

/// Space location — the runtime opened a directory path as a space. The
/// `spaces` Runtime-scoped projection (folded from this event in
/// `ship-space::spaces_projection`) materializes one row per opened space;
/// `entity_id` is the mint-once space_id (a nanoid, never derived from the
/// path — the space owns its identity, the path is just where it currently
/// lives). Payload: `SpaceOpened`.
pub const SPACE_OPENED: &str = "space.opened";
/// Space location — last-seen bump (a re-open of a known space). The runtime
/// emits this on every boot of a previously-opened space. Payload:
/// `SpaceSeen`.
pub const SPACE_SEEN: &str = "space.seen";
/// Stack membership — a stack root registered as a member of a space
/// (`entity_id` = the space's id). Membership is REGISTRATION: the composed
/// space is exactly the registered set; the runtime never scans directories
/// for members. Folds into the Runtime-scoped `space_stacks` projection.
/// Payload: `StackRegistered`.
pub const STACK_REGISTERED: &str = "stack.registered";

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

/// Unified scope-discriminated settings change — the keystone settings event.
/// One type across SPACE / ACCOUNT / RUNTIME (`SettingsService.Set` authors it);
/// the chosen `EventScope` rides the envelope and routes to the matching
/// per-scope db. `entity_id` is the setting key. Payload: `SettingChanged`.
pub const SETTINGS_CHANGED: &str = "settings.changed";

/// Agent — a dispatched task ran to completion. `entity_id` is the task id.
pub const AGENT_TASK_COMPLETE: &str = "agent.task.complete";


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
