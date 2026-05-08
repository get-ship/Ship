---
group: MCP Tools
order: 4
title: Workflow Patterns
description: The three-mode workflow -- planning, orchestration, and gate -- using workspaces, sessions, and jobs.
audience: internal
---

# Workflow Patterns

Three operational modes use the MCP tools: planning (understand the work), orchestration (dispatch and monitor), and gate (verify completion).

## Planning Mode

Read project state, define goals, break work into capabilities.

### Typical sequence

1. Read resources for context: `ship://project_info`, `ship://jobs`, `ship://workspaces`
2. Create jobs via `create_job` to capture planned work

### What planning produces

- Jobs documenting planned work
- Understanding of existing state from resources

## Orchestration Mode

Create workspaces, dispatch jobs, monitor progress, handle file conflicts.

### Workspace setup

```
create_workspace(name="auth-flow", kind="imperative", base_branch="main")
```

Workspace kinds determine worktree lifecycle:

| Kind | Worktree | On completion |
|------|----------|---------------|
| imperative | created | pruned |
| declarative | created | kept |
| service | none | n/a |

### Job dispatch

```
create_job(
  kind="feature",
  description="Implement OAuth provider",
  branch="auth-flow",
  file_scope=["apps/gateway/src/routes/"],
  acceptance_criteria=["OAuth flow completes", "Tokens stored"],
  capability_id="<cap-id>"
)
```

Jobs move through: `pending` -> `running` -> `complete` or `failed`.

### Monitoring

- Read `ship://events` -- watch for recent state changes across all workspaces
- Read `ship://workspaces` -- check workspace state

### Job dependencies

Jobs can block on other jobs:

```
create_job(kind="feature", description="Build login UI", blocked_by="<auth-job-id>")
```

The blocked job stays `pending` until the blocking job completes. Higher `priority` values schedule first.

## Gate Mode

Verify completed work against acceptance criteria. Either pass (capability becomes actual) or fail (job gets feedback).

### Pass sequence

```
update_job(id="<job-id>", status="completed")
complete_workspace(workspace_id="auth-flow", summary="OAuth flow implemented and tested")
```

`complete_workspace` writes a `handoff.md` in the worktree root and optionally prunes the worktree (default for imperative workspaces).

### Fail sequence

```
update_job(id="<job-id>", status="failed")
```

The job stays in the queue with failure context. A new job or a retry can pick up the remaining work.

## Session Lifecycle

Sessions are tracked automatically. The MCP server starts and manages a session on connection. Use `log_progress` to record progress notes during a session.

```
log_progress(note="Created auth provider config")
log_progress(note="Login UI renders, testing OAuth flow")
```

Session records are immutable once ended. They include: start/end timestamps, goal, progress notes, summary, file count, model used, and optional gate result.

## Event Log

Every mutation (workspace creation, session start/end, job updates) emits an event to the append-only log. Agents emit custom domain events via the `event` tool and read the log via the `ship://events` resource:

- `ship://events` -- 100 most recent events
- `ship://events/20` -- last 20 events

The event log is the audit trail for understanding what happened across all workspaces and agents.

## End-to-End Example

1. **Plan:** Read `ship://jobs` and `ship://workspaces`, create jobs for planned work
2. **Dispatch:** Create imperative workspace `auth-flow`, create job for the feature
3. **Work:** Implement the feature, log progress at checkpoints with `log_progress`
4. **Gate:** Run tests, update job to completed with `update_job`
5. **Complete:** Call `complete_workspace` (writes handoff.md, prunes worktree)

Each step uses the tools from the corresponding mode. The event log captures the full history.
