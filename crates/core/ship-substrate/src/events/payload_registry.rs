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
// `actor.created` is the row's BIRTH — the ActorProjection seeds `actors` from it
// on every rebuild. Reclassifying it as Durable is the right call now that the
// kernel's `route` consults `Retention` (job/transient-events spike,
// `.artifacts/transient-events-design.md`): losing it on daemon restart would
// leave the actors table empty. The activity ticks below (`actor.slept`,
// `actor.woke`, `actor.stopped`) keep Ephemeral — they are heartbeat-style
// signals the kernel delivers to live subscribers, not durable state.
register_event!("actor.created", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ActorCreated", crate::events::types::ActorCreated);
// `actor.mailbox_full` retired (B14, 2026-06-20) — the event had no emitter;
// the kernel's `route` is delivery-only and silently drops on mailbox full.
// Backpressure (post-emit signal) is a fast-follow. See `.artifacts/specs/...`.
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
// SEVERED (#26 spawn-intent, 2026-06-10): all 5 agent.session.* events
// deregistered. The dispatch primitive is now `dispatch.requested` (Runtime/
// Ephemeral, registered at L1 in agent-def/src/event_registrations.rs).
// The payload structs (and the whole `events::agent_session` module) are
// DELETED — the claimed "deserialization shim for historical replay" had
// zero call sites. `InvocationMode` moved to agent-def (L1).
// L0 count: 61→56 (-5); full count: 123→119 (-5+1 dispatch.requested).

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
// Reclassified to Durable (job/transient-events): the post-mortem query
// "every session we destroyed in the last 24h" the cleanup slice relies
// on is a durable fact. They are no longer transient observations — the
// kernel's `route` now honors `Retention` and the docs on `Ephemeral`
// ("Windowed; only the last N … retained") no longer fit these events
// whose consumers (the post-mortem projection) want the full history.
register_event!("executor.session_destroyed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ExecutorSessionDestroyedPayload", crate::events::executor::ExecutorSessionDestroyedPayload);
register_event!("executor.window_destroyed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ExecutorWindowDestroyedPayload", crate::events::executor::ExecutorWindowDestroyedPayload);

// ── hook.* ──────────────────────────────────────────────────────────────────
// DEREGISTERED (#25b, 2026-06-10): all 6 hook.* events retired. Emitters
// converted to op.* (tool-use) and session.turn.ended (turns). Session
// lifecycle sourced from actor.created / actor.stopped. The op.* family
// (op.tool_use_started / op.tool_use_completed / op.tool_use_denied) and
// session.turn.ended are registered at L1 in agent-def/src/event_registrations.rs.
// hook.tripped in ship-triggers is an unrelated dynamic trigger event — untouched.

// ── human.* ─────────────────────────────────────────────────────────────────
// Moved to ship-messaging/src/event_registrations.rs (keystone-A Phase A bundle 4b,
// 2026-06-10): human.response — the MESSAGING domain (human-gateway) owns it at L1.

// ── kernel.actor.* ──────────────────────────────────────────────────────────
// B2 (2026-06-20) retired kernel.actor.{snapshot, suspended, restored} along
// with the `KernelRouter::snapshot/suspend/restore` API + `ActorSnapshot`
// type. The shared-log architecture retired per-actor DB files, leaving
// `db_bytes` meaningless in a snapshot round-trip (see
// `persistence/snapshot.rs` STALE-SEMANTICS block). The `kernel.actor.*`
// events were the lifecycle channel; cut together. No replacement — actors
// reload from the shared event log on cold start.

// ── mesh.* ──────────────────────────────────────────────────────────────────
// 7 mesh.* registrations DELETED (residue cull, 2026-06-10): they were the
// vocabulary of the killed MeshService op surface (mesh.broadcast /
// registered / deregistered / send / send.failed / status /
// discover.response — zero live emitters; see shipd/lib.rs "MeshService was
// killed"). The LIVE mesh mechanism is exactly: mesh.register +
// ── permissions.* ───────────────────────────────────────────────────────────
// Runtime audit emit: the agent-file CRUD helper takes a ship_dir but no
// space_id and has no read-back projection (Phase 4d §6.3 resolution) → the
// daemon-wide runtime.db, where the off-kernel write lands.

// ── pipeline.* (engine cut, 2026-07-10) ──────────────────────────────────────
// pipeline.changed lives in ship-config (config-watcher ping). The
// orchestrator's engine-emitted `pipeline.*` events all die with the
// engine (zero consumers, zero emitters). The three `run.started`,
// `run.cancelled`, `step.done_declared` registrations stay — code-mode
// will re-emit them when it lands (the verbs are reusable). The
// `pipeline.step.observed` registration stays for backward-compat
// (the canonical signal in code-mode is `session.{completed,failed,cancelled}`).
register_event!("pipeline.run.cancelled", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunCancelledPayload", crate::events::pipeline::PipelineRunCancelledPayload);
register_event!("pipeline.run.started", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineRunStartedPayload", crate::events::pipeline::PipelineRunStartedPayload);
register_event!("pipeline.step.done_declared", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepDoneDeclaredPayload", crate::events::pipeline::PipelineStepDoneDeclaredPayload);
// Reclassified to Durable (job/transient-events). The pipeline_step_runs
// projection (engine cut, 2026-07-10) no longer anchors on this event
// — the canonical signal in code-mode is the canonical_sessions family
// — but the registration stays for backward-compat (existing post-mortem
// queries key off it; the durable audit trail of "we observed step X at
// time T with reason R" is preserved).
register_event!("pipeline.step.observed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PipelineStepObservedPayload", crate::events::pipeline::PipelineStepObservedPayload);
// Reclassified to Durable (job/transient-events): the `ActorProjection`
// backfills the actor's pid from `process.spawned` (the projection
// code path is documented as "apply the pid from `process.spawned` to
// the matching actor row so live = pid_alive works honestly instead of
// falling back to the activity_live timer"). Losing this on restart
// would mean every actor's pid column is empty on cold boot, and the
// activity_live timer has to re-learn every pid from scratch.
register_event!("process.spawned", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProcessSpawnedPayload", crate::events::process::ProcessSpawnedPayload);
register_event!("process.signal_sent", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "ProcessSignalSentPayload", crate::events::process::ProcessSignalSentPayload);
register_event!("process.exited", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProcessExitedPayload", crate::events::process::ProcessExitedPayload);
register_event!("process.killed", [EventScope::Runtime], Retention::Durable, Stability::Stable, "ProcessKilledPayload", crate::events::process::ProcessKilledPayload);
// ── project.* ───────────────────────────────────────────────────────────────
// Moved to ship-space/src/event_registrations.rs (keystone-A Phase A bundle 4,
// project.seen / project.switched — the SPACE domain owns its events at L1.
// (`project.log` was DELETED 2026-06-19 — dead code, no emitter.)

// ── rule.* / skill.* watcher pings ──────────────────────────────────────────
// rule.changed / skill.changed — moved to ship-config/src/event_registrations.rs
// (keystone-A Phase A bundle 3, 2026-06-10): watcher-ping family co-locates with
// its emitter in ship-config. The CRUD events (rule.created etc.) stay in
// agent-def (bundle 1, 2026-06-10).

// ── settings.changed unified scope keystone ──────────────────────────────────
// The space tier lives at EventScope::Space and rides the same `settings.changed`
// event as ACCOUNT / RUNTIME (registered above in `settings.*`). The legacy
// `space.setting.changed` family was dissolved in S4 — per-space overrides now
// author `settings.changed` (Space scope) from the SettingsService wire.

// ── studio.* ────────────────────────────────────────────────────────────────
// Moved to ship-agents/src/event_registrations.rs (keystone-A de-bucket,
// 2026-06-10): studio.annotation.created / studio.chat_turn /
// studio.feedback.created / studio.reply — the STUDIO domain owns its
// events at L1 in ship-agents (the dominant emitter for 3 of 4 events).

// ── supervisor.* ────────────────────────────────────────────────────────────
// The 4 supervisor.sweep_*/drift/coherence registrations moved to
// ship-pipeline-runtime/src/sweep_events.rs (keystone-A, 2026-06-10) — the
// coherence sweep is that crate's mechanism. supervisor.shutting_down stays:
// it is daemon-lifecycle, emitted at L0/shipd shutdown.
register_event!("supervisor.shutting_down", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SupervisorShuttingDown", crate::events::types::SupervisorShuttingDown);

// ── sync.* ──────────────────────────────────────────────────────────────────
// Moved to ship-cloud/src/event_registrations.rs (substrate-cohesion
// de-bucket, 2026-06-11): sync.push.completed / sync.push.failed /
// sync.started / sync.stopped — the CLOUD/SYNC domain owns its events at L1
// in ship-cloud (sole emitter: SyncServiceHandler). Payload structs remain
// in ship_types::payload. L0 count: 52→48.

// ── view.* ──────────────────────────────────────────────────────────────────
// view.changed — moved to ship-config/src/event_registrations.rs (keystone-A
// Phase A bundle 3, 2026-06-10): watcher-ping family co-locates with its emitter.

// ── legacy/deprecated (kept for backward compat, fail-closed classification) ─
// pipeline.run.updated was retired with the engine (2026-07-10) — the
// `pipeline_step_run` projection that anchored on it is gone; code-mode
// drops the deprecated human-to-step mailbox in favor of the canonical
// `session.*` family.
register_event!("daemon.crashed", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "DaemonCrashed", crate::events::daemon::CrashedPayload);
register_event!("daemon.pidfile_acquired", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "DaemonPidfileAcquired", crate::events::daemon::PidfileAcquiredPayload);
register_event!("daemon.pidfile_stale_taken", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "DaemonPidfileStaleTaken", crate::events::daemon::PidfileStaleTakenPayload);
register_event!("process.orphaned", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "ProcessOrphaned", crate::events::process::ProcessOrphanedPayload);
