// FILE_CAP_OVERRIDE: 123-event registry — all registrations must be co-located in one
// compilation unit so inventory::collect! sees them. No logic present; only static
// EventRegistration submissions. Split deferred until a second major event family lands.

//! Compile-time event registry population.
//!
//! Each `inventory::submit!` call below registers one `EventDef` in the global
//! `EVENT_REGISTRY`, collected by `ship_types::event_registry::registry()`.
//!
//! Two sources are registered here:
//! 1. Payload structs from `ship_types::payload` (the canonical typed payloads).
//! 2. Payload structs from `ship_substrate::events::*` sub-modules (hooks, agent_session,
//!    pipeline, process, supervisor, executor) which depend on `ship-pipelines` types
//!    and therefore cannot live in the leaf `ship-types` crate.
//!
//! After this module compiles, `ship_types::event_registry::registry()` returns all
//! 123 classified entries. `event_types::ALL` is derived from this registry.
//!
//! A1 classification ground truth: `.artifacts/specs/space/event-classification.md`
//! Fail-closed default (undetermined): runtime, durable.

use ship_types::event_registry::{EventDef, EventRegistration, EventScope, Retention, Stability};

// ── Macro-free registration helper ──────────────────────────────────────────
// We don't use #[derive(EventPayload)] here because the generated code references
// `::ship_types::event_registry::*` via absolute path, which works fine in ship-substrate
// (a downstream crate). We use inventory::submit! directly for clarity and to avoid
// adding the proc-macro as a compile-time dependency on every struct.

/// Register one event type with its **allowed-set** of scopes (Model C).
///
/// The scope argument is a square-bracketed list of one-or-more `EventScope`s.
/// A size-1 set (the common case) is the single-scope model: the lone element
/// is both the primary (stamped at `EventEnvelope::new`) and the only scope the
/// emission may ride. Poly-scopic events list every scope they may ride; the
/// emission's binding picks one and `enforce_scope` validates membership +
/// authorization.
macro_rules! register_event {
    // The scopes are passed as a bracketed, comma-separated list of one-or-more
    // `EventScope`s (the allowed-SET, Model C). Captured element-by-element so
    // the array is rebuilt inside the macro — `&[ ... ]` then unsizes to the
    // `&'static [EventScope]` field. A size-1 set is the single-scope model.
    ($name:expr, [$($scope:expr),+ $(,)?], $retention:expr, $stability:expr, $type_name:expr) => {
        inventory::submit! {
            EventRegistration {
                def: || {
                    static DEF: EventDef = EventDef {
                        name: $name,
                        allowed_scopes: &[$($scope),+],
                        retention: $retention,
                        stability: $stability,
                        since_version: None,
                        payload_type_name: $type_name,
                    };
                    &DEF
                },
            }
        }
    };
}

// ── account.* ───────────────────────────────────────────────────────────────
register_event!("account.key_refreshed", [EventScope::Account], Retention::Ephemeral, Stability::Stable, "AccountKeyRefreshed");
register_event!("account.login_completed", [EventScope::Account], Retention::Durable, Stability::Stable, "AccountLoginCompleted");
register_event!("account.login_started", [EventScope::Account], Retention::Durable, Stability::Stable, "AccountLoginStarted");
register_event!("account.logout", [EventScope::Account], Retention::Durable, Stability::Stable, "AccountLogout");

