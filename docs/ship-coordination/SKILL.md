---
name: ship-coordination
stable-id: ship-coordination
audience: internal
description: Use when logging progress or tracking work status in the Ship project. Covers ship MCP tools for workspace + job coordination, commit discipline, and cross-workspace signaling.
tags: [ship, coordination, mcp, internal]
authors: [ship]
---

# Ship Coordination Protocol

Agents coordinate through MCP tools backed by a per-project SQLite
database. The orchestrator (Commander) monitors state and routes work.

The tool surface evolved — there is **no** separate `start_session` /
`end_session` / `get_session` / `list_sessions` / `activate_workspace` /
`list_workspaces` / `set_agent`. The session is implicit on the workspace:
`create_workspace` sets it up, `log_progress` checkpoints it,
`complete_workspace` finalizes it.

## Workspace lifecycle (the full surface)

```
create_workspace       — create a new workspace (sets up git worktree, opens session)
log_progress           — checkpoint within a workspace; include a clear note + optional escalation
complete_workspace     — finalize the workspace with a handoff summary
write_session_file     — drop a file into the workspace's session for cross-agent context
```

These are the four canonical workspace tools. There is no separate
session API anymore.

## Job orchestration

```
create_job        — register a job + its steps (commander-only)
update_job        — update overall job status
step_done         — mark a single step complete; advances the pipeline
```

A dispatched agent's normal endgame is **`step_done`** (advances the
pipeline) followed by `complete_workspace` (finalizes the workspace
with a handoff summary).

## Mesh signaling (canonical names)

```
send_mesh_message       — directed message to one agent
broadcast_mesh_message  — broadcast to a topic (e.g. job.completed)
discover_mesh_agents    — find agents listening on a topic
update_mesh_status      — flip own status active|busy|idle
```

The old `mesh_send` / `mesh_broadcast` / `mesh_discover` / `mesh_status`
are deprecated aliases and currently still route to the canonical
names. New code should use the verb-first canonical form above.

## Documents (durable handoffs)

```
create_doc / update_doc / get_doc / delete_doc / query_docs
```

Use docs for **durable, queryable handoffs** that another workspace
needs to read days/weeks later (architecture decisions, contracts,
reference material). For ephemeral notes use `log_progress` or
`.artifacts/` files; for transient cross-workspace pings use
`broadcast_mesh_message`.

## View tools (canvas / studio integration)

```
get_view_errors    — read view compilation/lint errors
lint_view          — lint a SUI view
query_sdk          — look up SDK contracts (routes, types, etc.)
scaffold_view      — scaffold a new view
```

(Old `ship_view_errors` / `ship_view_lint` / `ship_sdk_query` /
`ship_view_scaffold` are deprecated aliases.)

## Artifacts

```
upload_artifact / get_artifact / list_artifacts / delete_artifact / promote_artifact
```

(Old `artifact_upload` / `artifact_get` / `artifact_list` /
`artifact_delete` / `artifact_promote` are deprecated aliases.)

## What to log

**create_workspace**: the task, branch, and goal.
**log_progress**: what was done, what is next, any discoveries that
affect other workspaces.
**complete_workspace**: what was completed, what was not done,
what unblocks downstream — copy real evidence (test counts, bundle
sizes, screenshot paths) — never just "Gate ✓".

## Cross-workspace signals

When your work unblocks another workspace, use `log_progress` with a
clear message:

```
log_progress "[UNBLOCKS web-import] /api/github/import contract stable — types: ProjectLibrary, errors: 404/422"
```

For broader broadcasts (job completed, blockers cleared) use
`broadcast_mesh_message`:

```
broadcast_mesh_message { topic: 'job.completed', workspace_id, summary }
```

## Commit discipline (HARD RULE — agents miss this)

**You MUST commit your work to git before calling `complete_workspace`.**
The dispatched-agent flow is:

```
1. git add <specific paths>     ← never `git add -A`; .env etc. leak
2. git commit -m 'feat: ...'    ← imperative; no AI attribution in subject
3. (optional) step_done         ← advances pipeline if part of a job
4. complete_workspace summary
5. broadcast_mesh_message
```

Skipping step 1-2 leaves work in the worktree only — if the
worktree is removed (post-merge cleanup) the work is lost. Commit
even WIP that fails its gate; commander can squash later.

Commit message rules:
- Type prefix: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`
- Subject in imperative mood, concise (under 72 chars)
- Body explains the WHY when non-obvious
- **No AI attribution in the subject line** (Co-Authored-By in trailer is fine)
- Stage explicit paths — `git add -A` and `git add .` can leak `.env`,
  `package-lock.json` from accidental npm invocations, large binaries

## Test commands

When the spec requires running tests, use **bun** (this is a bun-only
workspace — `bun.lock` is the lockfile) and **always wrap with a
timeout** so a hung test fails fast instead of pinning CPU for hours:

```
timeout 120 bun run --filter @ship/<package> test --run
```

Never use `npx vitest run` — npx pulls dependencies through npm and
creates a `package-lock.json` orphan. The repo uses bun; respect it.
