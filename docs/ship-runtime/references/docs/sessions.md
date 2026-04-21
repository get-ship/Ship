---
group: Runtime
order: 8
title: Sessions
description: WorkspaceSession lifecycle, liveness, progress tracking, session records, drain.
audience: internal
---

# Sessions

`crates/core/runtime/src/workspace/session.rs`, `session_lifecycle.rs`. A session is one agent's visit to a workspace, bounded in time. It starts when the agent begins, ends when the agent finishes or goes missing, and records metrics on close.

Only one **active** session per workspace. Enforced at session-start time — starting a session on a workspace with an existing active session would be an error.

## Types

```rust
pub struct WorkspaceSession {
    pub id: String,
    pub workspace_id: String,
    pub workspace_branch: String,
    pub status: WorkspaceSessionStatus,  // Active | Ended
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub agent_id: Option<String>,
    pub primary_provider: Option<String>,
    pub goal: Option<String>,
    pub summary: Option<String>,
    pub updated_workspace_ids: Vec<String>,
    pub session_record_id: Option<String>,
    pub compiled_at: Option<DateTime<Utc>>,
    pub compile_error: Option<String>,
    pub config_generation_at_start: Option<i64>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub liveness: Option<SessionLiveness>,
    pub stale_context: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum WorkspaceSessionStatus { Active, Ended }
pub enum SessionLiveness { Active, Idle, Abandoned }
```

`WorkspaceSessionRecord` is the immutable end-of-session snapshot. Written once on session close, never mutated. Holds: duration, agent, provider, model, files_changed, lines_added/removed, gate_result, tool_calls, exit_reason, summary.

## Liveness

Computed from `last_activity_at`, not stored:

| State | Condition |
|---|---|
| `Active` | Tool call within the TTL window (`SESSION_TTL_SECS = 600`, 10 minutes). |
| `Idle` | No activity for longer than TTL, but session not explicitly ended. Process may still be alive. |
| `Abandoned` | Session ended by the scanner because its underlying MCP process is dead. |

`SessionLiveness::from_activity(last_activity_at)` does the computation. No heartbeat = `Active` (just started).

## Lifecycle

### start_workspace_session

```rust
pub fn start_workspace_session(
    ship_dir: &Path,
    branch: &str,
    goal: Option<String>,
    agent_id: Option<String>,
    primary_provider: Option<String>,
) -> Result<WorkspaceSession>
```

Flow:

1. Validate branch key.
2. Get or create the workspace via `create_workspace`.
3. Resolve providers for the session's agent (via `compile::resolve_session_providers`).
4. Check for an existing active session — error if one exists.
5. Compile the workspace context (captures config generation for drift detection).
6. Insert the session row via `db::session_events::insert_session_with_started_event` — a single transaction does INSERT + emit `session.started` event.
7. Annotate stale context state (`stale_context: bool`) by comparing current config generation against the one captured.

The transactional insert is what `append_event_with_conn` is for — event and projection update share a connection.

### record_workspace_session_progress

```rust
pub fn record_workspace_session_progress(
    ship_dir: &Path,
    branch: &str,
    progress_note: &str,
) -> Result<()>
```

Appends a `session.progress` event. The session projection deliberately does not handle progress events — progress doesn't mutate the session row. Progress events live in the event log and are read on demand (session detail views, session records).

### end_workspace_session

```rust
pub fn end_workspace_session(
    ship_dir: &Path,
    branch: &str,
    request: EndWorkspaceSessionRequest,
) -> Result<WorkspaceSession>
```

`EndWorkspaceSessionRequest` carries the close-time metrics:

```rust
pub struct EndWorkspaceSessionRequest {
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

1. Resolve workspace + get active session (error if none).
2. If the caller didn't supply git metrics (`files_changed`, `lines_added`, `lines_removed`), collect them via `collect_git_metrics`: `git diff --stat HEAD` against `ship_dir.parent()`, parse the summary line.
3. Update session row + emit `session.ended` via `update_session_with_ended_event` (one transaction).
4. Emit `session.recorded` with the immutable `WorkspaceSessionRecord` for the session.
5. Run post-session hooks (`run_post_session_hooks`) — any `HookTrigger::SessionEnd` or `HookTrigger::Stop` hook configured for the effective agent.

The split between `session.ended` and `session.recorded` is deliberate: `session.ended` closes the live session; `session.recorded` publishes the immutable record for downstream consumers (sync, analytics, projection).

## Session drain

`crates/core/runtime/src/db/session_drain.rs`. Separate lifecycle for cases where a session must be closed without the agent running end:

- `session.drain_started` — marks a session for drain. Usually fired when the daemon detects a dead MCP process.
- `session.drain_completed` — drain finished cleanly.
- `session.drain_aborted` — drain couldn't complete (missing data, race with live activity).

The drain flow is what sets `SessionLiveness::Abandoned` — the scanner ends the session with an exit_reason of `drain`.

## Activity tracking

`last_activity_at` updates come from tool-call events (`session.tool_count_incremented`) and heartbeats (`session.heartbeat`). Liveness computation reads the column; no code writes liveness directly.

## Queries

| Function | Purpose |
|---|---|
| `get_active_workspace_session(workspace_id)` | Fetch the Active session if one exists. |
| `list_workspace_sessions(workspace_id)` | All sessions for a workspace, ordered newest first. |
| `get_workspace_session_record(session_id)` | Fetch the immutable end-record. |
| `record_workspace_session_progress(...)` | Append a `session.progress` event. |

## Artifact persistence

`session.rs` exposes `persist_session_artifact` — writes an artifact produced during the session to the `agent_artifact_registry` table with content-hashed id, associates it with the session_id. Emits an artifact event (handled in `artifact_events.rs`).

## Event taxonomy for sessions

From `ship-types::event_types`:

| Constant | Meaning |
|---|---|
| `SESSION_STARTED` | Session begins. Includes agent, provider, goal. |
| `SESSION_PROGRESS` | Progress note. Not projected — event log only. |
| `SESSION_ENDED` | Session closes. Includes summary + metrics. |
| `SESSION_RECORDED` | Immutable session record published. |
| `SESSION_ATTACHED` | External attach event (e.g. terminal reconnect). |
| `SESSION_HEARTBEAT` | Liveness signal. |
| `SESSION_TOOL_COUNT_INCREMENTED` | Tool call counter tick. Updates activity timestamp. |
| `SESSION_DRAIN_STARTED` / `_COMPLETED` / `_ABORTED` | Drain lifecycle. |

## Invariants

- One active session per workspace. Start rejects if another is active.
- `ended_at` is set exactly once. Cannot be cleared.
- `WorkspaceSessionRecord` is write-once-on-close. Never updated.
- Liveness is computed, never stored. `last_activity_at` is the only stored signal.
- `session.progress` events are not projected. They live in the event log and are consumed on demand.

## Related

- [Workspaces](/reference/runtime/workspaces/) — sessions live inside a workspace.
- [Projections](/reference/runtime/projections/) — `SessionProjection` maintains the `workspace_session` table.
- [Platform DB](/reference/runtime/platform-db/) — `workspace_session` and `workspace_session_record` tables.
- [Hooks](/reference/runtime/hooks/) — `SessionEnd` and `Stop` hook triggers.
