//! Event registrations for the canonical session family AND the agents-domain
//! facet vocabulary.
//!
//! This module registers the 7 canonical session events + 3 op.* (fine-grain
//! tool-use) + 1 dispatch.* (spawn-intent) + the AGENTS-domain facets
//! (skill.* / rule.* / permissions.updated / compile.completed /
//! agent.task.complete) with the global event registry via the hoisted
//! `register_event!` macro from `ship_types`, making them available to
//! downstream code that calls `event_registry::registry()`.
//!
//! The `agent.def.updated` registration lives in `ship-config` (its emitter).

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

// ── session.* / session.turn.* (canonical — jobs: canonical-session-events +
// workbench-teardown) ──────────────────────────────────────────────────
// The CANONICAL agent-session family (see `crate::canonical_session`).
// The complete ship-shaped session vocabulary: 5 coarse-grain events
// (session.started + 4 terminals) + 2 medium-grain turn events. Scope
// rule: session.* (coarse) → Space/Durable, session.turn.* (medium) →
// Runtime/Ephemeral. The workbench family is RETIRED (R6) — no
// `SessionStarted` workbench payload struct, no `session.attached` etc.
register_event!("session.started", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionStarted", crate::canonical_session::SessionStarted);
register_event!("session.completed", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionCompleted", crate::canonical_session::SessionCompleted);
register_event!("session.failed", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionFailed", crate::canonical_session::SessionFailed);
register_event!("session.cancelled", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionCancelled", crate::canonical_session::SessionCancelled);
register_event!("session.suspended", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionSuspended", crate::canonical_session::SessionSuspended);
register_event!("session.turn.started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "canonical_session::TurnStarted", crate::canonical_session::TurnStarted);
register_event!("session.turn.ended", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "canonical_session::TurnEnded", crate::canonical_session::TurnEnded);

// ── op.* (fine-grained) — Runtime-scoped, Ephemeral ────────────────────────────
// R2: op.* replaces the retired hook.* family (#25b, merged). Emitters:
// http_ingress → op.* conversion; ship-code synth_emit.rs → op.tool_use_*.
// Fine-grained (per tool-call) so Runtime + Ephemeral — same scope rule as
// `session.turn.*`, just one more level of granularity.
register_event!("op.tool_use_started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "op::OpToolUseStarted", crate::op::OpToolUseStarted);
register_event!("op.tool_use_completed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "op::OpToolUseCompleted", crate::op::OpToolUseCompleted);
register_event!("op.tool_use_denied", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "op::OpToolUseDenied", crate::op::OpToolUseDenied);

// ── dispatch.* (spawn-intent channel, #26) ─────────────────────────────────
// Runtime-scoped, Ephemeral. The ONE sanctioned driver-edge carrier. Replaces
// the legacy `agent.session.started` as the hard-failing spawn trigger.
// The `service.agents` actor subscribes to this event and owns the spawn atom.
// EXEMPT from the no-driver-edges guard in `crate::guards` — by design.
register_event!("dispatch.requested", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "dispatch_intent::DispatchRequestedPayload", crate::dispatch_intent::DispatchRequestedPayload);

// ── agents-domain facets (keystone-A Phase A, 2026-06-10) ──────────────────
// Moved from ship-substrate payload_registry.rs: the AGENTS domain owns its
// facet vocabulary (skills / rules / permissions / compile). The def-update
// registration moved to ship-config/src/event_registrations.rs — the
// `agent.def.updated` event is EMITTED by the ship-config watcher/write path,
// so its registration must be link-time present in ship-config. Payload
// structs remain in ship_types::payload (L0 leaf types crate, no
// layering concern); the REGISTRATIONS — ownership of name, scope, retention
// and the carried validator — live with the EMITTER's crate.
register_event!("agent.task.complete", [EventScope::Space], Retention::Durable, Stability::Stable, "AgentTaskComplete", ship_types::payload::AgentTaskComplete);
register_event!("compile.completed", [EventScope::Space], Retention::Durable, Stability::Stable, "CompileCompleted", ship_types::payload::CompileCompleted);
register_event!("permissions.updated", [EventScope::Runtime], Retention::Durable, Stability::Stable, "PermissionsUpdated", ship_types::payload::PermissionsUpdated);
register_event!("rule.created", [EventScope::Runtime], Retention::Durable, Stability::Stable, "RuleCreated", ship_types::payload::RuleCreated);
register_event!("rule.deleted", [EventScope::Runtime], Retention::Durable, Stability::Stable, "RuleDeleted", ship_types::payload::RuleDeleted);
register_event!("rule.updated", [EventScope::Runtime], Retention::Durable, Stability::Stable, "RuleUpdated", ship_types::payload::RuleUpdated);
register_event!("skill.created", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SkillCreated", ship_types::payload::SkillCreated);
register_event!("skill.deleted", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SkillDeleted", ship_types::payload::SkillDeleted);
register_event!("skill.updated", [EventScope::Runtime], Retention::Durable, Stability::Stable, "SkillUpdated", ship_types::payload::SkillUpdated);
register_event!("skill.started", [EventScope::Runtime], Retention::Ephemeral, Stability::Deprecated, "SkillStarted", ship_types::payload::SkillStarted);
register_event!("skill.completed", [EventScope::Runtime], Retention::Ephemeral, Stability::Deprecated, "SkillCompleted", ship_types::payload::SkillCompleted);
register_event!("skill.failed", [EventScope::Runtime], Retention::Durable, Stability::Deprecated, "SkillFailed", ship_types::payload::SkillFailed);
