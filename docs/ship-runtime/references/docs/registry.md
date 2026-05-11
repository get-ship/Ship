---
group: Runtime
order: 16
title: Registry
description: Dependency resolver and content-addressed package cache. Install, fetch, resolve, verify, lockfile tracking.
audience: internal
---

# Registry

`crates/core/runtime/src/registry/`. Ship v0.1's dependency resolver plus a content-addressed cache at `~/.ship/cache/`. Packages are git repos at specific commits; the cache stores extracted content keyed by hash.

```
registry/
  mod.rs               — public API
  install.rs           — resolve_and_fetch (orchestrator)
  install_helpers.rs   — load_lock, fetch_and_store, write_lock_atomic
  fetch.rs             — fetch_package_content (git clone)
  resolver.rs          — resolve_version (git tag → commit), alias resolution, remote ref listing
  constraint.rs        — VersionConstraint parser
  hash.rs              — SHA-256 tree/file/export hashing
  cache.rs             — PackageCache (content-addressed)
  verify.rs            — hash verification on install
  tracking.rs          — anonymous install telemetry
  init_deps.rs         — default dependencies seeded on `ship init`
  types.rs             — ShipManifest, ShipLock, LockedPackage (stubs; compiler crate will eventually own these)
```

## Public API

```rust
pub use cache::{CachedPackage, PackageCache};
pub use constraint::{VersionConstraint, parse_constraint};
pub use hash::{ExportHashes, compute_export_hashes, compute_file_hash, compute_tree_hash};
pub use init_deps::{DEFAULT_INIT_DEPS, SeedResult, seed_default_dependencies, try_install_init_deps};
pub use install::{InstallOptions, InstallResult, resolve_and_fetch};
```

## Install flow

`install::resolve_and_fetch(manifest, lock, options) -> InstallResult`:

1. **Resolve** each dependency's version constraint to a specific commit (`resolver::resolve_version`).
2. **Fetch** the package content from git into the cache (`install_helpers::fetch_and_store` → `fetch::fetch_package_content`).
3. **Verify** content hashes match expectations (`verify` module).
4. **Write lock** atomically via `install_helpers::write_lock_atomic` if `should_write_lock` says it changed.
5. **Track** the install anonymously if telemetry is enabled (`tracking::track_install`).

Returns `InstallResult { installed, updated, unchanged, lock_path, ... }` summarizing the result.

## PackageCache

`cache::PackageCache` at `~/.ship/cache/`. Content-addressed: each package version extracted under a directory keyed by its SHA-256 tree hash. Multiple manifests can reference the same commit and share the one cached copy.

- `CachedPackage` — a resolved, extracted package on disk.
- `PackageCache::open(cache_dir)` — opens an existing cache or creates it.
- `PackageCache::has(hash)` — presence check before fetch.
- `PackageCache::store(hash, content_dir)` — move/copy a freshly-fetched package into the cache.
- `PackageCache::path(hash)` — resolve a hash to a filesystem path.

Cache never evicts today. Cleanup is manual (`rm -rf ~/.ship/cache`).

## Version resolution

`constraint::parse_constraint(s)` parses strings like:

- `"1.2.3"` — exact version.
- `"^1.2"`, `"~1.2.3"` — semver range.
- `"branch:main"`, `"tag:v1"` — direct ref pointers.
- `"commit:abc123"` — exact commit.

`resolver::resolve_version(git_url, constraint)` calls `git ls-remote` via `list_remote_refs`, picks the best match for the constraint, returns the exact commit SHA. Alias resolution (`resolver::resolve_alias`) converts shortcuts like `ship-labs/foo` into full git URLs.

## Hashing

`hash.rs`. SHA-256 throughout. Deterministic by construction — file order, line-endings, symlink handling all canonicalized.

- `compute_file_hash(path)` — single file.
- `compute_tree_hash(root)` — walk the tree, hash files in sorted order, combine.
- `compute_export_hashes(...)` — per-export hashes used during compiler emission to detect drift.

## Lockfile

`types::ShipLock` — the lockfile shape. Today these are stubs pending the compiler crate exposing its own equivalents (per the mod.rs comment). `ShipManifest`, `LockedPackage` are the input-manifest and per-package record.

- `parse_ship_lock(content)` — deserialize from the on-disk file.
- `serialize_ship_lock(&lock)` — re-serialize, sorted deterministically.
- `load_lock_if_present(path)` — convenience to return `Option<ShipLock>`.
- `write_lock_atomic(path, &lock)` — write-temp-then-rename.
- `should_write_lock(existing, new)` — only write if changed. Avoids mtime churn when nothing actually changed.

## init_deps

`init_deps::DEFAULT_INIT_DEPS` — packages seeded on `ship init`. `seed_default_dependencies(ship_dir)` writes them into the project's `ship.jsonc`. `try_install_init_deps(ship_dir)` runs `resolve_and_fetch` against the seeded manifest — best-effort; failures are logged but don't block init (offline users can still init projects).

Returns `SeedResult` indicating how many deps were seeded vs already present.

## Tracking

`tracking.rs`. Anonymous telemetry: on successful install, POST a minimal event (package path + offline flag) to the registry index. No user identifier, no repo info. `percent_encode` handles URL-safe encoding for the package path segment. `track_install(package_path, offline)` is fire-and-forget — network failure doesn't fail the install.

## Invariants

- Cache is content-addressed. Same commit, same hash, same directory. No versioned-directory-per-call tree.
- Install is idempotent. Re-running `resolve_and_fetch` with an unchanged manifest and lockfile is a no-op except for hash re-verification.
- Lockfile writes are atomic. Interrupted installs never leave a half-written lock.
- Verification happens post-fetch. A corrupt or tampered cache entry fails verification before being used.
- Telemetry is fire-and-forget. Network failures never block install success.

## Related

- [Platform DB](/runtime/platform-db/) — `registry` module doesn't use the DB directly; it's purely on-filesystem.
- [Compiler](/compiler/) — eventual owner of `ShipManifest` / `ShipLock` types (stubs here today).
