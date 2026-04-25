---
title: "Artifact Storage-Key Opacity"
description: "ADR: ArtifactRecord.path is an opaque storage key, not a filesystem path; a StorageBackend trait abstracts read/write so callers stay clean across local FS, in-memory tests, and future sync targets."
group: ADRs
order: 40
audience: internal
status: accepted
date: 2026-04-25
supersedes: []
tags: [artifacts, storage, contract-freeze]
---

# Artifact Storage-Key Opacity

## Context

Audit Item 19 — artifact records carry a `path` field that today is a filesystem path under `.ship/artifacts/` (or `~/.ship/library/...` after the home-layout migration). Several callers treat `path` as parseable filesystem path:

- Joining it with local prefixes to build absolute paths.
- Splitting on `/` to extract filename or directory components.
- Passing it directly to `std::fs::read` / `tokio::fs::read`.
- Rendering it in UI as a clickable path.

This is wrong even before any cloud story. It's a hygiene and testability problem:

1. **In-memory test backends become impossible.** A test that wants to write an artifact, run something, and read it back has to either touch the real filesystem or every caller has to be filesystem-agnostic. The latter is what we want; the former leaks state across tests.
2. **Sync changes the addressing.** Cross-machine sync (laptop → desktop) can't rely on identical filesystem paths. Path-as-key vs path-as-string-on-disk is the distinction.
3. **Callers parsing `path` lock in the on-disk layout as a contract.** If the artifact directory ever moves (e.g. when home-layout consolidated `library/`/`machine/`), every parser breaks.

The fix is to lift `path` from "filesystem path" to "opaque key the storage layer understands." Most callers don't need to know where bytes live; they need to ask the storage layer for them.

This is local-first hygiene first. Cloud sync, when it ships, gets the abstraction for free.

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

A lint rule (`runtime-raw-artifact-path`) flags filesystem operations on a field named `path` of an `ArtifactRecord` outside the storage module — same enforcement pattern as `runtime-raw-ship-path` in `home-layout-and-scope-convention`.

### 2. `StorageBackend` trait abstracts blob I/O

```rust
// crates/core/runtime/src/storage/mod.rs
pub struct StorageKey(String);

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn read(&self, key: &StorageKey) -> Result<Bytes, StorageError>;
    async fn write(&self, key: &StorageKey, data: Bytes) -> Result<(), StorageError>;
    async fn delete(&self, key: &StorageKey) -> Result<(), StorageError>;
    async fn exists(&self, key: &StorageKey) -> Result<bool, StorageError>;
    // Streaming + range support added when needed; not v1.
}
```

Concrete backends:

- **`FsBackend`** (default) — keys map to files under `~/.ship/library/artifacts/`. The mapping is implementation detail; callers never compute it.
- **`InMemoryBackend`** (tests) — `HashMap<StorageKey, Bytes>`. Drop-in for unit tests that need artifact write/read cycles without touching disk.
- **Future backends** (object storage, sync target, etc.) — additive. The trait is the contract; new backends slot in.

`ArtifactService` (or whatever owns artifact CRUD) holds an `Arc<dyn StorageBackend>` and is the only code path that calls into it. Callers see records with `path`, never see the backend.

### 3. Content-addressing is not mandated v1

The `StorageKey` is opaque from the caller's perspective but the *backend* may choose its key format:

- `FsBackend` could use UUIDs, hashes, or whatever fits.
- A future content-addressed backend could derive keys from `sha256(content)` and gain dedup + integrity for free.

Mandating content-addressing now would force a migration if we ever decide we want filename-readable keys (e.g., for human-debuggable artifact directories). Leaving the choice to the backend keeps options open.

## Consequences

### Positive

- **Tests get an in-memory backend.** Artifact-touching tests stop hitting the filesystem; faster, isolated.
- **Sync is straightforward.** A sync layer reads keys + bytes from one backend and writes to another. No filesystem-path translation.
- **Callers stop caring about layout.** Moving artifacts between directories (e.g. when home-layout reorganized) doesn't break any caller.
- **Future backends are additive.** Object storage, S3-compatible, content-addressed — all swap in without caller changes.

### Negative

- **Audit cost.** Every existing artifact-`path` reader needs review. Most are read-only display, easy. A few may parse paths to derive context — those need refactoring.
- **One more abstraction.** Adds a trait + a default backend implementation. The cost is small but real; engineers reading code see indirection where there used to be `fs::read`.
- **Lint rule maintenance.** `runtime-raw-artifact-path` follows the same shape as the existing raw-path lint; not free, but not novel either.

## Alternatives Considered

### Keep `path` as filesystem path; add helpers

Helper functions that wrap `fs::read(record.path)` and centralize the "this is a filesystem path" assumption. Rejected because it bakes the FS assumption into the helper signature; future non-FS backends still need a refactor. Doesn't solve the test isolation problem.

### Derive a separate `key` field; deprecate `path`

Add `key: StorageKey` and leave `path` for backward compat. Rejected because pre-launch we have no backward-compat constraint, and two fields create permanent ambiguity about which is canonical.

### Mandate content-addressing now

Force every backend to use `sha256` keys. Rejected: locks out filename-readable schemes for human-debuggable artifact dirs. Better to leave the backend free.

## Migration

1. Add `StorageBackend` trait + `FsBackend` default in `crates/core/runtime/src/storage/`.
2. `ArtifactService` adopts `Arc<dyn StorageBackend>`.
3. Audit existing artifact-path readers (likely under `apps/shipd/src/`, `apps/ship-cli/src/`); replace `fs::read(record.path)` with `storage.read(&record.path.into())`.
4. Add `runtime-raw-artifact-path` lint rule. Initial run reports current violators; fix them; flip to error.
5. Add `InMemoryBackend` once the trait stabilizes; migrate tests that touched `.ship/artifacts/` directly.

## How to Measure Success

- `rg "\.path\)" apps/ crates/` returning calls into `fs::read` / `File::open` for `ArtifactRecord` fields → zero hits.
- An in-memory backend test exists and is the example of "how to test artifact write/read cycles."
- Adding a new storage backend to `crates/core/runtime/src/storage/` requires no caller changes.
