---
title: "Overview"
sidebar:
  label: "Overview"
  order: 1
---
Ship's workflow layer tracks what agents are doing, where they are doing it, and what happened. It is built on three runtime primitives: workspaces, sessions, and events.

## How the pieces fit together

```
Project
  └── Workspace (branch: "feat/new-ui")
        ├── active_agent: "web-lane"
        ├── status: active
        └── Session (goal: "implement sidebar")
              ├── progress notes
              ├── session record (immutable, created on end)
              └── events (session.started, session.progress, session.ended)
```

A **workspace** is identity. It answers "who is working on what branch, with which agent config." A workspace is keyed by its git branch name and persists across sessions.

A **session** is a heartbeat. It answers "what is this agent doing right now." Sessions are short-lived -- they start when an agent begins work and end when it finishes or is interrupted.

**Events** are the append-only audit trail. Every workspace activation, session start, progress note, and session end emits an event. Events are never updated or deleted.

## Multi-agent coordination

Ship supports multiple agents working in parallel through git worktrees. The pattern:

1. A **commander** agent operates on the main branch. It reads project state (targets, capabilities, jobs) and decides what work to dispatch.
2. The commander creates workspaces for specialist agents using `create_workspace`, which sets up a git worktree branched from a base (usually `main`).
3. Each specialist gets its own worktree directory, branch, workspace, and session. It works in filesystem isolation -- its file scope is limited to its worktree.
4. The commander monitors progress via `ship://workspaces` and `ship://sessions` resources. Specialists log their work via `log_progress`.
5. When a specialist finishes, it calls `complete_workspace` to archive the workspace.

This model keeps agents from stepping on each other's files while maintaining a shared project database for coordination.

## The typical agent workflow

```
(session auto-started by daemon)  →  log_progress (repeated)  →  complete_workspace
```

1. **Session auto-start** -- The daemon auto-creates a workspace and session when an agent begins work in a worktree. The agent does not call `start_session` directly; the daemon handles this on dispatch.
2. **Work** -- The agent performs its task. It calls `log_progress` periodically via the Ship MCP tool to record what it did, decided, or got blocked on.
3. **Complete** -- When done, the agent calls `complete_workspace` (available on ShipServer) to archive the workspace and write a handoff note.

## State storage

Workspace and session state lives in Ship's SQLite database, accessed through MCP tools or CLI commands. Session artifacts (snapshots, timelines, notes) are persisted to the global Ship directory at `~/.ship/projects/<slug>/sessions/`.

All state access goes through the runtime API. Direct database queries are not supported.
