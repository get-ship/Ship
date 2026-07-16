//! Canonical ship-shaped agent-session event family — consts only.
//!
//! Three tiers, scope follows granularity:
//!
//! - **Session (coarse)** — Runtime-scoped, Durable. The whole agent
//!   invocation: one harness, one session_id, one end state. Per-machine
//!   execution state, not a collaborative-space concern.
//! - **Turn (medium)** — Runtime-scoped, Ephemeral. A single back-and-forth
//!   inside a keepalive session; churns too fast to be Durable.
//! - **Op (fine-grained)** — Runtime-scoped, Ephemeral. Per-tool-call
//!   observations; churns too fast to be Durable. The `op.*` namespace is the
//!   canonical successor to the `hook.*` family; the contract (consts +
//!   registration) is landed, but the hook→op emission conversion has NOT
//!   happened yet — harness hook data is still emitted as `hook.*` (tracked
//!   #25b).
//!
//! The legacy workbench `session.*` family is retired (jobs
//! `canonical-session-events` + `workbench-teardown`). The
//! third-party-harness `hook.*` family is NOT yet retired — it still
//! coexists with `op.*`; the hook→op conversion and hook.* deregistration
//! are tracked as #25b. See `.artifacts/agents-architecture.md`.

// ── Session (coarse) — Runtime-scoped, Durable ─────────────────────────────

/// A session was opened. The first event of a session; the only
/// non-terminal coarse event. Runtime-scoped + Durable so
/// `canonical_sessions` has a row to pivot later terminals off of.
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

// ── Op (fine-grained) — Runtime-scoped, Ephemeral ──────────────────────────
//
// Tool-use op-card surface. PLANNED R2 conversion (#25b): third-party-harness
// hook data will be converted into these canonical envelopes, and the
// analytics projection repointed from `hook.*` to `op.*`. Today the ingress
// still emits `hook.*` and the analytics projection still reads `hook.*`.

/// A tool invocation is about to start. Carries the structured
/// arguments so analytics can count "all bash commands" or
/// "all file edits to $WORKSPACE/auth" without re-parsing.
pub const OP_TOOL_USE_STARTED: &str = "op.tool_use_started";

/// A tool invocation has completed (ok / error / unknown). Pairs
/// with `OP_TOOL_USE_STARTED` — the start event INSERTs a row,
/// the completion UPDATEs the matching row by `(session_id, tool)`.
pub const OP_TOOL_USE_COMPLETED: &str = "op.tool_use_completed";

/// A tool call was REJECTED before invocation. Distinct from
/// `OP_TOOL_USE_COMPLETED{status="error"}` because the call
/// never produced a started/completed pair — the tool never ran.
pub const OP_TOOL_USE_DENIED: &str = "op.tool_use_denied";

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

/// Every canonical op.* event name in one slice — for the const-set
/// consistency test and any consumer that needs a closed list of this
/// family's events. The op.* family is part of the same L1
/// canonical family as session.* / session.turn.*.
pub const ALL_CANONICAL_OP: &[&str] = &[
    OP_TOOL_USE_STARTED,
    OP_TOOL_USE_COMPLETED,
    OP_TOOL_USE_DENIED,
];
