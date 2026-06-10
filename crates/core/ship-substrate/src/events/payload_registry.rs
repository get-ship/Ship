// FILE_CAP_OVERRIDE: the event registry is a single compilation unit so
// `inventory::collect!` sees every `EventDef`. No logic present; only static
// `EventRegistration` submissions. Split deferred until a second major
// event family lands.

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
//! After this module compiles, `ship_types::event_registry::registry()` returns every
//! classified entry. NOTE: `event_types::ALL` is a separate, HAND-MAINTAINED list — it
//! is NOT auto-derived from this registry. The registry-vs-`ALL` consistency RATCHET
//! lives in `events::tests::registry_vs_all_ratchet`: a registration added here
//! without an `ALL` entry (or a justified-absence row there) fails that test.
//!
//! A1 classification ground truth: `.artifacts/specs/space/event-classification.md`
//! Fail-closed default (undetermined): runtime, durable.


use ship_types::event_registry::{EventDef, EventRegistration, EventScope, Retention, Stability};

// ── Macro-free registration helper ───────────────────────────────────────────────────────────────────
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
    // The final argument is the payload TYPE PATH: the registration carries
    // its own schema validator (Phase 4a — deserialize-into-the-declared-type),
    // so registering an event without a real payload type is impossible. The
    // string form ($type_name) stays as the stable identifier the fixture/
    // codegen tools key on.
    ($name:expr, [$($scope:expr),+ $(,)?], $retention:expr, $stability:expr, $type_name:expr, $payload:ty) => {
        inventory::submit! {
            EventRegistration {
                def: || {
                    fn validate(json: &str) -> Result<(), String> {
                        serde_json::from_str::<$payload>(json)
                            .map(|_| ())
                            .map_err(|e| e.to_string())
                    }
                    static DEF: EventDef = EventDef {
                        name: $name,
                        allowed_scopes: &[$($scope),+],
                        retention: $retention,
                        stability: $stability,
                        since_version: None,
                        payload_type_name: $type_name,
                        validate_payload: validate,
                    };
                    &DEF
                },
            }
        }
    };
}

// ── account.* ───────────────────────────────────────────────────────────────
register_event!("account.key_refreshed", [EventScope::Account], Retention::Ephemeral, Stability::Stable, "AccountKeyRefreshed", crate::events::types::AccountKeyRefreshed);
register_event!("account.login_completed", [EventScope::Account], Retention::Durable, Stability::Stable, "AccountLoginCompleted", crate::events::types::AccountLoginCompleted);
register_event!("account.login_started", [EventScope::Account], Retention::Durable, Stability::Stable, "AccountLoginStarted", crate::events::types::AccountLoginStarted);
register_event!("account.logout", [EventScope::Account], Retention::Durable, Stability::Stable, "AccountLogout", crate::events::types::AccountLogout);

