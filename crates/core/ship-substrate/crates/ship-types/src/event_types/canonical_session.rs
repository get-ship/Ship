//! Canonical ship-shaped agent-session event family — consts only.
//!
//! Two tiers, scope follows granularity:
//!
//! - **Session (coarse)** — Space-scoped, Durable. The whole agent
//!   invocation: one harness, one session_id, one Space, one end state.
//! - **Turn (medium)** — Runtime-scoped, Ephemeral. A single back-and-forth
//!   inside a keepalive session; churns too fast to be Durable.
//!
//! The fine-grained op.* tier is DEFERRED to analytics (it rides the hook
//! stream); it does NOT live here.
//!
//! This family is the COMPLETE ship-shaped session vocabulary (job
//! `canonical-session-events` + the `workbench-teardown` follow-up that
//! adopted `session.started`). The legacy workbench `session.*` family is
//! retired — see R6 / `.artifacts/agents-architecture.md`.

// ── Session (coarse) — Space-scoped, Durable ───────────────────────────────

/// A session was opened. The first event of a session; the only
/// non-terminal coarse event. Space-scoped + Durable so the Space's
/// full history has a row to pivot later terminals off of.
pub const SESSION_STARTED: &str = "session.started";

/// Session reached a normal terminal state (harness exited zero, operator
/// stopped a keepalive cleanly, or completion was declared by the runtime).
pub const SESSION_COMPLETED: &str = "session.completed";

/// Session terminated abnormally (harness exit non-zero, spawn failed,
/// sweep killed the process). Distinct from `cancelled` so observers can
/// distinguish intentional stop from accidental failure.
pub const SESSION_FAILED: &str = "session.failed";

/// Operator cancellation. Rolls up the same as `failed` status-wise, but
/// recorded as the cause (so dashboards can render "why was this stopped").
pub const SESSION_CANCELLED: &str = "session.cancelled";

/// Session paused for later resume; carries a checkpoint handle the
/// runtime can reload to skip already-completed work. Distinct from
/// `failed` — a suspended session is expected to come back.
pub const SESSION_SUSPENDED: &str = "session.suspended";

// ── Turn (medium) — Runtime-scoped, Ephemeral ──────────────────────────────

/// A new turn has begun inside the session. The session is still alive
/// (no terminal event in between). Runtime-scoped, Ephemeral — turns
/// churn too fast for full history.
pub const TURN_STARTED: &str = "session.turn.started";

/// The just-begun turn has ended. The session is now idle, awaiting the
/// next turn. Runtime-scoped, Ephemeral.
pub const TURN_ENDED: &str = "session.turn.ended";

/// Every canonical-session event name in one slice — for the const-set
/// consistency test and any consumer that needs a closed list of this
/// family's events.
pub const ALL_CANONICAL_SESSION: &[&str] = &[
    SESSION_STARTED,
    SESSION_COMPLETED,
    SESSION_FAILED,
    SESSION_CANCELLED,
    SESSION_SUSPENDED,
    TURN_STARTED,
    TURN_ENDED,
];