// ── actor.* ─────────────────────────────────────────────────────────────────
register_event!("actor.crashed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ActorCrashed");
register_event!("actor.created", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ActorCreated");
register_event!("actor.mailbox_full", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ActorMailboxFull");
register_event!("actor.slept", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ActorSlept");
register_event!("actor.stopped", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ActorStopped");
register_event!("actor.woke", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ActorWoke");

// ── agent.* ─────────────────────────────────────────────────────────────────
register_event!("agent.changed", [EventScope::Space], Retention::Ephemeral, Stability::Stable, "AgentChanged");
// Format-2 def change notification — lives in a space's `.ship/agents`, so
// Space-scoped; a Bucket-2 notification (content NOT carried), so Ephemeral.
register_event!("agent.def.updated", [EventScope::Space], Retention::Ephemeral, Stability::Stable, "AgentDefUpdated");
register_event!("agent.grants.provisioned", [EventScope::Account], Retention::Durable, Stability::Stable, "AgentGrantsProvisioned");
register_event!("agent.task.complete", [EventScope::Space], Retention::Durable, Stability::Stable, "AgentTaskComplete");

// ── agent.session.* ─────────────────────────────────────────────────────────
register_event!("agent.session.cancelled", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionCancelledPayload");
register_event!("agent.session.completed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionCompletedPayload");
register_event!("agent.session.dispatched", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionDispatchedPayload");
register_event!("agent.session.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionFailedPayload");
register_event!("agent.session.started", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionStartedPayload");

// ── artifact.* ──────────────────────────────────────────────────────────────
register_event!("artifact.created", [EventScope::Space], Retention::Durable, Stability::Stable, "ArtifactCreated");
register_event!("artifact.deleted", [EventScope::Space], Retention::Durable, Stability::Stable, "ArtifactDeleted");
register_event!("artifact.moved", [EventScope::Space], Retention::Durable, Stability::Stable, "ArtifactMoved");
register_event!("artifact.updated", [EventScope::Space], Retention::Durable, Stability::Stable, "ArtifactUpdated");

// ── auth.session.* ──────────────────────────────────────────────────────────
register_event!("auth.session.created", [EventScope::Account], Retention::Durable, Stability::Stable, "AuthSessionCreated");
register_event!("auth.session.expired", [EventScope::Account], Retention::Durable, Stability::Stable, "AuthSessionExpired");
register_event!("auth.session.revoked", [EventScope::Account], Retention::Durable, Stability::Stable, "AuthSessionRevoked");

// ── collection.* ────────────────────────────────────────────────────────────
register_event!("collection.created", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionCreated");
register_event!("collection.item_created", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionItemCreated");
register_event!("collection.item_deleted", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionItemDeleted");
register_event!("collection.item_updated", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionItemUpdated");
register_event!("collection.schema.changed", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionSchemaChanged");

// ── compile.* ───────────────────────────────────────────────────────────────
register_event!("compile.completed", [EventScope::Space], Retention::Durable, Stability::Stable, "CompileCompleted");

// ── config.* ────────────────────────────────────────────────────────────────
// Machine/daemon runtime settings (daemon.port, terminal.program, watcher.*,
// isolation tiers) — per-machine, never synced. `set_setting_and_emit` mints
// daemon-internal with no space; Runtime is the correct partition.
register_event!("config.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ConfigChanged");
register_event!("config.file_changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ConfigFileChanged");

// ── settings.* ──────────────────────────────────────────────────────────────
// The unified scope-discriminated settings keystone (SettingsService.Set).
// Poly-scopic (Model C): the chosen scope at emit picks the partition —
// Runtime (machine launch defaults, daemon_settings), Account (account.db),
// or Space (the active space's space.db overlay). The binding authorizes the
// chosen scope and the kernel routes to the matching per-scope db.
register_event!("settings.changed", [EventScope::Runtime, EventScope::Account, EventScope::Space], Retention::Durable, Stability::Stable, "SettingChanged");

// ── daemon.* ────────────────────────────────────────────────────────────────
// Daemon/platform-lifecycle events: per-machine execution state, never synced.
// Runtime owns them (folds the former System partition).
register_event!("daemon.port_bind_failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "DaemonPortBindFailed");
register_event!("daemon.shutdown_completed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "DaemonShutdownCompleted");
register_event!("daemon.startup_completed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "DaemonStartupCompleted");

// ── daemon.owner.* ──────────────────────────────────────────────────────────
register_event!("daemon.owner.bound", [EventScope::Account], Retention::Durable, Stability::Stable, "DaemonOwnerBound");
register_event!("daemon.owner.unbound", [EventScope::Account], Retention::Durable, Stability::Stable, "DaemonOwnerUnbound");

// ── executor.* ──────────────────────────────────────────────────────────────
register_event!("executor.session_destroyed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ExecutorSessionDestroyedPayload");
register_event!("executor.window_destroyed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ExecutorWindowDestroyedPayload");

// ── hook.* ──────────────────────────────────────────────────────────────────
register_event!("hook.permission_denied", [EventScope::Runtime], Retention::Durable, Stability::Stable, "HookPermissionDeniedPayload");
register_event!("hook.session_ended", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "HookSessionEndedPayload");
register_event!("hook.session_started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "HookSessionStartedPayload");
register_event!("hook.tool_use_completed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "HookToolUseCompletedPayload");
register_event!("hook.tool_use_started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "HookToolUseStartedPayload");
register_event!("hook.turn_observed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "HookTurnObservedPayload");

// ── human.* ─────────────────────────────────────────────────────────────────
// Poly-scopic (Model C): a human reply may unblock space work (Space), arrive at
// the runtime ingress before its run is correlated (Runtime), or address the
// account (Account). The webhook ingress emits under Runtime; downstream
// correlation can re-emit under the resolved Space.
register_event!("human.response", [EventScope::Space, EventScope::Runtime, EventScope::Account], Retention::Durable, Stability::Stable, "HumanResponse");

// ── kernel.actor.* ──────────────────────────────────────────────────────────
register_event!("kernel.actor.restored", [EventScope::Runtime], Retention::Durable, Stability::Stable, "KernelActorRestored");
register_event!("kernel.actor.snapshot", [EventScope::Runtime], Retention::Durable, Stability::Stable, "KernelActorSnapshot");
register_event!("kernel.actor.suspended", [EventScope::Runtime], Retention::Durable, Stability::Stable, "KernelActorSuspended");

// ── mesh.* ──────────────────────────────────────────────────────────────────
register_event!("mesh.broadcast", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshBroadcast");
register_event!("mesh.deregister", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshDeregister");
register_event!("mesh.deregistered", [EventScope::Runtime], Retention::Durable, Stability::Stable, "MeshDeregistered");
register_event!("mesh.discover.response", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshDiscoverResponse");
register_event!("mesh.message", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshMessage");
register_event!("mesh.register", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshRegister");
register_event!("mesh.registered", [EventScope::Runtime], Retention::Durable, Stability::Stable, "MeshRegistered");
register_event!("mesh.send", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshSend");
register_event!("mesh.send.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "MeshSendFailed");
register_event!("mesh.status", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshStatus");

// ── permissions.* ───────────────────────────────────────────────────────────
// Runtime audit emit: the agent-file CRUD helper takes a ship_dir but no
// space_id and has no read-back projection (Phase 4d §6.3 resolution) → the
// daemon-wide runtime.db, where the off-kernel write lands.
register_event!("permissions.updated", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PermissionsUpdated");

// ── pipeline.* ──────────────────────────────────────────────────────────────
register_event!("pipeline.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "PipelineChanged");
register_event!("pipeline.gate.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineGateFailedPayload");
register_event!("pipeline.gate.passed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineGatePassedPayload");
register_event!("pipeline.gate.requested", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineGateRequestedPayload");
register_event!("pipeline.run.blocked", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunBlockedPayload");
register_event!("pipeline.run.cancelled", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunCancelledPayload");
register_event!("pipeline.run.completed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunCompletedPayload");
register_event!("pipeline.run.dispatched", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunDispatchedPayload");
register_event!("pipeline.run.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunFailedPayload");
register_event!("pipeline.run.merged", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunMergedPayload");
register_event!("pipeline.run.started", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunStartedPayload");
register_event!("pipeline.step.completed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepCompletedPayload");
register_event!("pipeline.step.done_declared", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepDoneDeclaredPayload");
register_event!("pipeline.step.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepFailedPayload");
register_event!("pipeline.step.observed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "PipelineStepObservedPayload");
register_event!("pipeline.step.output", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "PipelineStepOutputPayload");
register_event!("pipeline.step.retry", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepRetry");
register_event!("pipeline.step.spawn_failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepSpawnFailedPayload");
register_event!("pipeline.step.started", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepStartedPayload");

// ── process.* ───────────────────────────────────────────────────────────────
register_event!("process.exited", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProcessExitedPayload");
register_event!("process.killed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProcessKilledPayload");
register_event!("process.signal_sent", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ProcessSignalSentPayload");
register_event!("process.spawned", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ProcessSpawnedPayload");

// ── project.* ───────────────────────────────────────────────────────────────
// The `projects` registry is the daemon's machine-wide record of known project
// paths, read by id/path with no space in hand — and the source the space_id
// resolver itself consults. So the registry events are Runtime-owned (Phase 4d
// §6.1): a space-partitioned registry could not bootstrap space-id resolution,
// and routing them to space.db while reading daemon-wide would force fan-out.
// `project.log` is a daemon-wide audit stream (fixed "project" entity, no
// space) → also Runtime. `project.switched` carries a space binding through
// the kernel and stays Space.
register_event!("project.archived", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProjectArchived");
register_event!("project.log", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ProjectLog");
register_event!("project.registered", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProjectRegistered");
register_event!("project.seen", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ProjectSeen");
register_event!("project.switched", [EventScope::Space], Retention::Durable, Stability::Stable, "ProjectSwitched");

// ── rule.* ──────────────────────────────────────────────────────────────────
// Runtime audit emit: agent-file CRUD helpers take a ship_dir but no space_id
// and have no read-back projection (Phase 4d §6.3) → daemon-wide runtime.db.
register_event!("rule.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "RuleChanged");
register_event!("rule.created", [EventScope::Runtime], Retention::Durable, Stability::Stable, "RuleCreated");
register_event!("rule.deleted", [EventScope::Runtime], Retention::Durable, Stability::Stable, "RuleDeleted");
register_event!("rule.updated", [EventScope::Runtime], Retention::Durable, Stability::Stable, "RuleUpdated");

// ── canonical session.* / session.turn.* (job: canonical-session-events) ─────
// The CANONICAL agent-session family (see `events::canonical_session`). Adds 6
// new event_type names that didn't exist before: session.completed, session.failed,
// session.cancelled, session.suspended, session.turn.started, session.turn.ended.
// `session.started` is intentionally NOT re-scoped — the workbench family owns
// it (Runtime/Durable); the canonical family adds the post-start lifecycle +
// turn events. Scope rule: session.* (coarse) → Space/Durable, session.turn.*
// (medium) → Runtime/Ephemeral.
register_event!("session.completed", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionCompleted");
register_event!("session.failed", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionFailed");
register_event!("session.cancelled", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionCancelled");
register_event!("session.suspended", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionSuspended");
register_event!("session.turn.started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "canonical_session::TurnStarted");
register_event!("session.turn.ended", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "canonical_session::TurnEnded");

// ── session.* (workbench — preserved) ─────────────────────────────────────────
// The workbench `session.*` consts in `ship_types::event_types::session` stay
// untouched. The canonical family adds the new names; the workbench `session.started`
// remains the active session-start emit (the substrate's `session_events` module
// depends on it at Runtime scope).
register_event!("session.attached", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SessionAttached");
register_event!("session.drain_aborted", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SessionDrainAborted");
register_event!("session.drain_completed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SessionDrainCompleted");
register_event!("session.drain_started", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SessionDrainStarted");
register_event!("session.ended", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SessionEnded");
register_event!("session.heartbeat", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SessionHeartbeat");
register_event!("session.progress", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SessionProgress");
register_event!("session.started", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SessionStarted");
register_event!("session.status_changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SessionStatusChanged");
register_event!("session.tool_count_incremented", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SessionToolCountIncremented");
register_event!("session.turn_failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SessionTurnFailed");

// ── skill.* ─────────────────────────────────────────────────────────────────
// Runtime audit emit: agent-file CRUD helpers take a ship_dir but no space_id
// and have no read-back projection (Phase 4d §6.3) → daemon-wide runtime.db.
register_event!("skill.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SkillChanged");
register_event!("skill.created", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SkillCreated");
register_event!("skill.deleted", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SkillDeleted");
register_event!("skill.updated", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SkillUpdated");

// ── space.setting.* ─────────────────────────────────────────────────────────
register_event!("space.setting.changed", [EventScope::Space], Retention::Durable, Stability::Stable, "SpaceSettingChanged");

// ── studio.* ────────────────────────────────────────────────────────────────
register_event!("studio.annotation.created", [EventScope::Space], Retention::Durable, Stability::Stable, "StudioAnnotationCreated");
register_event!("studio.chat_turn", [EventScope::Space], Retention::Durable, Stability::Stable, "StudioChatTurn");
register_event!("studio.feedback.created", [EventScope::Space], Retention::Durable, Stability::Stable, "StudioFeedbackCreated");
register_event!("studio.reply", [EventScope::Space], Retention::Durable, Stability::Stable, "StudioReply");

// ── supervisor.* ────────────────────────────────────────────────────────────
register_event!("supervisor.coherence_restored", [EventScope::Runtime], Retention::Durable, Stability::Stable, "CoherenceRestoredPayload");
register_event!("supervisor.drift_detected", [EventScope::Runtime], Retention::Durable, Stability::Stable, "DriftDetectedPayload");
register_event!("supervisor.shutting_down", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SupervisorShuttingDown");
register_event!("supervisor.sweep_completed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SweepCompletedPayload");
register_event!("supervisor.sweep_started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SweepStartedPayload");

// ── sync.* ──────────────────────────────────────────────────────────────────
register_event!("sync.push.completed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SyncPushCompleted");
register_event!("sync.push.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SyncPushFailed");
register_event!("sync.started", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SyncStarted");
register_event!("sync.stopped", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SyncStopped");

// ── view.* ──────────────────────────────────────────────────────────────────
register_event!("view.changed", [EventScope::Space], Retention::Ephemeral, Stability::Stable, "ViewChanged");

// ── legacy/deprecated (kept for backward compat, fail-closed classification) ─
register_event!("workspace.created", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "WorkspaceCreated");
register_event!("workspace.activated", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "WorkspaceActivated");
register_event!("workspace.archived", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "WorkspaceArchived");
register_event!("daemon.crashed", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "DaemonCrashed");
register_event!("daemon.pidfile_acquired", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "DaemonPidfileAcquired");
register_event!("daemon.pidfile_stale_taken", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "DaemonPidfileStaleTaken");
register_event!("pipeline.run.updated", [EventScope::Runtime], Retention::Ephemeral, Stability::Deprecated, "PipelineRunUpdatedPayload");
register_event!("process.orphaned", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "ProcessOrphaned");
register_event!("skill.started", [EventScope::Runtime], Retention::Ephemeral, Stability::Deprecated, "SkillStarted");
register_event!("skill.completed", [EventScope::Runtime], Retention::Ephemeral, Stability::Deprecated, "SkillCompleted");
register_event!("skill.failed", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "SkillFailed");
