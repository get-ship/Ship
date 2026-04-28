---
group: Architecture
order: 1
title: Runtime Model
description: The mental model for what Ship's runtime owns — events as the write path, projections in platform.db, collections as a feature. Project is a directory with .ship/; actors own isolation; worktrees are an attachment, not a primitive.
---

# Runtime Model

This is the model Ship was designed around. Read this before changing
where state lives, what's a primitive, or how scope works. The
substrate has drifted from it more than once; the drift is the
antipattern, not the model.

## Domain — what the runtime owns

**Ship's runtime IS the domain.** The runtime crate owns persistent
state for the daemon, and the daemon is the user's control plane.
Everything that happens — a session starting, an agent spawning, a job
dispatched, an integration registered — happens *as a runtime event*
or it doesn't happen.

The runtime is local-first. One daemon per user. Cloud sync is a
transport for select state, not a tier the daemon defers to. Macaroons
gate shared resources at the cloud edge; the daemon stays single-user.

## Write path — events are canonical

Events are the canonical write path for runtime state. Append-only,
immutable, kernel-mediated. Every state change of consequence emits an
event; every concrete table is a projection of those events.

```
KernelRouter::emit(event) → events table → projection → platform.db row
```

Projections live in `runtime::projections::*` and write into
platform.db tables (or kv namespaces) that are *derived state*, not
primary records:

| Projection | Source events | Target |
|---|---|---|
| `projections::session` | `session.*` | `workspace_session` table |
| `projections::actor` | `actor.*` | `actors` table |
| `projections::workspace` | `workspace.*` | `kv_state` namespace `worktree` (via `runtime::worktrees`) |
| `projections::job` (in-memory) | `job.*` / `step.*` | `JobRecord` map, computed on demand |

The job projection is a fold-in-memory rather than a table — there's
no `jobs` row to consult; `runtime::projections::job::load_jobs()`
queries the event log directly. The workspace projection writes to
kv-state instead of a table; worktrees are an attachment, not a
runtime primitive (see Five Primitives below).

This is the canonical pattern. New runtime concerns ride it.

## Antipatterns

**Bespoke tables that bypass events.** A table whose rows are written
by ad-hoc SQL inserts, not as a projection of events, is an
antipattern. Recent examples we've cleaned up:

- `branch_config` — created in 0001, zero callers ever. Dead.
- `file_claim` — same. Dead.
- `git_commit_drafts` — full plumbing (table, module, trait, handlers,
  routes, SDK), zero callers. Deleted.
- `managed_mcp_state` — two scalar fields per provider; was a
  dedicated table, moved to `kv_state`. Should ultimately be event-
  sourced (see Integrations below) but the move-to-KV was an
  improvement.
- `spec` — dedicated table for what was already a queryable
  user-content row. Migrated into the `specs` collection.
- `workspace` — projection table for a primitive that conflated
  session-identity, dev-worktree, tmux-attachment, and agent-sandbox.
  Each of those jobs moved to its proper home (sessions table, kv
  namespace `worktree`, actor row, actor row). Table dropped; the
  workspace.* events now project to kv via `runtime::worktrees`.
- `workspace_session_record` — write-only mirror of session telemetry
  that was already (or could trivially be) on `workspace_session`.
  Collapsed.
- `jobs` — declared in schema, zero R/W; job state lives entirely in
  the event log, projected in-memory by `load_jobs()`.
- `compiled_asset_cache` — claimed to be a content-hash dedupe cache
  but `content_hash` was write-only; the table was actually a uuid ↔
  external_id translator for ModeRow refs in kv. Tautological. Excised.

**Rust "migrator" code that lives forever.** Pre-launch we wipe and
rebuild rather than carry data-migration helpers in the binary
indefinitely. SQL migrations (sqlx, timestamp-prefix files) are fine
forever — they're declarative DDL. Rust orchestrators that copy rows
between shapes are debt.

## Storage backends

The daemon has six places state can live. Each is right-fit for one
shape; using the wrong backend is the leak.

| Backend | Right when | Wrong when |
|---|---|---|
| **Events** (`events` table) | Append-only history of state transitions; kernel-touching; replay-able; multiple subscribers care | Mutable settings; high-frequency reads of a single key |
| **Projections** (platform.db tables) | Derived current-state of an event-sourced concern (sessions, jobs, actors) | Hand-written rows that bypass the event log |
| **KV** (`kv_state`) | Mutable settings, namespaced; small structured values; per-key access | Anything with relations or a lifecycle worth modeling |
| **Disk** (`<scope>/.ship/<vector>/`) | Authored config Ship's runtime depends on; external-tool readable (claude-code et al); hot-reloadable; git-trackable | Runtime-mutable state; transient; high-frequency writes |
| **Collections** (data.db) | User-authored or user-saved structured data; multi-instance; queryable; UI-edited | Authored *config* the runtime depends on; needs external-tool consumption |
| **Artifacts** (`~/.ship/artifacts/`) | Binary or large content; content-addressable; output-shaped | Anything you'd expect to query, edit, or publish |

**Disk-backed config is publishable.** `<project>/.ship/<vector>/`
is git-tracked when the project is git-backed; that's how teams
share. We do not split `.ship/` across two on-disk locations to dodge
git tracking. If a piece of state is sensitive, it goes in a secrets
store (planned), not a stealth gitignore. The publishability of
`.ship/` is a feature, not a problem to route around.

## Collections are a feature, not a primitive

Collections are great. They give users custom schemas + queryable
rows + view-rendering. Ship dogfoods them with first-party schemas
shipped in the binary: `specs`, `notes`, `feedback`, `annotations`,
`artifacts`. **The runtime does not depend on collections to
function.** Loss of data.db doesn't stop the daemon; loss of the
events table does.

