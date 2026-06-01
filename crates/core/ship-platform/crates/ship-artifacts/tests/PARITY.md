# ship-artifacts — parity report

## Summary

| | Count |
|---|---|
| PORT (tests added in parity.rs) | 40 |
| IGNORED/GAP (in parity.rs, `#[ignore]`) | 0 |
| OBSOLETE (old plumbing gone, omitted) | 9 |

Old baseline: **~49 old tests** across integration_backend.rs, integration_gc.rs,
integration_promote.rs, integration_store.rs.

New baseline (before this job): **9 tests** in behavior.rs.

New baseline (after this job): **9 (behavior.rs) + 40 (parity.rs) = 49 tests**.

---

## Behavioral changes (no gaps, but operator awareness required)

| Change | Detail |
|--------|--------|
| `patch` return type | Old: `Option<ArtifactRecord>` (None if missing). New: `Result<ArtifactRecord, ArtifactError>` (Err(NotFound) if missing). Tested as `parity_patch_missing_returns_err`. |
| `promote` added `retained: bool` param | Old promote had no retained flag. New: `retained=true` clears `expires_at` (artifact becomes durable). Tested as `parity_promote_retained_clears_expires_at`. |
| GC scope | Old GC was called with `(&ship_dir, &mut data, &kernel)`. New GC: `collect_expired(&svc)` — scoped to the service's `SpaceContext` space. Same behavioral contract. |

---

## PARITY-GAPs

None. All meaningful old behaviors are covered by the PORT set.

---

## OBSOLETE tests (not ported)

- `artifacts_backend_project/run/user_scope_blob_path_layout` — `LocalFsBackend` and
  `blob_path()` are `pub(crate)`; behavior ported via `record.path` field assertions
  (`parity_project/run/user_scope_blob_path_contains_*`).

- `artifacts_backend_sha256_hex_known_vector`, `artifacts_backend_sha256_hex_is_lowercase`,
  `artifacts_backend_sha256_hex_deterministic` — `sha256_hex` is `pub(crate)`; behavior
  ported via `content_hash` field on stored records
  (`parity_sha256_known_vector_via_content_hash`, `parity_content_hash_is_lowercase`,
  `parity_content_hash_is_deterministic`).

- `artifacts_backend_invalid_scope_row_errors_loudly` — old test used internal
  `create_item` from old store module; ported via `DataStore::insert` through the public
  port (`parity_invalid_scope_row_no_silent_fallback`).

- Old fixture helpers (`make_kernel` using `KernelRouter::new`, `setup()` returning
  `DataService` separately) — replaced by new port-based fixtures matching behavior.rs
  pattern (`SpaceContext` + `KernelLog`).