// ── actor.* ─────────────────────────────────────────────────────────────────
register_event!("actor.crashed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ActorCrashed", crate::events::types::ActorCrashed);
register_event!("actor.created", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ActorCreated", crate::events::types::ActorCreated);
register_event!("actor.mailbox_full", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ActorMailboxFull", crate::events::types::ActorMailboxFull);
register_event!("actor.slept", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ActorSlept", crate::events::types::ActorSlept);
register_event!("actor.stopped", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ActorStopped", crate::events::types::ActorStopped);
register_event!("actor.woke", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ActorWoke", crate::events::types::ActorWoke);

// ── agent.* ─────────────────────────────────────────────────────────────────
register_event!("agent.changed", [EventScope::Space], Retention::Ephemeral, Stability::Stable, "AgentChanged", crate::events::types::ConfigChanged);
// Format-2 def change notification — lives in a space's `.ship/agents`, so
// Space-scoped; a Bucket-2 notification (content NOT carried), so Ephemeral.
register_event!("agent.def.updated", [EventScope::Space], Retention::Ephemeral, Stability::Stable, "AgentDefUpdated", crate::events::types::AgentDefUpdated);
register_event!("agent.grants.provisioned", [EventScope::Account], Retention::Durable, Stability::Stable, "AgentGrantsProvisioned", crate::events::types::AgentGrantsProvisioned);
register_event!("agent.task.complete", [EventScope::Space], Retention::Durable, Stability::Stable, "AgentTaskComplete", crate::events::types::AgentTaskComplete);

// ── agent.session.* ─────────────────────────────────────────────────────────
register_event!("agent.session.cancelled", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionCancelledPayload", crate::events::agent_session::AgentSessionCancelledPayload);
register_event!("agent.session.completed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionCompletedPayload", crate::events::agent_session::AgentSessionCompletedPayload);
register_event!("agent.session.dispatched", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionDispatchedPayload", crate::events::agent_session::AgentSessionDispatchedPayload);
register_event!("agent.session.failed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionFailedPayload", crate::events::agent_session::AgentSessionFailedPayload);
register_event!("agent.session.started", [EventScope::Runtime], Retention::Durable, Stability::Stable, "AgentSessionStartedPayload", crate::events::agent_session::AgentSessionStartedPayload);

// ── artifact.* ──────────────────────────────────────────────────────────────
register_event!("artifact.created", [EventScope::Space], Retention::Durable, Stability::Stable, "ArtifactCreated", crate::events::types::ArtifactCreated);
register_event!("artifact.deleted", [EventScope::Space], Retention::Durable, Stability::Stable, "ArtifactDeleted", crate::events::types::ArtifactDeleted);
register_event!("artifact.moved", [EventScope::Space], Retention::Durable, Stability::Stable, "ArtifactMoved", crate::events::types::ArtifactMoved);
register_event!("artifact.updated", [EventScope::Space], Retention::Durable, Stability::Stable, "ArtifactUpdated", crate::events::types::ArtifactUpdated);

// ── auth.session.* ──────────────────────────────────────────────────────────
register_event!("auth.session.created", [EventScope::Account], Retention::Durable, Stability::Stable, "AuthSessionCreated", crate::events::types::AuthSessionCreated);
register_event!("auth.session.expired", [EventScope::Account], Retention::Durable, Stability::Stable, "AuthSessionExpired", crate::events::types::AuthSessionExpired);
register_event!("auth.session.revoked", [EventScope::Account], Retention::Durable, Stability::Stable, "AuthSessionRevoked", crate::events::types::AuthSessionRevoked);

// ── collection.* ────────────────────────────────────────────────────────────
// collection.changed: the coarse `<domain>.changed` watcher event for the
// `.ship/collections/` domain (ship-config `author_change`, ConfigChanged
// payload — same emit path as config/skill/pipeline/rule.changed). Registered
// Runtime/Ephemeral to MATCH those siblings AND the live emit behavior: the
// watcher emits both user-library (space=None) and project scope via
// `daemon_internal(space)`; a Space registration would hard-reject the
// space-unbound user-library emit (`NoSpaceForSpaceEvent`).
register_event!("collection.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ConfigChanged", crate::events::types::ConfigChanged);
register_event!("collection.created", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionCreated", crate::events::types::CollectionCreated);
register_event!("collection.item_created", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionItemCreated", crate::events::types::CollectionItemCreated);
register_event!("collection.item_deleted", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionItemDeleted", crate::events::types::CollectionItemDeleted);
register_event!("collection.item_updated", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionItemUpdated", crate::events::types::CollectionItemUpdated);
register_event!("collection.schema.changed", [EventScope::Space], Retention::Durable, Stability::Stable, "CollectionSchemaChanged", crate::events::types::CollectionSchemaChanged);

// ── compile.* ───────────────────────────────────────────────────────────────
register_event!("compile.completed", [EventScope::Space], Retention::Durable, Stability::Stable, "CompileCompleted", crate::events::types::CompileCompleted);

// ── config.* ────────────────────────────────────────────────────────────────
// Machine/daemon runtime settings (daemon.port, terminal.program, watcher.*,
// isolation tiers) — per-machine, never synced. `set_setting_and_emit` mints
// daemon-internal with no space; Runtime is the correct partition.
register_event!("config.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ConfigChanged", crate::events::types::ConfigChanged);
register_event!("config.file_changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ConfigFileChanged", crate::events::types::ConfigFileChanged);

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
// Poly-scopic (Model C): a human reply may unblock space work (Space), arrive at
// the runtime ingress before its run is correlated (Runtime), or address the
// account (Account). The webhook ingress emits under Runtime; downstream
// correlation can re-emit under the resolved Space.
register_event!("human.response", [EventScope::Space, EventScope::Runtime, EventScope::Account], Retention::Durable, Stability::Stable, "HumanResponse", crate::events::types::HumanResponse);

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
register_event!("permissions.updated", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PermissionsUpdated", crate::events::types::PermissionsUpdated);

// ── pipeline.* ──────────────────────────────────────────────────────────────
register_event!("pipeline.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "PipelineChanged", crate::events::types::ConfigChanged);
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
// The `projects` registry is the daemon's machine-wide record of known project
// paths, read by id/path with no space in hand — and the source the space_id
// resolver itself consults. So the registry events are Runtime-owned (Phase 4d
// §6.1): a space-partitioned registry could not bootstrap space-id resolution,
// and routing them to space.db while reading daemon-wide would force fan-out.
// `project.log` is a daemon-wide audit stream (fixed "project" entity, no
// space) → also Runtime. `project.switched` carries a space binding through
// the kernel and stays Space.
register_event!("project.archived", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProjectArchived", crate::events::types::ProjectArchived);
register_event!("project.log", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ProjectLog", crate::events::types::ProjectLog);
register_event!("project.registered", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProjectRegistered", crate::events::types::ProjectRegistered);
register_event!("project.seen", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ProjectSeen", crate::events::types::ProjectSeen);
register_event!("project.switched", [EventScope::Space], Retention::Durable, Stability::Stable, "ProjectSwitched", crate::events::types::ProjectSwitched);

// ── rule.* ──────────────────────────────────────────────────────────────────
// Runtime audit emit: agent-file CRUD helpers take a ship_dir but no space_id
// and have no read-back projection (Phase 4d §6.3) → daemon-wide runtime.db.
register_event!("rule.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "RuleChanged", crate::events::types::ConfigChanged);
register_event!("rule.created", [EventScope::Runtime], Retention::Durable, Stability::Stable, "RuleCreated", crate::events::types::RuleCreated);
register_event!("rule.deleted", [EventScope::Runtime], Retention::Durable, Stability::Stable, "RuleDeleted", crate::events::types::RuleDeleted);
register_event!("rule.updated", [EventScope::Runtime], Retention::Durable, Stability::Stable, "RuleUpdated", crate::events::types::RuleUpdated);

// ── skill.* ───────────────────────────────────────────────────────────────────
// Runtime audit emit: agent-file CRUD helpers take a ship_dir but no space_id
// and have no read-back projection (Phase 4d §6.3) → daemon-wide runtime.db.
register_event!("skill.changed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "SkillChanged", crate::events::types::ConfigChanged);

register_event!("skill.created", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SkillCreated", crate::events::types::SkillCreated);
register_event!("skill.deleted", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SkillDeleted", crate::events::types::SkillDeleted);
register_event!("skill.updated", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SkillUpdated", crate::events::types::SkillUpdated);

// ── space.setting.* ─────────────────────────────────────────────────────────
register_event!("space.setting.changed", [EventScope::Space], Retention::Durable, Stability::Stable, "SpaceSettingChanged", crate::events::types::SpaceSettingChanged);

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
register_event!("view.changed", [EventScope::Space], Retention::Ephemeral, Stability::Stable, "ViewChanged", crate::events::types::ConfigChanged);

// ── legacy/deprecated (kept for backward compat, fail-closed classification) ─
register_event!("workspace.created", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "WorkspaceCreated", crate::events::types::WorkspaceCreated);
register_event!("workspace.activated", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "WorkspaceActivated", crate::events::types::WorkspaceActivated);
register_event!("workspace.archived", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "WorkspaceArchived", crate::events::types::WorkspaceArchived);
register_event!("daemon.crashed", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "DaemonCrashed", crate::events::daemon::CrashedPayload);
register_event!("daemon.pidfile_acquired", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "DaemonPidfileAcquired", crate::events::daemon::PidfileAcquiredPayload);
register_event!("daemon.pidfile_stale_taken", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "DaemonPidfileStaleTaken", crate::events::daemon::PidfileStaleTakenPayload);
register_event!("pipeline.run.updated", [EventScope::Runtime], Retention::Ephemeral, Stability::Deprecated, "PipelineRunUpdatedPayload", crate::events::pipeline::PipelineRunUpdatedPayload);
register_event!("process.orphaned", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "ProcessOrphaned", crate::events::process::ProcessOrphanedPayload);
register_event!("skill.started", [EventScope::Runtime], Retention::Ephemeral, Stability::Deprecated, "SkillStarted", crate::events::types::SkillStarted);
register_event!("skill.completed", [EventScope::Runtime], Retention::Ephemeral, Stability::Deprecated, "SkillCompleted", crate::events::types::SkillCompleted);
register_event!("skill.failed", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "SkillFailed", crate::events::types::SkillFailed);
