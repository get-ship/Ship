---
title: "Artifact Storage-Key Opacity"
description: "ADR: ArtifactRecord.path is an opaque storage key, not a filesystem path; a StorageBackend trait abstracts read/write so cloud can swap filesystem for object storage."
sidebar:
  label: "Artifact Storage-Key Opacity"
  order: 40
---
## Context

Audit Item 19 (`.artifacts/launch-audit/wire-and-storage.md`) landed a *defer-to-Phase-B* verdict on artifact layout with one caveat: document `path` as an opaque storage key before launch. This ADR does that work, and elevates the constraint from docs to contract.

Today the `collection_artifacts` built-in collection (see `collections-as-data-layer`) carries these fields:

```
name, path, mime, size, collection_id, item_id,
scope (run|project|user), project_id, run_id, user_id,
content_hash, expires_at, owner_actor
```

`path` is currently a filesystem path under `.ship/artifacts/` (or the equivalent in `~/.ship/library/` for user-scoped artifacts once `home-layout-and-scope-convention` lands). Callers today occasionally:

- Build filesystem paths by joining `path` with local prefixes.
- Split `path` on `/` to pull out filename vs. directory.
- Assume `path` exists on the local filesystem at all.

Cloud deployment breaks every one of these assumptions. In cloud:

- Blobs live in object storage (S3, R2, GCS, Azure Blob). Keys look like `artifacts/{tenant}/{scope}/{uuid}` or a content-addressed hash — not a filesystem path.
- The daemon writing the artifact and the daemon serving it may be different processes in different regions. Local filesystem is not a shared substrate.
- Presigned URLs, range requests, and multipart uploads are first-class; `std::fs::read(path)` is not even wrong.

The audit flagged this as "shape right; `path` already opaque; cleanup policy is docs." That undersells it. The *shape* is right only if callers stop treating `path` as a filesystem path. If even one caller today parses `path`, the cloud swap is a breaking change.

This ADR locks in the contract now — before v1 launches with a filesystem-assuming caller that has to be retrofitted.

## Decision

### 1. `ArtifactRecord.path` is an opaque storage key

Callers treat `path` as a string identifier. They do not:

- Join it with `.ship/artifacts/` or any other filesystem root.
- Split it on `/` to extract filename or directory components.
- Pass it to `std::fs::read`, `tokio::fs::read`, `File::open`, or any filesystem primitive.
- Assume it is a valid filesystem path.
- Render it in UIs as a clickable filesystem link.

The only permitted operations on `path` are:

1. Store it, index it, and pass it to the `StorageBackend` trait (below).
2. Compare two `path` values for equality (the key identifies a blob).

A lint rule (`runtime-raw-artifact-path`) flags any filesystem operation on a field named `path` of an `ArtifactRecord` outside the storage module, matching the pattern already established by `runtime-raw-ship-path` in `home-layout-and-scope-convention`.

### 2. `StorageBackend` trait abstracts blob I/O

```rust
// crates/core/runtime/src/storage/mod.rs
pub struct StorageKey(String);

pub trait StorageBackend: Send + Sync {
    async fn put(&self, key: &StorageKey, bytes: &[u8]) -> Result<()>;
    async fn get(&self, key: &StorageKey) -> Result<Vec<u8>>;
    async fn stream_get(&self, key: &StorageKey) -> Result<impl AsyncRead>;
    async fn delete(&self, key: &StorageKey) -> Result<()>;
    async fn exists(&self, key: &StorageKey) -> Result<bool>;
    async fn presigned_url(&self, key: &StorageKey, ttl: Duration) -> Result<Option<Url>>;
}
```

`presigned_url` returns `None` on backends that do not support it (filesystem); the caller falls back to streaming through the daemon.

### 3. Filesystem backend is the default; object-store backends are additive

