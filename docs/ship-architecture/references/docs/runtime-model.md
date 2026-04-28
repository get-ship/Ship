---
group: Architecture
order: 1
title: Runtime Model
description: The mental model for what Ship's runtime owns — events as the write path, projections in platform.db, collections as a feature, project as a directory with .ship/, workspace as agent sandbox.
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
platform.db tables that are *derived state*, not primary records:

| Projection | Source events | Target table |
|---|---|---|
| `projections::session` | `workspace.session.*` | `workspace_session`, `workspace_session_record` |
| `projections::job` | `job.*` | `jobs` |
| `projections::actor` | `actor.*` | `actors` |

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

## Project ≠ Workspace

**Project = a directory with `.ship/` inside it.** A grouping of work,
git-backed or not. The unit the user thinks about. Identified by a
stable hash of the canonical path (`runtime::projects` registry).

**Workspace = an agent sandbox.** A boundary for an agent's filesystem
access and process lifetime. Multiple workspaces can exist within one
project (one per running agent, in principle). The user mostly
doesn't think about workspaces.

The current `workspace` table conflates these — the same row holds
project-level selections, project-level compile cache, AND
process-attachment metadata, all keyed by branch. That's wrong on two
axes:

1. Project and workspace shouldn't share a row.
2. "Branch" assumes git-as-primitive; many projects don't have a
   branch at all.

The model going forward:

- **Project** is the user-facing primitive. Path-keyed registry.
- **Workspace** is internal isolation. Created per-agent-invocation.
  Has a cwd; has a lifetime; doesn't need to be branch-named.
- The `workspace` table is being decomposed. Its three jobs land
  separately:
  - Project-level **selections** → KV namespace `project.<id>.*`
    (or events if we want history of "user changed default agent").
  - Project-level **compile cache** → projection from a
    `config.compiled` event into a small cache table or in-memory.
  - Per-agent **process attachment** → already in `actors` (the
    actor projection) — workspace was duplicating it.

`workspace.active_agent` / `active_preset` are stale: there are
many agents in a project's scope at once; there is no single
"active" one at the project level. Those columns predate the
multi-agent reality and should be dropped.

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

## What `--workspace` is

Dev workflow plumbing — multiple Ship-dev worktrees on one machine,
each running their own daemon on hash-derived ports. Should be hidden
behind a dev-only flag with semantic naming
(e.g. `--dev-worktree-mode`). For users, the daemon serves the user;
projects are selected per-view via the registry. cwd-discovery is the
fallback for first-launch UX.

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
