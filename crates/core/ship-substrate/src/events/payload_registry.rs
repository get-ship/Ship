// FILE_CAP_OVERRIDE: No logic present; only static `EventRegistration`
// submissions. This module registers the substrate-owned entries. Domain-owned
// families (session.*, session.turn.*, op.*) are registered from agent-def's
// event_registrations.rs; `registry()` returns every classified entry only
// once all registering crates are linked into the binary.

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
//! After all registering crates link, `ship_types::event_registry::registry()` returns
//! every classified entry. NOTE: `event_types::ALL` is a separate, HAND-MAINTAINED list — it
//! is NOT auto-derived from this registry. The registry-vs-`ALL` consistency RATCHET
//! lives in `events::tests::registry_vs_all_ratchet`: a registration added here
//! without an `ALL` entry (or a justified-absence row there) fails that test.
//!
//! A1 classification ground truth: `.artifacts/specs/space/event-classification.md`
//! Fail-closed default (undetermined): runtime, durable.


use ship_types::event_registry::{EventScope, Retention, Stability};

// Use the hoisted register_event! macro from ship-types (event_registry module).
// The macro is #[macro_export] from ship-types; importing it here brings it
// into scope for all the register_event! calls below.
use ship_types::register_event;

// ── account.* ───────────────────────────────────────────────────────────────
// Moved to ship-auth/src/event_registrations.rs (keystone-A Phase A bundle 2,
// 2026-06-10): account.key_refreshed / account.login_completed /
// account.login_started / account.logout — the AUTH domain owns its events at L1.