The v1 default implementation is `FilesystemBackend`, rooted at `machine/artifacts/` (following `home-layout-and-scope-convention`'s `library/` + `machine/` split — artifacts are machine-local unless explicitly promoted; scoped artifacts live per-scope under this root).

Cloud deployments configure `S3Backend` (or equivalent). No code path outside `storage/` changes when the backend changes.

Key format is backend-defined:

- `FilesystemBackend`: `{scope}/{year}/{month}/{uuid}-{slug}` — hierarchical for human browsing if someone goes spelunking with `ls`, but never parsed by the runtime.
- `S3Backend`: `ship/{tenant}/{scope}/{uuid}` — flatter, keyed for bucket-prefix-based access control.

Both formats satisfy the contract that `path` is opaque to callers.

### 4. Artifact URLs are daemon-served, not storage-served

The HTTP API exposes `/api/v1/artifacts/{id}` (or `/api/v1/t/{tenant}/artifacts/{id}` in cloud — see the URL shape ADR). The daemon resolves the artifact by ID, looks up the storage key, and:

- If the backend supports presigned URLs and the caller is authorized for a direct-to-storage fetch, redirects or returns the signed URL.
- Otherwise, streams bytes through the daemon.

The client never constructs a URL from `path`. The URL surface is `/api/v1/artifacts/{id}` — full stop. Storage keys never appear in user-facing URLs.

### 5. Scope semantics are preserved; storage backend is independent

The existing `scope` field (`run | project | user`) governs artifact lifecycle and visibility. It does not select a storage backend. A single deployment can host all three scopes on the same backend. Local runs on filesystem for all scopes; cloud runs on S3 for all scopes. Mixed-backend deployments (e.g., user-scope on filesystem, run-scope on object storage) are a Phase B concern.

### 6. Migration for existing callers

Callers that currently build filesystem paths from `ArtifactRecord.path`:

1. Replace `path_buf.join(artifact.path)` with `storage.get(&artifact.key()).await?`.
2. Replace filesystem-relative URL construction in views with `/api/v1/artifacts/{id}`.
3. Replace `fs::remove_file(artifact.path)` with `storage.delete(&artifact.key()).await?`.

An `ArtifactRecord::key(&self) -> StorageKey` accessor wraps the `path` field; callers use the accessor rather than the raw field going forward.

Existing artifacts on disk retain their current paths. The filesystem backend reads them as-is. No data migration is required for v1.

## Consequences

### Positive

- **Cloud swap is a configuration change, not a code change.** Swapping `FilesystemBackend` for `S3Backend` touches one wiring point; every caller above the trait is unchanged.
- **Presigned URLs become possible without refactoring.** Large artifacts (job logs, bundle outputs) stream directly from storage to client in cloud, bypassing the daemon hop.
- **Local stays simple.** Development continues to read files from disk. No minio/localstack dependency to run `shipd`.
- **One lint rule enforces the boundary.** `runtime-raw-artifact-path` prevents regression. New callers cannot accidentally reintroduce filesystem assumptions.
- **Scope model survives intact.** `run | project | user` is backend-agnostic; cleanup policies (artifact TTLs, scope transitions) can be implemented once and apply to both backends.

### Negative

- **Refactor touches every existing caller that parses `path`.** Mechanical but broad. Mitigant: pre-launch, the caller set is bounded.
- **Lint rule is project-specific noise** until a second backend ships. Accepted as the price of preventing regression.
- **Presigned URL semantics leak into the API.** When a daemon returns a redirect to a presigned S3 URL, the client must follow redirects and handle the signed URL correctly. Documented in the HTTP API reference.
- **Streaming-through-daemon fallback costs bandwidth** for backends without presigned URLs. Acceptable for filesystem (no network hop) and for small artifacts on any backend.

## Alternatives Considered

### Keep `path` as a filesystem path, mount S3 via FUSE

Use `s3fs`, `rclone mount`, or `goofys` to mount object storage as a filesystem. Callers keep using `std::fs::read(path)`.

Rejected. FUSE-on-S3 is operationally fragile: it caches aggressively, fails in non-obvious ways on partial network outages, and has per-platform support differences (macOS requires MacFUSE; Linux requires kernel modules). Presigned URLs are unavailable; every fetch goes through the daemon. Performance is worse than either native filesystem or native S3 SDK. The simplicity of "same code path" is illusory — the failure modes are new, not absent.

### Store the backend kind in the record and dispatch at call sites

Add `ArtifactRecord.backend: "fs" | "s3"` and let callers branch:

```rust
match artifact.backend {
    Backend::Fs => fs::read(&artifact.path).await?,
    Backend::S3 => s3.get(&artifact.path).await?,
}
```

Rejected. This is the exact coupling the trait exists to avoid. Every caller becomes backend-aware; adding a third backend (R2, Azure Blob) requires editing every call site. The trait puts the branch in one place.

### Serve artifacts at their storage URL directly

Expose `/s3/bucket/key/...` or similar, letting clients fetch from object storage without daemon involvement.

Rejected. Storage URLs leak backend-specific conventions into the public API. Migrating from S3 to R2 would become a breaking URL change. The daemon-mediated URL (`/api/v1/artifacts/{id}`) is stable across backend swaps; it may *redirect* to a storage URL when presigned URLs are available, but the daemon URL is the contract.

### Use the `name` field as the identifier and drop `path` entirely

Let `name` double as both a human-readable name and a lookup key.

Rejected. `name` is not unique (two artifacts can share a name across scopes or runs), and human-readable names change (rename operations). Storage keys must be immutable and unique for the lifetime of the blob. Separation of concerns: `name` for humans, `path` (as opaque storage key) for the storage layer, `id` for database references.

### Content-address everything; use `content_hash` as the storage key

The existing `content_hash` field could serve as the storage key directly, giving deduplication for free.

Rejected as the *mandated* model — kept as a viable future optimization. Content-addressed storage is a real win for deduplicating identical job outputs, but mandating it means the backend must resolve hash collisions, handle the "blob exists but metadata doesn't" case, and deal with garbage collection across references. Those are solvable problems, but they are additional complexity that the v1 contract does not need to take on. A future backend can key by `content_hash` internally; this ADR requires only that the key format is opaque.

## Migration Plan

1. Land `StorageBackend` trait and `FilesystemBackend` implementation in `crates/core/runtime/src/storage/`.
2. Add `ArtifactRecord::key()` accessor and `runtime-raw-artifact-path` lint rule.
3. Migrate callers: replace `fs::*` on artifact `path` with `storage.*` calls.
4. Update `/api/v1/artifacts/{id}` handler to call `storage.get()` / `storage.stream_get()`.
5. Enable the lint rule at error level; CI fails on raw filesystem operations on artifact paths.
6. (Later, cloud) Ship `S3Backend`; wire via config.

## How to Measure Success

- `rg 'fs::\w+\(.*artifact.*\.path' crates apps` returns zero hits outside `crates/core/runtime/src/storage/`.
- `rg '\.path' crates apps` matches against `ArtifactRecord` are all inside storage module or are field construction.
- Swapping the default backend constant from `FilesystemBackend` to `S3Backend` and supplying S3 config is sufficient to run the daemon against object storage, with no other code changes.
- Artifact reads, writes, deletes, and range fetches all go through `StorageBackend`.
- `/api/v1/artifacts/{id}` is the only externally-quoted URL shape; no URL exposes the raw storage key.

## Cross-references

- Audit: `.artifacts/launch-audit/wire-and-storage.md` — Item 19.
- Audit: `.artifacts/launch-audit/contract-freeze-menu.md` — Item 19 (defer-to-Phase-B with opacity-documentation caveat this ADR fulfills), Item 27 (cloud coherence).
- ADR: `collections-as-data-layer` — `collection_artifacts` is a built-in collection with the `path` field this ADR reinterprets.
- ADR: `home-layout-and-scope-convention` — the filesystem backend's root lives in `machine/artifacts/`; scope-to-location mapping follows the library/machine split.
- ADR: `api-url-shape-and-versioning` — `/api/v1/artifacts/{id}` is the only user-visible artifact URL shape.
