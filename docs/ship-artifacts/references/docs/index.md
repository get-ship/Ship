---
group: Artifacts
order: 1
title: Artifacts
description: Ship's file API — the storage primitive for agent-produced content. Three-scope model (ephemeral, project, global), with cloud-backed storage as the long-term plan.
audience: internal
---

# Artifacts

An artifact is a file Ship manages. Agents produce them (HTML canvases, generated documents, compiler output, diffs, logs). Users consume them (annotations, review flows, downloads). Ship owns where they live, how long they last, and — in cloud mode — how they sync.

The artifacts model is the right framing for Ship's filesystem surface. Today's implementation is scattered across several subsystems that predate the framing; this page describes the **intended** shape and points at what exists vs what's aspirational.

## The three scopes

| Scope | Lifespan | Location (local) | Backing (cloud) | Examples |
|---|---|---|---|---|
| **Ephemeral** | Job / session / worktree lifetime | Inside the worktree or per-session temp | Not synced | `.artifacts/pipeline/NN-step.md` at dispatch; scratch outputs an agent produces mid-job; tmux scrollback |
| **Project** | Lifetime of the project's `.ship/` | `{project}/.ship/files/` (planned: `xx/<id>` content-addressed layout) | Per-project S3 prefix | Canvas artifacts, generated design docs, files a user wants to keep with the project |
| **Global** | Lifetime of the user's `~/.ship/` | `~/.ship/files/`, `~/.ship/cache/`, `~/.ship/trash/` | User-level S3 prefix | Registry package cache, cross-project templates, user-level skill assets |

"Lifespan" is the storage contract, not a hard TTL. Ephemeral artifacts GC with their owner (session ends → session artifacts cleaned up; worktree removed → `.artifacts/` goes with it). Project and global persist until the user deletes them or Ship's trash-retention policy expires them.

## The intended API

A unified files API — same call shape regardless of scope or storage backend:

```
artifact.put(scope, id, bytes, metadata?) -> ArtifactRef
artifact.get(ref) -> Bytes
artifact.list(scope, filter?) -> Vec<ArtifactRef>
artifact.delete(ref)
artifact.move(ref, new_scope)   // promote ephemeral → project, etc.
```

Where:

- `scope: Ephemeral(session_id) | Project(project_dir) | Global`
- `id` is user-provided or Ship-generated. Content-addressed by default (SHA-256) so the same bytes dedupe across artifacts.
- `metadata` carries kind, mime, owner, created_at, plus free-form tags.

Consumers don't care whether bytes live in `~/.ship/files/`, `.ship/files/` inside the project, or in S3. The ref is stable; the storage is swappable.

**None of this API is implemented as a single module today.** Current code does most of it piecemeal. See "Current state" below.

## Current state

### What exists

- **`agent_artifact_registry` table** in `platform.db`. Content-hashed index over compiled skills/rules/MCP servers. Three `kind` values: `mcp`, `skill`, `rule`. Populated by `sync_agent_artifact_registry`. Internal bookkeeping; not an API for users.
- **`workspace_session_record` table**. Per-session metrics + change summaries (files_changed count, lines added/removed, gate result). Published on session close via `session.recorded`.
- **`.artifacts/pipeline/*` files**. Orchestrator writes per-step spec bodies here at dispatch. Ephemeral-scoped by nature (live with the worktree).
- **Trash** (`runtime::workspace::trash`). Soft-delete for workspace files. 24h retention under `{global_dir}/trash/{workspace}/{timestamp}/`. Workspace-scoped but the implementation is global.
- **Registry cache** (`runtime::registry::cache::PackageCache`). Content-addressed at `~/.ship/cache/`. Closest existing code to the intended `Global` scope artifact shape.

### What's designed but not built