// ── actor.* ─────────────────────────────────────────────────────────────────
register_event!("actor.crashed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ActorCrashed", crate::events::types::ActorCrashed);
register_event!("actor.created", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ActorCreated", crate::events::types::ActorCreated);
register_event!("actor.mailbox_full", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ActorMailboxFull", crate::events::types::ActorMailboxFull);
register_event!("actor.slept", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ActorSlept", crate::events::types::ActorSlept);
register_event!("actor.stopped", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ActorStopped", crate::events::types::ActorStopped);
register_event!("actor.woke", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ActorWoke", crate::events::types::ActorWoke);

// ── agent.* ─────────────────────────────────────────────────────────────────
// agent.changed — moved to ship-config/src/event_registrations.rs (keystone-A
// Phase A bundle 3, 2026-06-10): watcher-ping family co-locates with its emitter.
// Format-2 def change notification — lives in a space's `.ship/agents`, so
// Space-scoped; a Bucket-2 notification (content NOT carried), so Ephemeral.
// agent.grants.provisioned — moved to ship-auth/src/event_registrations.rs
// (keystone-A Phase A bundle 2, 2026-06-10): AUTH domain owns its events at L1.

// ── agent.session.* ─────────────────────────────────────────────────────────
register_event!("agent.session.cancelled", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionCancelledPayload", crate::events::agent_session::AgentSessionCancelledPayload);
register_event!("agent.session.completed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionCompletedPayload", crate::events::agent_session::AgentSessionCompletedPayload);
register_event!("agent.session.dispatched", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionDispatchedPayload", crate::events::agent_session::AgentSessionDispatchedPayload);
register_event!("agent.session.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionFailedPayload", crate::events::agent_session::AgentSessionFailedPayload);
register_event!("agent.session.started", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionStartedPayload", crate::events::agent_session::AgentSessionStartedPayload);

// ── artifact.* ──────────────────────────────────────────────────────────────
// Moved to ship-artifacts/src/event_registrations.rs (keystone-A de-bucket,
// 2026-06-10): the ARTIFACT domain owns its events at L1.

// ── auth.session.* ──────────────────────────────────────────────────────────
// Moved to ship-auth/src/event_registrations.rs (keystone-A Phase A bundle 2,
// 2026-06-10): auth.session.created / auth.session.expired / auth.session.revoked
// — the AUTH domain owns its events at L1.

// ── collection.* ────────────────────────────────────────────────────────────
// collection.changed: moved to ship-config/src/event_registrations.rs (keystone-A
// Phase A bundle 3, 2026-06-10) — it is a watcher-ping event emitted by
// ship-config's watcher, co-locating with the watcher family.
// collection.created / collection.item_* / collection.schema.changed — the 5
// CRUD events are now registered at L1 by `ship-collection`'s own
// `event_registrations` module (keystone-A de-bucket: collection domain owns
// its events at L1). They are no longer registered here.

// ── compile.* ───────────────────────────────────────────────────────────────

// ── config.* ────────────────────────────────────────────────────────────────
// Moved to ship-config/src/event_registrations.rs (keystone-A Phase A bundle 3,
// 2026-06-10): config.changed / config.file_changed — the CONFIG watcher owns
// these events at L1 alongside the rest of the watcher-ping family.

// ── settings.* ──────────────────────────────────────────────────────────────
// The unified scope-discriminated settings keystone (SettingsService.Set).
// Poly-scopic (Model C): the chosen scope at emit picks the partition —
// Runtime (machine launch defaults, daemon_settings), Account (account.db),
// or Space (the active space's space.db overlay). The binding authorizes the
// chosen scope and the kernel routes to the matching per-scope db.
register_event!("settings.changed", [EventScope::Runtime, EventScope::Account, EventScope::Space], Retention::Durable, Stability::Stable, "SettingChanged", crate::events::types::SettingChanged);

// ── daemon.* ────────────────────────────────────────────────────────────────
// Daemon/platform-lifecycle events: per-machine execution state, never synced.
// Runtime owns them (folds the former System partition).
register_event!("daemon.port_bind_failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "DaemonPortBindFailed", crate::events::daemon::PortBindFailedPayload);
register_event!("daemon.shutdown_completed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "DaemonShutdownCompleted", crate::events::daemon::ShutdownCompletedPayload);
register_event!("daemon.startup_completed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "DaemonStartupCompleted", crate::events::daemon::StartupCompletedPayload);

// ── daemon.owner.* ──────────────────────────────────────────────────────────
register_event!("daemon.owner.bound", [EventScope::Account], Retention::Durable, Stability::Stable, "DaemonOwnerBound", crate::events::types::DaemonOwnerBound);
register_event!("daemon.owner.unbound", [EventScope::Account], Retention::Durable, Stability::Stable, "DaemonOwnerUnbound", crate::events::types::DaemonOwnerUnbound);

// ── executor.* ──────────────────────────────────────────────────────────────
register_event!("executor.session_destroyed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ExecutorSessionDestroyedPayload", crate::events::executor::ExecutorSessionDestroyedPayload);
register_event!("executor.window_destroyed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ExecutorWindowDestroyedPayload", crate::events::executor::ExecutorWindowDestroyedPayload);

// ── hook.* ──────────────────────────────────────────────────────────────────
// hook.permission_denied: downgraded Durable → Ephemeral (R2 hook conversion).
// The denied-permission signal is a point-in-time runtime gate decision; there
// is no projection that folds it and it must not survive compaction.
//
// op.* family: the op.tool_use_started / op.tool_use_completed /
// op.tool_use_denied events carry the same ephemeral semantics as
// hook.tool_use_* and are NOW REGISTERED (Runtime/Ephemeral) at L1, in
// `agent-def/src/event_registrations.rs` (job: op-canon) — NOT here, because
// their payload structs (`op::OpToolUse*`) live in agent-def. The remaining
// follow-up is the hook→op CONVERSION (re-point emitters + final hook.*
// deregistration), tracked separately (#25b); it is not a registration blocker.
register_event!("hook.permission_denied", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "HookPermissionDeniedPayload", crate::events::hooks::HookPermissionDeniedPayload);
register_event!("hook.session_ended", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "HookSessionEndedPayload", crate::events::hooks::HookSessionEndedPayload);
register_event!("hook.session_started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "HookSessionStartedPayload", crate::events::hooks::HookSessionStartedPayload);
register_event!("hook.tool_use_completed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "HookToolUseCompletedPayload", crate::events::hooks::HookToolUseCompletedPayload);
register_event!("hook.tool_use_started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "HookToolUseStartedPayload", crate::events::hooks::HookToolUseStartedPayload);
register_event!("hook.turn_observed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "HookTurnObservedPayload", crate::events::hooks::HookTurnObservedPayload);

// ── human.* ─────────────────────────────────────────────────────────────────
// Moved to ship-messaging/src/event_registrations.rs (keystone-A Phase A bundle 4b,
// 2026-06-10): human.response — the MESSAGING domain (human-gateway) owns it at L1.

// ── kernel.actor.* ──────────────────────────────────────────────────────────
register_event!("kernel.actor.restored", [EventScope::Runtime], Retention::Durable, Stability::Stable, "KernelActorRestored", crate::events::types::KernelActorRestored);
register_event!("kernel.actor.snapshot", [EventScope::Runtime], Retention::Durable, Stability::Stable, "KernelActorSnapshot", crate::events::types::KernelActorSnapshot);
register_event!("kernel.actor.suspended", [EventScope::Runtime], Retention::Durable, Stability::Stable, "KernelActorSuspended", crate::events::types::KernelActorSuspended);

// ── mesh.* ──────────────────────────────────────────────────────────────────
register_event!("mesh.broadcast", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshBroadcast", crate::events::types::MeshBroadcast);
register_event!("mesh.deregister", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshDeregister", crate::events::types::MeshDeregister);
register_event!("mesh.deregistered", [EventScope::Runtime], Retention::Durable, Stability::Stable, "MeshDeregistered", crate::events::types::MeshDeregistered);
register_event!("mesh.discover.response", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshDiscoverResponse", crate::events::types::MeshDiscoverResponse);
register_event!("mesh.message", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshMessage", crate::events::types::MeshMessage);
register_event!("mesh.register", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshRegister", crate::events::types::MeshRegister);
register_event!("mesh.registered", [EventScope::Runtime], Retention::Durable, Stability::Stable, "MeshRegistered", crate::events::types::MeshRegistered);
register_event!("mesh.send", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshSend", crate::events::types::MeshSend);
register_event!("mesh.send.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "MeshSendFailed", crate::events::types::MeshSendFailed);
register_event!("mesh.status", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "MeshStatus", crate::events::types::MeshStatus);

// ── permissions.* ───────────────────────────────────────────────────────────
// Runtime audit emit: the agent-file CRUD helper takes a ship_dir but no
// space_id and has no read-back projection (Phase 4d §6.3 resolution) → the
// daemon-wide runtime.db, where the off-kernel write lands.

// ── pipeline.* ──────────────────────────────────────────────────────────────
// pipeline.changed — moved to ship-config/src/event_registrations.rs (keystone-A
// Phase A bundle 3, 2026-06-10): watcher-ping family co-locates with its emitter.
register_event!("pipeline.gate.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineGateFailedPayload", crate::events::pipeline::PipelineGateFailedPayload);
register_event!("pipeline.gate.passed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineGatePassedPayload", crate::events::pipeline::PipelineGatePassedPayload);
register_event!("pipeline.gate.requested", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineGateRequestedPayload", crate::events::pipeline::PipelineGateRequestedPayload);
register_event!("pipeline.run.blocked", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunBlockedPayload", crate::events::pipeline::PipelineRunBlockedPayload);
register_event!("pipeline.run.cancelled", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunCancelledPayload", crate::events::pipeline::PipelineRunCancelledPayload);
register_event!("pipeline.run.completed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunCompletedPayload", crate::events::pipeline::PipelineRunCompletedPayload);
register_event!("pipeline.run.dispatched", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunDispatchedPayload", crate::events::pipeline::PipelineRunDispatchedPayload);
register_event!("pipeline.run.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunFailedPayload", crate::events::pipeline::PipelineRunFailedPayload);
register_event!("pipeline.run.merged", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunMergedPayload", crate::events::pipeline::PipelineRunMergedPayload);
register_event!("pipeline.run.started", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunStartedPayload", crate::events::pipeline::PipelineRunStartedPayload);
register_event!("pipeline.step.completed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepCompletedPayload", crate::events::pipeline::PipelineStepCompletedPayload);
register_event!("pipeline.step.done_declared", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepDoneDeclaredPayload", crate::events::pipeline::PipelineStepDoneDeclaredPayload);
register_event!("pipeline.step.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepFailedPayload", crate::events::pipeline::PipelineStepFailedPayload);
register_event!("pipeline.step.observed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "PipelineStepObservedPayload", crate::events::pipeline::PipelineStepObservedPayload);
register_event!("pipeline.step.output", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "PipelineStepOutputPayload", crate::events::pipeline::PipelineStepOutputPayload);
register_event!("pipeline.step.retry", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepRetry", crate::events::types::PipelineStepRetry);
register_event!("pipeline.step.spawn_failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepSpawnFailedPayload", crate::events::pipeline::PipelineStepSpawnFailedPayload);
register_event!("pipeline.step.started", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepStartedPayload", crate::events::pipeline::PipelineStepStartedPayload);

// ── process.* ───────────────────────────────────────────────────────────────
register_event!("process.exited", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProcessExitedPayload", crate::events::process::ProcessExitedPayload);
register_event!("process.killed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProcessKilledPayload", crate::events::process::ProcessKilledPayload);
register_event!("process.signal_sent", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ProcessSignalSentPayload", crate::events::process::ProcessSignalSentPayload);
register_event!("process.spawned", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ProcessSpawnedPayload", crate::events::process::ProcessSpawnedPayload);

// ── project.* ───────────────────────────────────────────────────────────────
// Moved to ship-space/src/event_registrations.rs (keystone-A Phase A bundle 4,
// 2026-06-10): project.archived / project.log / project.registered /
// project.seen / project.switched — the SPACE domain owns its events at L1.

// ── rule.* / skill.* watcher pings ──────────────────────────────────────────
// rule.changed / skill.changed — moved to ship-config/src/event_registrations.rs
// (keystone-A Phase A bundle 3, 2026-06-10): watcher-ping family co-locates with
// its emitter in ship-config. The CRUD events (rule.created etc.) stay in
// agent-def (bundle 1, 2026-06-10).


// ── space.setting.* ─────────────────────────────────────────────────────────
// Moved to ship-space/src/event_registrations.rs (keystone-A Phase A bundle 4,
// 2026-06-10): space.setting.changed — the SPACE domain owns it at L1.

// ── studio.* ────────────────────────────────────────────────────────────────
register_event!("studio.annotation.created", [EventScope::Space], Retention::Durable, Stability::Stable, "StudioAnnotationCreated", crate::events::types::StudioAnnotationCreated);
register_event!("studio.chat_turn", [EventScope::Space], Retention::Durable, Stability::Stable, "StudioChatTurn", crate::events::types::StudioChatTurn);
register_event!("studio.feedback.created", [EventScope::Space], Retention::Durable, Stability::Stable, "StudioFeedbackCreated", crate::events::types::StudioFeedbackCreated);
register_event!("studio.reply", [EventScope::Space], Retention::Durable, Stability::Stable, "StudioReply", crate::events::types::StudioReply);

// ── supervisor.* ────────────────────────────────────────────────────────────
register_event!("supervisor.coherence_restored", [EventScope::Runtime], Retention::Durable, Stability::Stable, "CoherenceRestoredPayload", crate::events::supervisor::CoherenceRestoredPayload);
register_event!("supervisor.drift_detected", [EventScope::Runtime], Retention::Durable, Stability::Stable, "DriftDetectedPayload", crate::events::supervisor::DriftDetectedPayload);
register_event!("supervisor.shutting_down", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SupervisorShuttingDown", crate::events::types::SupervisorShuttingDown);
register_event!("supervisor.sweep_completed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SweepCompletedPayload", crate::events::supervisor::SweepCompletedPayload);
register_event!("supervisor.sweep_started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SweepStartedPayload", crate::events::supervisor::SweepStartedPayload);

// ── sync.* ──────────────────────────────────────────────────────────────────
register_event!("sync.push.completed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SyncPushCompleted", crate::events::types::SyncPushCompleted);
register_event!("sync.push.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SyncPushFailed", crate::events::types::SyncPushFailed);
register_event!("sync.started", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SyncStarted", crate::events::types::SyncStarted);
register_event!("sync.stopped", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SyncStopped", crate::events::types::SyncStopped);

// ── view.* ──────────────────────────────────────────────────────────────────
// view.changed — moved to ship-config/src/event_registrations.rs (keystone-A
// Phase A bundle 3, 2026-06-10): watcher-ping family co-locates with its emitter.

// ── legacy/deprecated (kept for backward compat, fail-closed classification) ─
// workspace.* moved to ship-space/src/event_registrations.rs (keystone-A Phase A
// bundle 4, 2026-06-10) — the SPACE domain owns its deprecated legacy events.
register_event!("daemon.crashed", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "DaemonCrashed", crate::events::daemon::CrashedPayload);
register_event!("daemon.pidfile_acquired", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "DaemonPidfileAcquired", crate::events::daemon::PidfileAcquiredPayload);
register_event!("daemon.pidfile_stale_taken", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "DaemonPidfileStaleTaken", crate::events::daemon::PidfileStaleTakenPayload);
register_event!("pipeline.run.updated", [EventScope::Runtime], Retention::Ephemeral, Stability::Deprecated, "PipelineRunUpdatedPayload", crate::events::pipeline::PipelineRunUpdatedPayload);
register_event!("process.orphaned", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "ProcessOrphaned", crate::events::process::ProcessOrphanedPayload);
