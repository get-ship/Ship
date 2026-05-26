---
group: Runtime
order: 8
title: Sessions
description: Session lifecycle, liveness, progress tracking, inline telemetry, drain.
audience: internal
---

# Sessions

`crates/core/ship-platform/src/sessions/session.rs`, `session_lifecycle.rs`, `types_session.rs`. A session is one agent's bounded-in-time interval of work on a branch. It starts when the agent begins, ends when the agent finishes or goes missing, and records metrics inline on close. Sessions are immutable analytics records over actor activity — the `workspace` runtime primitive was retired entirely.

Only one **active** session per branch. `start_session` is idempotent: if an active session already exists it returns that session and emits `session.attached` — it never errors.

## Types

```rust
pub struct Session {
    pub id: String,
    pub workspace_id: String,
    pub branch: String,
    pub status: SessionStatus,  // Active | Ended
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub agent_id: Option<String>,
    pub primary_harness: Option<String>,
    pub goal: Option<String>,
    pub summary: Option<String>,
    pub updated_workspace_ids: Vec<String>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub liveness: Option<SessionLiveness>,
    // End-of-session telemetry — set on session.ended, None while active:
    pub duration_secs: Option<i64>,
    pub model: Option<String>,
    pub files_changed: Option<i64>,
    pub lines_added: Option<i64>,
    pub lines_removed: Option<i64>,
    pub tool_calls: Option<i64>,
    pub gate_result: Option<String>,
    pub exit_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum SessionStatus { Active, Ended }
pub enum SessionLiveness { Active, Idle }
```

End-of-session telemetry (`duration_secs`, `model`, `files_changed`, `lines_added/removed`, `tool_calls`, `gate_result`, `exit_reason`) lands inline on the `Session` row when `session.ended` is applied. There is no separate record type — `WorkspaceSessionRecord` and the `session.recorded` event were deleted.

## Liveness

Computed from `last_activity_at`, not stored:

| State | Condition |
|---|---|
| `Active` | Tool call within the TTL window (`SESSION_TTL_SECS = 600`, 10 minutes), or no activity yet (just started). |
| `Idle` | No activity for longer than TTL, but session not explicitly ended. Process may still be alive. |

`SessionLiveness::from_activity(last_activity_at)` does the computation — it only ever returns `Active` or `Idle`. No heartbeat = `Active`.

## Lifecycle

### start_session

```rust
pub fn start_session(
    ship_dir: &Path,
    branch: &str,
    goal: Option<String>,
    agent_id: Option<String>,
    primary_harness: Option<String>,
) -> Result<Session>
```

Flow:

1. Validate branch key; derive `workspace_id` via `sanitize_file_name(branch)`.
2. **Idempotent check** — if an active session already exists for the branch, emit `session.attached`, persist an "attach" artifact, and return the existing session. No error.
3. Resolve harnesses for the session's agent (via `compile::resolve_session_harnesses`) and the primary harness.
4. Compile the session context.
5. Resolve `space_id` — the path-hash of the project root (`space_id_for_ship_dir`), always populated. It is the sole space scope; the legacy branch-slug `workspace_id` is no longer a scope key.
6. Insert the session row via `db::session_events::insert_session_with_started_event` — a single transaction does INSERT + emit `session.started`.

The transactional insert keeps event append and projection update on one connection.

### record_session_progress

```rust
pub fn record_session_progress(
    ship_dir: &Path,
    branch: &str,
    progress_note: &str,
) -> Result<()>
```

Appends a `session.progress` event. The session projection deliberately does not handle progress events — progress doesn't mutate the session row. Progress events live in the event log and are read on demand (session detail views).

### end_session

```rust
pub fn end_session(
    ship_dir: &Path,
    branch: &str,
    request: EndSessionRequest,
) -> Result<Session>
```

`EndSessionRequest` carries the close-time metrics:

```rust
pub struct EndSessionRequest {
    pub summary: Option<String>,
    pub updated_workspace_ids: Vec<String>,
    pub model: Option<String>,
    pub files_changed: Option<i64>,
    pub gate_result: Option<String>,
    pub lines_added: Option<i64>,
    pub lines_removed: Option<i64>,
    pub tool_calls: Option<i64>,
    pub exit_reason: Option<String>,
}
```

Flow:

1. Get the active session for the branch — error if none.
2. Compute `duration_secs` from `started_at` / `ended_at`.
3. Collect git metrics best-effort via `git::collect_metrics(project_dir)` — failure doesn't block session end. Caller-supplied `files_changed` / `lines_added` / `lines_removed` take precedence.
4. Update session row + emit `session.ended` via `update_session_with_ended_event` (one transaction). End-of-session telemetry lands inline on the session row — there is no separate `session.recorded` event.

## Session drain

`crates/core/ship-substrate/src/db/session_drain.rs`. Separate lifecycle for cases where a session must be closed without the agent running end:

- `session.drain_started` — marks a session for drain. Usually fired when the daemon detects a dead MCP process.
- `session.drain_completed` — drain finished cleanly.
- `session.drain_aborted` — drain couldn't complete (missing data, race with live activity).

The drain flow ends the session with an exit_reason of `drain` when the scanner detects a dead process.

## Activity tracking

`last_activity_at` updates come from tool-call events (`session.tool_count_incremented`) and heartbeats (`session.heartbeat`). Liveness computation reads the column; no code writes liveness directly.

## Queries

| Function | Purpose |
|---|---|
| `get_active_session(workspace_id)` | Fetch the Active session if one exists. |
| `list_sessions(workspace_id)` | All sessions for a branch, ordered newest first. |
| `record_session_progress(...)` | Append a `session.progress` event. |

## Artifact persistence

`session.rs` exposes `persist_session_artifact` — writes an artifact produced during the session to the artifact registry with content-hashed id, associates it with the session_id.

## Event taxonomy for sessions

From `ship-types::event_types`:

| Constant | Meaning |
|---|---|
| `SESSION_STARTED` | Session begins. Includes agent, harness, goal. |
| `SESSION_PROGRESS` | Progress note. Not projected — event log only. |
| `SESSION_ENDED` | Session closes. Carries summary + inline telemetry. |
| `SESSION_ATTACHED` | Idempotent attach — `start_session` found an active session. |
| `SESSION_HEARTBEAT` | Liveness signal. |
| `SESSION_TOOL_COUNT_INCREMENTED` | Tool call counter tick. Updates activity timestamp. |
| `SESSION_DRAIN_STARTED` / `_COMPLETED` / `_ABORTED` | Drain lifecycle. |
| `SESSION_STATUS_CHANGED` | Session status transition (e.g. Idle → Thinking). |
| `SESSION_TURN_FAILED` | A turn failed to spawn or run end-to-end; the session stays alive. |

## Invariants

- One active session per branch. `start_session` is idempotent — it returns the existing session and emits `session.attached`, never errors.
- `ended_at` is set exactly once. Cannot be cleared.
- End-of-session telemetry lands inline on the `Session` row on close. No separate record type.
- Liveness is computed, never stored. `last_activity_at` is the only stored signal.
- `session.progress` events are not projected. They live in the event log and are consumed on demand.

## Related

- [Projections](/runtime/projections/) — `SessionProjection` maintains the `sessions` table.
- [Platform DB](/runtime/platform-db/) — the `sessions` table.
- [Hooks](/runtime/hooks/) — agent lifecycle hooks.
