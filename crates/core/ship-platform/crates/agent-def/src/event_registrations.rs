//! Event registrations for the canonical session family.
//!
//! This module registers the 7 canonical session events with the global
//! event registry using the `inventory::submit!` mechanism, making them
//! available to downstream code that calls `event_registry::registry()`.
//!
//! Uses the hoisted `register_event!` macro from `ship_types` (formerly a
//! local copy in both payload_registry.rs and here; de-duped in keystone-A
//! de-bucket — collection domain owns its events at L1).

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
// R2 hook conversion: the op.* family is the designated replacement for the
// legacy `hook.*` family. The hook→op conversion and hook.* deregistration are
// tracked in #25b and have NOT landed — these registrations are the additive
// contract only; hook ingestion still emits hook.* and nothing emits op.* today.
// Fine-grained (per tool-call) so Runtime + Ephemeral — same scope rule as
// `session.turn.*`, just one more level of granularity.
register_event!("op.tool_use_started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "op::OpToolUseStarted", crate::op::OpToolUseStarted);
register_event!("op.tool_use_completed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "op::OpToolUseCompleted", crate::op::OpToolUseCompleted);
register_event!("op.tool_use_denied", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "op::OpToolUseDenied", crate::op::OpToolUseDenied);

// ── agents-domain facets (keystone-A Phase A, 2026-06-10) ──────────────────
// Moved from ship-substrate payload_registry.rs: the AGENTS domain owns its
// facet vocabulary (skills / rules / permissions / compile / def updates).
// Payload structs remain in ship_types::payload (L0 leaf types crate, no
// layering concern); the REGISTRATIONS — ownership of name, scope, retention
// and the carried validator — live with the domain.
register_event!("agent.def.updated", [EventScope::Space], Retention::Ephemeral, Stability::Stable, "AgentDefUpdated", ship_types::payload::AgentDefUpdated);
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
