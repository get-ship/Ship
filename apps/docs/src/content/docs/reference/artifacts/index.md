---
title: Artifacts
description: Ship's three-scope file store — upload, download, promote, and GC for agent-produced files.
---

# Artifacts

The artifact store is Ship's managed file layer. Upload a file, get a stable ID back. Scope controls how long it lives and where it goes.

## Scopes

| Scope | Lifespan | Use when |
|---|---|---|
| `run` | TTL (default 24 h) | Step scratch, transient outputs |
| `project` | Until deleted | Docs, designs, files you want to keep |
| `user` | Until deleted | Cross-project personal storage |

## Upload

```http
POST /api/artifacts
Content-Type: multipart/form-data

scope=project
file=<bytes>
name=report.html
mime=text/html
```

For `run` scope, also include `run_id`.

Returns `{ "id": "<ulid>", ... }`.

## Download

```http
GET /api/artifacts/:id
```

Returns the raw bytes with the artifact's MIME type.

## Metadata

```http
GET /api/artifacts/:id/metadata
```

Returns the full metadata record:

```json
{
  "id": "01J...",
  "name": "report.html",
  "mime": "text/html",
  "size": 14832,
  "scope": "project",
  "project_id": "my-project",
  "content_hash": "abc123...",
  "created_at": "2026-04-22T10:00:00Z"
}
```

## Update metadata

```http
PATCH /api/artifacts/:id
Content-Type: application/json

{ "name": "new-name.html", "expires_at": "2026-05-01T00:00:00Z" }
```

Only `name`, `owner_actor`, and `expires_at` are mutable. Bytes are immutable — upload a new artifact to change content.

## Delete

```http
DELETE /api/artifacts/:id
```

Removes both the blob and the metadata row.

## Promote

Move an artifact to a more persistent scope:

```http
POST /api/artifacts/:id/promote
Content-Type: application/json

{ "target_scope": "project" }
```

Valid promotions: `run → project`, `run → user`, `project → user`. Requires the `PromoteArtifact` permission.

## List

```http
GET /api/artifacts?scope=project&mime=text/html&limit=20
```

Filter parameters: `scope`, `project_id`, `run_id`, `user_id`, `mime`, `collection_id`, `content_hash`, `owner_actor`, `expires_at`, `limit`, `offset`, `sort`.

## MCP tools

Agents use the MCP tools instead of the REST API directly:

| Tool | Description |
|---|---|
| `artifact_upload` | Upload base64-encoded bytes. Returns `{ id }`. |
| `artifact_get` | Retrieve a record and its bytes. |
| `artifact_list` | List with optional scope and filter. |
| `artifact_delete` | Delete by ID. |
| `artifact_promote` | Promote to a new scope. |

## GC

Run-scope artifacts expire automatically. The default TTL is 24 hours, configurable in `.ship/ship.jsonc`:

```jsonc
"artifacts": {
  "run": { "ttl_hours": 24, "size_max_mb": 50 }
}
```

Override per-upload by setting `expires_at` explicitly. The daemon runs GC on startup and every hour.

## Settings

```jsonc
"artifacts": {
  "run":     { "ttl_hours": 24, "size_max_mb": 50 },
  "project": { "size_max_mb": 100, "total_max_gb": 5 },
  "user":    { "size_max_mb": 100, "total_max_gb": 10 },
  "mime_allowlist": null,
  "mime_denylist":  ["application/x-msdownload"],
  "sync": { "enabled": false }
}
```

## Content deduplication

Uploads are content-addressed (SHA-256). Uploading the same bytes twice in the same scope reuses the blob file — you get two distinct artifact IDs pointing to the same content.

## Related

- [Collections](/reference/collections/) — artifact metadata is stored in the `artifacts` collection. `File` field types in collection schemas resolve to artifact IDs.
- [Step I/O](/reference/pipelines/step-io/) — declare what each pipeline step produces and consumes; promote run artifacts to project scope on success.
