---
title: "Sessions"
sidebar:
  label: "Sessions"
  order: 3
---
A session is a heartbeat within a workspace. It represents one continuous unit of agent work -- from the moment an agent starts a task to the moment it finishes or is interrupted.

## Session fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Unique session ID (nanoid). |
| `workspace_id` | string | ID of the parent workspace. |
| `workspace_branch` | string | Branch name of the parent workspace. |
| `status` | `active` or `ended` | Current session state. |
| `started_at` | datetime | When the session began. |
| `ended_at` | datetime (optional) | When the session ended. |
| `agent_id` | string (optional) | Agent profile active during this session. |
| `primary_provider` | string (optional) | AI provider used (e.g. "claude", "gemini"). |
| `goal` | string (optional) | What the session set out to accomplish. |
| `summary` | string (optional) | What was accomplished (set on end). |
| `stale_context` | bool | Whether the workspace config changed after this session started. |
| `config_generation_at_start` | i64 (optional) | Config generation counter at session start, used for staleness detection. |
| `session_record_id` | string (optional) | ID of the immutable record created on end. |

## Starting a session

Sessions are started automatically by the daemon when it dispatches an agent into a worktree. The runtime performs these steps:

1. Resolves the branch from the worktree's git HEAD.
2. Auto-creates the workspace record if it does not exist.
3. Activates the workspace if it is not already active.
4. Checks for an existing active session. If one exists, attaches to it.
5. Resolves allowed providers from the agent config and validates the requested provider.
6. Compiles the workspace context (agent config into provider-specific output).
7. Emits a `session.started` event and creates the session record.
8. Persists a session artifact snapshot.

Only one active session per workspace. The attach-on-existing behavior prevents duplicate sessions when an agent restarts or reconnects.

Agents do not call `start_session` directly — it is not available as an agent-facing MCP tool on the ShipServer.

## Logging progress

```
log_progress({
  note: "Completed sidebar layout, starting event handlers",
  branch: "feat/new-ui"    // optional
})
```

Progress notes are appended to the session as `session.progress` events. They serve two purposes:

- **Observability** -- Other agents (like a commander) can monitor what specialists are doing.
- **Persistence** -- Notes are written to `notes.ndjson` in the session artifacts directory.

`log_progress` requires an active session. If no session exists on the branch, it returns an error asking you to call `start_session` first.

Notes cannot be empty. The runtime trims whitespace and rejects blank strings.

## Ending a session

Sessions end when an agent calls `complete_workspace` (available on ShipServer) or when the daemon-side session lifecycle completes. The runtime performs these steps:

1. Finds the active session on the workspace.
2. Sets the session status to `ended` and records the end timestamp.
3. Computes duration from `started_at` to `ended_at`.
4. Emits a `session.ended` event with summary, duration, and gate result.
5. Creates an immutable `WorkspaceSessionRecord` with the session's metadata.
6. Emits a `session.recorded` event.
7. Persists the final session artifact snapshot and appends to the timeline.
8. Runs any configured post-session hooks (hooks with trigger `session_end` or `stop`).

## Session records

When a session ends, the runtime creates a `WorkspaceSessionRecord` -- an immutable snapshot of the session's outcome. Records include:

| Field | Description |
|-------|-------------|
| `summary` | What was accomplished. |
| `duration_secs` | How long the session lasted. |
| `provider` | AI provider used. |
| `model` | Model ID used. |
| `agent_id` | Agent profile that was active. |
| `files_changed` | Number of files modified. |
| `gate_result` | Quality gate outcome (pass/fail/skip). |

Records are write-once. They provide the historical trail for what each session produced.

## Staleness detection

Sessions track the workspace's config generation counter at start time. If the workspace config is recompiled while a session is active (e.g., because the agent profile changed), the session is marked `stale_context: true`. This signals that the agent may be operating with outdated instructions.

## Events

Sessions emit four event types:

| Event | When | Payload |
|-------|------|---------|
| `session.started` | Session begins | goal, workspace_id, agent_id, provider, config_generation |
| `session.progress` | Agent logs progress | message |
| `session.ended` | Session closes | summary, duration_secs, gate_result, updated_workspace_ids |
| `session.recorded` | Immutable record created | record_id, summary, duration, provider, model, files_changed |

Events are append-only. They are never updated or deleted.

## Session artifacts

The runtime persists session artifacts to `~/.ship/projects/<slug>/sessions/<session-id>/`:

- **`session.json`** -- JSON snapshot of the session, updated at start, attach, and end.
- **`timeline.ndjson`** -- Append-only log of phase transitions with timestamps.
- **`notes.ndjson`** -- Append-only log of progress notes with timestamps.

These files exist for debugging and observability. They are not the source of truth -- the event store is.

## Post-session hooks

Hooks configured with trigger `session_end` or `stop` run after a session ends. They receive `SHIP_SESSION_ID` and `SHIP_SESSION_BRANCH` as environment variables. Hook failures are logged to stderr but do not fail the session end operation.

## Querying sessions

Read session state via MCP resources (available on ShipServer):

```
ship://sessions             # recent sessions across all workspaces (up to 50)
ship://sessions/{workspace} # sessions for a specific workspace
ship://workspaces/{branch}/session  # active session for a branch
```

`list_sessions` and `get_session` are not available as agent-facing MCP tools; use the `ship://` resources above instead.
