---
name: ship-workflow
stable-id: ship-workflow
description: Use when working with Ship's workflow primitives — workspaces, sessions, pipelines, multi-agent coordination, job dispatch, and the session lifecycle.
tags: [ship, workflow, sessions, workspaces, pipelines]
authors: [ship]
---

# Ship Workflow

Ship's workflow layer manages workspaces, sessions, pipelines, and multi-agent coordination. Workspaces are branch-keyed containers for agent work. Sessions track individual units of work within a workspace. Pipelines define multi-step agent workflows.

## Workspaces

A workspace is a branch-based identity for a unit of work. Every workspace is keyed by its git branch name and tracks:

- **Active agent** — the agent profile compiled into this workspace's provider configs
- **Worktree** — whether the workspace has a dedicated git worktree and its filesystem path

Workspaces are created implicitly when a session starts on a branch, or explicitly via `create_workspace`. Workspaces are resource containers, not state machines — they are created, hold sessions, and get archived.

### Workspace Trash

Workspaces support soft-delete via `ship workspace trash`. Trashed files are retained for 24 hours and auto-purged on daemon startup. Restore with `ship workspace restore <filename>`.

## Sessions

A session represents one continuous unit of agent work within a workspace. The lifecycle:

1. **Start** — `start_session` creates a session record, auto-creates the workspace if needed, compiles agent config, and resolves providers.
2. **Work** — the agent performs its task, logging progress with `log_progress`.
3. **End** — `end_session` closes the session, computes duration, creates an immutable session record, persists artifacts, and runs post-session hooks.

Only one active session per workspace at a time. Starting a session on a workspace that already has one returns the existing session (attach semantics).

### Session Artifacts

Sessions persist artifacts to `~/.ship/projects/<slug>/sessions/<session-id>/`:

- `session.json` — current session snapshot
- `timeline.ndjson` — append-only log of phase transitions (start, attach, end)
- `notes.ndjson` — append-only log of progress notes

## Pipelines (v2)

Pipelines define multi-step agent workflows. Define them in `.ship/pipelines/<name>.yaml`.

### Short Form

The simplest format — list agents to run sequentially:

```yaml
name: tdd
description: Test-driven development — tests first, implement second, gate.
agents: [test-writer, "{{ impl_agent }}", reviewer]
```

Each agent becomes a sequential step. The last agent named `reviewer` or `gate` automatically becomes a gate step.

### Long Form

For mixed step types, use the `steps:` key:

```yaml
name: custom
steps:
  - my-agent                           # agent step (bare string)
  - agent: builder                     # agent step with overrides
    model: opus
    provider: claude
    max_retries: 2
    goal: "Build the feature"
  - run: "cargo test"                  # shell command (exit 0 = pass)
  - human: "Review the PR"            # pause for human action
  - parallel:                          # fan-out group
      - test-writer
      - lint-agent
  - reviewer                           # gate step
```

### Step Types

| Type | Syntax | Behavior |
|------|--------|----------|
| `Agent` | bare string or `agent:` object | Spawns an agent process in the workspace worktree |
| `Gate` | agent named `reviewer`/`gate`, or explicit | Review step — can retry failed reviews up to `max_retries` |
| `Run` | `run: "<command>"` | Executes a shell command; exit 0 = success |
| `Human` | `human: "<prompt>"` | Pauses pipeline for human action |
| `Parallel` | `parallel: [...]` | Fan-out: all items run concurrently, next step waits for all |

### Pipeline Variables

Use `{{ var }}` template syntax in agent names and goals. Variables are interpolated at dispatch time.

Define defaults in the YAML:

```yaml
name: standard
vars:
  impl_agent: default-agent
agents: ["{{ impl_agent }}", reviewer]
```

Override from the CLI:

```bash
ship run standard "my task" --var impl_agent=rust-lane
```

CLI/payload vars take precedence over YAML defaults.

### Gate Retry

When a gate step (`StepType::Gate`) observes a failure, the orchestrator checks `max_retries` on the gated step. If retries remain, the failed step is re-dispatched. If exhausted, the pipeline emits `job.failed`.

### Step Done Mechanism

The `StepMonitor` watches the agent process exit. When the process exits, the monitor sets a done flag and the orchestrator calls `advance_pipeline`, which finds newly unblocked steps in the DAG and dispatches them.

### Built-in Pipelines

Ship includes three pipelines:

| Pipeline | Steps | Use case |
|----------|-------|----------|
| `simple` | `{{ agent }}` | Single agent, no gate |
| `standard` | `{{ impl_agent }}` -> `reviewer` | Implement then gate |
| `tdd` | `test-writer` -> `{{ impl_agent }}` -> `reviewer` | Tests first, implement, gate |

## Multi-Agent Coordination

Ship supports multiple agents working in parallel through git worktrees:

- The **commander** operates on the main branch and dispatches work by creating workspaces with dedicated worktrees.
- **Specialist agents** each get their own worktree, branch, workspace, and session. They work in isolation with scoped file access.
- `create_workspace` sets up a git worktree, writes a `workspace.jsonc` config, and optionally assigns a preset and file scope.
- Each workspace tracks which agent is active and which providers are compiled.

### Session Scopes

The daemon enforces per-agent capabilities via scoped sessions. When an MCP server registers with the daemon, it receives a session token with role-based scopes:

| Role | Capabilities |
|------|-------------|
| `commander` | Read/write workspaces, read/create/manage jobs, emit events, mesh send/broadcast, log progress |
| `lead` | Read/write workspaces, read/create jobs, log progress, step done, mesh send, emit events |
| `contributor` | Full access (bypass all scope checks) |
| `worker` | Read workspaces, read jobs, log progress, step done, mesh send, emit events |

## Events

All state changes emit append-only events. Session events include `session.started`, `session.progress`, `session.ended`, and `session.recorded`. Pipeline events include `job.dispatched`, `job.completed`, `job.failed`, `step.observed`. Events are never updated or deleted.

## MCP Tools

| Tool | Purpose |
|------|---------|
| `start_session` | Start or attach to a session. Params: `branch`, `goal`, `agent_id`, `provider_id` |
| `log_progress` | Append a progress note to the active session. Params: `note`, `branch` |
| `end_session` | End the active session with summary and metadata. Params: `branch`, `summary`, `model`, `files_changed`, `gate_result` |
| `get_session` | Get the active session for a workspace |
| `list_sessions` | List sessions, optionally filtered by branch. Params: `branch`, `limit` |
| `create_workspace` | Create a workspace with a git worktree. Params: `name`, `branch`, `base_branch`, `preset_id`, `file_scope` |
| `activate_workspace` | Activate a workspace and compile its agent config. Params: `branch`, `agent_id` |
| `list_workspaces` | List all workspaces, optionally filtered by status |
| `complete_workspace` | Archive a workspace after work is done |