Things that look like collections-shaped but aren't:

- Pipelines, agents, skills, rules, MCP servers, views, triggers,
  permissions. These are **platform schemas** Ship defines; users
  author *instances* against them. They live on disk because:
  - external harnesses (claude-code, codex) read them as files;
  - hot-reload via fs-watch is the load-bearing mechanism;
  - git is a perfectly fine integration for sharing them.

- Settings. KV is shaped for them.

- Integration state (MCP servers configured, OAuth connections,
  webhooks registered). Should be event-sourced — see below.

## Five primitives

Workspace as a runtime primitive retired (see
`.artifacts/specs/workspace-retirement/`). The runtime now has five
concepts, each with one job:

1. **Project** — a directory with `.ship/` inside it (or no
   directory, for headless / consumer flows that haven't picked one).
   A tagging primitive: views accept it as a filter, apps register
   against it. Not a context-switch — multiple projects can be
   visible at once. Path-hash keyed in `kv_state` namespace `project`
   via `runtime::projects`. Default `personal` project lives at
   `~/.ship/`.

2. **Actor** — a running thing. Owns isolation: `cwd`, permissions,
   transport, lifecycle, MCP tool gate. Supervisor-managed. Already
   the unit of "what's running"; no parallel "active workspace"
   concept any more. Per-actor scope replaces the old MCP
   `active_workspace` singleton.

3. **Worktree** — a dev-flow attachment. A pairing of branch + path
   (+ optional tmux session) that some actors opt into. **NOT a
   runtime primitive** — kv-backed via `runtime::worktrees`,
   maintained by the workspace.* event handlers. Consumer flows that
   don't need an isolated checkout simply don't have a worktree row.

4. **Session** — analytics rollup over actor activity. Has a `kind`
   discriminator (`pipeline_step | chat_thread | direct_attach`) and
   context dimensions (`project_id`, `pipeline_run_id?`,
   `chat_thread_id?`, `branch?`). Telemetry (tokens, files_changed,
   exit_reason, etc.) is inline; no separate "record" table. There
   is no live session row — "what's running" is an actor question;
   "what happened" is a session query.

5. **Adapter** — in-process module that talks to the outside world
   (git, tmux, providers, future containers) and emits events into
   the runtime log. Adapters never write runtime projection tables.
   Today: `runtime::git`, `runtime::supervisor`'s tmux + executor
   transports. The eventual `ExecutionContext` port (sketched in
   `.artifacts/specs/workspace-retirement/decisions.md`) will unify
   the worktree-management subset; deferred until a second adapter
   is concrete.

### Why this set

Sessions are analytics, not stateful entities. Worktrees are an
attachment, not a primitive. Pipelines are one entry-point of many
(chats and direct CLI attach are others). Actors are the unification
— every spawn produces an actor; everything else is a dimension on
that actor's activity. Project is the tag that lets us scope and
filter without making it a context-lock.

`workspace` and its associated MCP tools (`create_workspace`,
`activate_workspace`) survive in the user-facing surface only as a
naming vestige; the runtime no longer has the concept. A follow-up
arc may rename them.

## Integrations are event-sourced

MCP servers, OAuth connections, webhooks, future cloud connectors —
**integrations are event-sourced**, not bespoke tables, not
collections.

```
integration.registered     ← user adds an MCP server
integration.enabled        ← state change
integration.disabled
integration.credential_rotated
integration.deregistered
```

A `runtime::projections::integration` projects these into a current-
state table the registry can query. Same shape as `actors`,
`sessions`, `jobs`. Uniform. Audit-able. Replay-able.

The current `managed_mcp_state` (now in KV) is a step toward this but
not the destination. KV was a smaller wrong than a dedicated table;
events + projection is the right shape.

## What `--dev-worktree` is

Dev workflow plumbing — multiple Ship-dev worktrees on one machine,
each running their own daemon on hash-derived ports. The flag was
renamed from `--workspace` to make its dev-only nature obvious. For
users, the daemon serves the user; projects are selected per-view via
the registry. cwd-discovery is the fallback for first-launch UX.

## What `.ship/` is

Project-scoped, publishable, git-trackable when the user wants it.
Single on-disk location per project (no `.local` sibling, no
`~/.ship/library/projects/<id>/` shadow). Outputs (artifacts) live
elsewhere because they're not publishable; sensitive values live in
secrets (planned) because they need encryption + scoped read.

## What `~/.ship/` is

The daemon's home for one user. Holds:

- `library/` — user-scope authoring (cross-project personal
  pipelines, skills, agents, etc.)
- `machine/` — daemon runtime state (db, logs, sockets, daemons/)
- `platform.db` — events + projections + KV
- `data.db` — collections
- `artifacts/` — content-addressed blob outputs

Nothing in `~/.ship/` is project-scoped except artifacts (which are
addressed by project_id, not directory).

## Decision rules

When adding new runtime state, walk this:

1. **Is it a state transition the kernel should know about?**
   → Event + projection. Don't add a bespoke table.

2. **Is it a setting the user authored?**
   → Disk if it's part of `.ship/<vector>/` and Ship-defined.
   → KV if it's runtime-mutable, namespaced, transient.

3. **Is it user content with a custom schema?**
   → Collection. First-party schemas register at startup;
     user-defined schemas land via `.ship/collections/<id>.jsonc`.

4. **Is it derived from other state?**
   → Don't store unless the recompute cost is real. If you do
     store, store it as a projection from an event, not a hand-
     written row.

5. **Is it a binary blob?**
   → Artifact. Content-addressed. Scoped run/project/user.

6. **Is it secret?**
   → Secrets store (planned). Don't smuggle it via env vars or KV
     without encryption.

If none of those fit cleanly, it's probably not a thing the runtime
needs to own.
