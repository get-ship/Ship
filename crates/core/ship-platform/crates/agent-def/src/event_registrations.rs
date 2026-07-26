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
// rule: the WHOLE family is Runtime — a session is per-machine execution
// state, not a collaborative-space concern, and its projection
// (`canonical_sessions`) lives in `runtime.db`. Retention: coarse events
// are Durable; turn events are Windowed (persisted like Durable, swept
// past 7 days — projections folding them are bounded by that horizon).
// The workbench family is RETIRED (R6) — no `SessionStarted` workbench
// payload struct, no `session.attached` etc.
register_event!(
    "session.started",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Stable,
    "canonical_session::SessionStarted",
    crate::canonical_session::SessionStarted
);
register_event!(
    "session.completed",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Stable,
    "canonical_session::SessionCompleted",
    crate::canonical_session::SessionCompleted
);
register_event!(
    "session.failed",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Stable,
    "canonical_session::SessionFailed",
    crate::canonical_session::SessionFailed
);
register_event!(
    "session.cancelled",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Stable,
    "canonical_session::SessionCancelled",
    crate::canonical_session::SessionCancelled
);
register_event!(
    "session.suspended",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Stable,
    "canonical_session::SessionSuspended",
    crate::canonical_session::SessionSuspended
);
register_event!(
    "session.turn.started",
    [EventScope::Runtime],
    Retention::Windowed { ttl_days: 7 },
    Stability::Stable,
    "canonical_session::TurnStarted",
    crate::canonical_session::TurnStarted
);
register_event!(
    "session.turn.ended",
    [EventScope::Runtime],
    Retention::Windowed { ttl_days: 7 },
    Stability::Stable,
    "canonical_session::TurnEnded",
    crate::canonical_session::TurnEnded
);

// ── op.* (fine-grained) — Runtime-scoped, Windowed 7d ──────────────────────
// R2: op.* replaces the retired hook.* family (#25b, merged). Emitters:
// http_ingress → op.* conversion; ship-code synth_emit.rs → op.tool_use_*.
// Fine-grained (per tool-call): persisted so analytics fold in-TX, swept
// past 7 days — the analytics projections claim the window, not history.
register_event!(
    "op.tool_use_started",
    [EventScope::Runtime],
    Retention::Windowed { ttl_days: 7 },
    Stability::Stable,
    "op::OpToolUseStarted",
    crate::op::OpToolUseStarted
);
register_event!(
    "op.tool_use_completed",
    [EventScope::Runtime],
    Retention::Windowed { ttl_days: 7 },
    Stability::Stable,
    "op::OpToolUseCompleted",
    crate::op::OpToolUseCompleted
);
register_event!(
    "op.tool_use_denied",
    [EventScope::Runtime],
    Retention::Windowed { ttl_days: 7 },
    Stability::Stable,
    "op::OpToolUseDenied",
    crate::op::OpToolUseDenied
);

// ── dispatch.* (spawn-intent channel, #26) ─────────────────────────────────
// Runtime-scoped, Windowed 7d: the intent is fulfilled immediately, but the
// recent dispatch record is queryable for post-mortems inside the window.
// The ONE sanctioned driver-edge carrier. Replaces the legacy
// `agent.session.started` as the hard-failing spawn trigger.
// The `service.agents` actor subscribes to this event and owns the spawn atom.
// EXEMPT from the no-driver-edges guard in `crate::guards` — by design.
register_event!(
    "dispatch.requested",
    [EventScope::Runtime],
    Retention::Windowed { ttl_days: 7 },
    Stability::Stable,
    "dispatch_intent::DispatchRequestedPayload",
    crate::dispatch_intent::DispatchRequestedPayload
);

// ── agents-domain facets (keystone-A Phase A, 2026-06-10) ──────────────────
// Moved from ship-substrate payload_registry.rs: the AGENTS domain owns its
// facet vocabulary (skills / rules / permissions / compile) AND the
// `agent.def.updated` def-update event. Payload structs remain in
// ship_types::payload (L0 leaf types crate, no layering concern); the
// REGISTRATIONS — ownership of name, scope, retention
// and the carried validator — live with the domain.
// `agent.def.updated` is the durable domain event for an agent-def file
// change. Reclassified from Ephemeral to Durable in job/transient-events:
// the read-only agent-def index (`ship-config::agents_index`) reads back
// from the space db on boot, and the agent-fanout consumers
// (`ship-agents::services::agents::watch`) key off its presence.
// Registered here (the domain owner); ship-config's tests link agent-def via a
// dev-dependency, and the daemon links both in production.
register_event!(
    "agent.def.updated",
    [EventScope::Space],
    Retention::Durable,
    Stability::Stable,
    "AgentDefUpdated",
    ship_types::payload::AgentDefUpdated
);
register_event!(
    "agent.task.complete",
    [EventScope::Space],
    Retention::Durable,
    Stability::Stable,
    "AgentTaskComplete",
    ship_types::payload::AgentTaskComplete
);
register_event!(
    "compile.completed",
    [EventScope::Space],
    Retention::Durable,
    Stability::Stable,
    "CompileCompleted",
    ship_types::payload::CompileCompleted
);
register_event!(
    "permissions.updated",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Stable,
    "PermissionsUpdated",
    ship_types::payload::PermissionsUpdated
);
register_event!(
    "rule.created",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Stable,
    "RuleCreated",
    ship_types::payload::RuleCreated
);
register_event!(
    "rule.deleted",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Stable,
    "RuleDeleted",
    ship_types::payload::RuleDeleted
);
register_event!(
    "rule.updated",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Stable,
    "RuleUpdated",
    ship_types::payload::RuleUpdated
);
register_event!(
    "skill.created",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Stable,
    "SkillCreated",
    ship_types::payload::SkillCreated
);
register_event!(
    "skill.deleted",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Stable,
    "SkillDeleted",
    ship_types::payload::SkillDeleted
);
register_event!(
    "skill.updated",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Stable,
    "SkillUpdated",
    ship_types::payload::SkillUpdated
);
register_event!(
    "skill.failed",
    [EventScope::Runtime],
    Retention::Durable,
    Stability::Deprecated,
    "SkillFailed",
    ship_types::payload::SkillFailed
);