- **`files` collection**. Proposed in the canvas-lane-01 spec as the storage for canvas artifacts — entries in a built-in collection on `data.db`, bytes at `.ship/files/<id[0:2]>/<id>`. This pattern, generalized, is the project-scope implementation. Spec exists; collection + blob layer not wired.
- **Unified `artifact` module** in runtime. No `runtime::artifacts::*` module today. Once the API lands, it owns the scope routing and the backend dispatch.
- **Cloud S3 backend**. `runtime::sync` (feature-gated `unstable`) has sync-cursor primitives but no artifact-specific sync path. S3 is the target; the abstraction is designed so the local → cloud transition is a backend swap.
- **Promotion between scopes**. Moving an ephemeral artifact to project scope (agent produces a draft, user keeps it) has no API today — a copy via the files API is the sketch.

### Where "artifacts" appears in the code today

The word is overloaded. To read code confidently:

- `agent_artifact_registry` — the platform.db table. Index of compiled assets by content hash.
- `artifact_events.rs` in `runtime::workspace` — emits events when a session produces an artifact. Today used primarily for session records.
- `.artifacts/` directory — project-root conventional dir for job specs, pipeline bodies, working notes. Gitignored by default.
- `session artifacts` — per-session outputs tracked in `workspace_session_record`.
- `canvas artifacts` — canvas HTML authored by agents via `view.open` → stored in the session state of the view session registry today (moving to `files` collection per the canvas-lane spec).

None of these is wrong, but they're not one mechanism. The target is: one artifact concept, one API, one set of events (`artifact.created`, `artifact.moved`, `artifact.deleted`), with scope + backend as implementation details.

## Gitignore contract

Artifacts that Ship owns the lifecycle of are gitignored:

- `.ship/files/` — project-scope artifact storage (once implemented).
- `.artifacts/` — ephemeral (pipeline dispatch bodies, working notes).
- `~/.ship/` — entirely outside any repo.

Artifacts that users want committed (e.g. a generated document they consider a source file) should be promoted out of Ship's managed paths. Ship doesn't manage committed files.

## Cloud sync (aspirational)

Long-term plan: project-scope artifacts back to a per-project S3 prefix; global-scope to a user-level prefix. Sync is event-driven — `artifact.created` / `artifact.moved` / `artifact.deleted` trigger sync cursors that push/pull bytes and metadata.

The contract an agent or user sees stays the same regardless of backend. A `Project` artifact at `.ship/files/<id>` in the local case is `s3://ship-project-xyz/<id>` in the cloud case; both resolve through the same ref.

No S3 code exists today. The design is that adding it doesn't break the local API.

## Events

Once the API lands, the event taxonomy is:

| Event | Emitted when |
|---|---|
| `artifact.created` | New artifact written. Payload: ref, scope, kind, size, hash. |
| `artifact.updated` | Metadata change (rename, tag). Bytes don't change — new bytes = new artifact. |
| `artifact.moved` | Scope promotion (e.g. ephemeral → project). |
| `artifact.deleted` | Artifact removed (or trashed, depending on policy). |

Subscribers: projections (listing views), sync (cloud backend), triggers (the trigger system's `collection.*` actions extend to `artifact.*`).

## What to do today

Because the unified API doesn't exist yet:

- **Canvas-style outputs**: use the `view.open` / `view.patch` / `view.append` MCP tools. Session-scoped today; will migrate to the files collection per canvas-lane-01.
- **Per-session outputs**: session record fields + session-progress events. Surfaces in Studio automatically.
- **Per-project saved files**: write to the project's `.ship/` with a path you choose. Gitignore accordingly. Migrate when the API lands.
- **Global caches**: use `runtime::registry::cache` for anything that's a resolved-package shape; otherwise write directly to `~/.ship/` with care.

This will get simpler.

## Related

- [Collections](/reference/collections/) — the `files` collection (planned) backs project-scope artifacts.
- [Views](/reference/views/) — canvas views produce artifacts today via the session registry; migrating to the files API.
- [Registry](/reference/runtime/registry/) — the closest existing global-scope content-addressed storage (`PackageCache`). A pattern to generalize.
- [Sync](/reference/runtime/daemon/) — sync module will handle the cloud backend when it lands.
