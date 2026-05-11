---
group: Artifacts
order: 2
title: Artifacts Reference
description: REST API, MCP tools, scope routing, metadata schema, permissions, events, GC.
audience: internal
---

# Artifacts Reference

An artifact is a managed file: bytes with metadata. Agents produce them; users consume them. Ship owns storage paths, GC, content addressing, and scope routing.

## Three scopes

| Scope | Lifespan | Path (local) | Typical use |
|---|---|---|---|
| `run` | TTL-based (default 24 h) | `{global}/artifacts/{project}/runs/{run_id}/{id[0:2]}/{id}` | Step outputs, scratch, logs |
| `project` | Until deleted | `{global}/artifacts/{project}/blobs/{id[0:2]}/{id}` | Canvas docs, design files, persistent outputs |
| `user` | Until deleted | `{global}/artifacts/users/{user_id}/{id[0:2]}/{id}` | Cross-project personal storage |

Content addressing: SHA-256 hex. Same bytes at the same scope path share a blob file; a new metadata row is still written.

## REST API

All routes require a valid daemon token.

| Method | Path | Description |
|---|---|---|
| `POST` | `/api/artifacts` | Upload (multipart). Required: `scope`. For `run`: also `run_id`. |
| `GET` | `/api/artifacts/:id` | Download bytes. |
| `HEAD` | `/api/artifacts/:id` | Metadata headers only (no body). |
| `GET` | `/api/artifacts/:id/metadata` | JSON metadata record. |
| `PATCH` | `/api/artifacts/:id` | Update `name`, `owner_actor`, `expires_at`. Bytes are immutable. |
| `DELETE` | `/api/artifacts/:id` | Delete blob + metadata. |
| `POST` | `/api/artifacts/:id/promote` | Promote to new scope. Body: `{ "target_scope": "project" \| "user" }`. |
| `GET` | `/api/artifacts` | List with filters. |

### List filters

`scope`, `project_id`, `run_id`, `user_id`, `mime`, `collection_id`, `content_hash`, `owner_actor`, `expires_at`, plus `limit`/`offset`/`sort`.

## MCP tools

| Tool | Description |
|---|---|
| `artifact_upload` | Upload bytes (base64-encoded). Returns `{ id }`. |
| `artifact_get` | Retrieve record + bytes. |
| `artifact_list` | List with optional scope and filter JSON. |
| `artifact_delete` | Delete by ID. |
| `artifact_promote` | Promote to new scope. |

## Metadata schema

Stored in the built-in `artifacts` collection on `data.db`.

| Field | Type | Description |
|---|---|---|
| `id` | ULID string | Stable artifact ID. |
| `name` | string | File name. |
| `mime` | string | MIME type. |
| `size` | integer | Bytes. |
| `path` | string | Relative blob path (backend-opaque). |
| `scope` | string | `run` \| `project` \| `user` |
| `project_id` | string? | Set for `run` and `project` scopes. |
| `run_id` | string? | Set for `run` scope. |
| `user_id` | string? | Set for `user` scope. |
| `content_hash` | string | SHA-256 hex. |
| `expires_at` | ISO-8601 string? | GC deadline. Run-scope gets a default TTL if unset. |
| `owner_actor` | string? | Actor that created the artifact. |
| `collection_id` | string? | Optional parent collection. |
| `item_id` | string? | Optional parent collection item. |
| `created_at` | ISO-8601 string | Creation timestamp. |

## Permissions

| Scope | Grant required |
|---|---|
| Read run artifacts | `ReadArtifactsRun` |
| Write run artifacts | `WriteArtifactsRun` |
| Delete run artifacts | `DeleteArtifactsRun` |
| Read project artifacts | `ReadArtifactsProject` |
| Write project artifacts | `WriteArtifactsProject` |
| Delete project artifacts | `DeleteArtifactsProject` |
| Read user artifacts | `ReadArtifactsUser` |
| Write user artifacts | `WriteArtifactsUser` |
| Delete user artifacts | `DeleteArtifactsUser` |
| Promote between scopes | `PromoteArtifact` |

Pipeline agents (worker/lead/commander roles) receive `Read/Write/DeleteArtifactsRun` and `ReadArtifactsProject` automatically.

## Events

| Event | Payload |
|---|---|
| `artifact.created` | `id`, `scope`, `name`, `mime`, `size`, `content_hash`, `owner_actor`, `project_id`, `run_id` |
| `artifact.updated` | `id`, `scope`, `changed` (map of updated fields) |
| `artifact.moved` | `id`, `from_scope`, `to_scope` |
| `artifact.deleted` | `id`, `scope`, `project_id`, `run_id` |

Each mutation also emits a `collection.item_*` event for the underlying metadata row.

## Settings

In `.ship/ship.jsonc`:

```jsonc
"artifacts": {
  "run":     { "ttl_hours": 24, "size_max_mb": 50 },
  "project": { "size_max_mb": 100, "total_max_gb": 5 },
  "user":    { "size_max_mb": 100, "total_max_gb": 10 },
  "mime_allowlist": null,
  "mime_denylist":  ["application/x-msdownload", "application/x-executable"],
  "sync": { "enabled": false }
}
```

All values have defaults. The daemon validates the block at startup; negative sizes or zero TTLs are rejected.

## GC

Run-scope artifacts with `expires_at < now` are deleted on daemon startup and every hour. `expires_at` is set at upload time; callers can override per-upload. Default TTL is `run.ttl_hours` from settings (24 h).

## Backend abstraction

The `ArtifactBackend` trait in `crates/core/runtime/src/artifacts/backend.rs` decouples storage from logic. Phase 1 ships `LocalFsBackend`. Cloud (R2/S3) swap is phase 3 — no API changes required.

## `compiled_asset_cache` (formerly `agent_artifact_registry`)

The internal table that indexes compiled skills, rules, and MCP configs is `compiled_asset_cache` in `platform.db`. It is **not** an artifact-store table — it's a compiler cache for the daemon's internal asset pipeline. It was renamed from `agent_artifact_registry` to reduce confusion with the artifact storage described on this page.

## Related

- [Collections](/collections/) — the `artifacts` collection stores metadata rows. `File` field types in collection schemas resolve to artifact IDs.
- [Pipelines](/pipelines/) — step I/O via artifact produce/consume/promote is a phase-2 deliverable.
- [Views](/views/) — canvas views use project-scope artifacts via `artifact_upload`.
