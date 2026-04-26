---
group: Runtime
order: 3
title: Migrations
description: How to author a platform.db migration — naming, idempotency, the 0001–0006 freeze, and the tests your migration must include.
audience: public
---

# Migrations

Location: `crates/core/runtime/migrations/`. Applied by sqlx in
lexicographic order on `DataService::open` and `ensure_db_at`.

If you are writing a migration, read the directory's `README.md` first —
this page is the companion guide for authoring agents.

## The freeze

`0001_schema.sql` through `0006_drop_platform_user_tables.sql` are
frozen. They shipped to real installations under their current hashes;
editing them would break every deployment that already ran them. The
platform-table-drop job was the final allowed edit to the numeric
series.

From 2026-04-24 onward, every new migration uses a timestamp prefix:

```
YYYYMMDDHHMM_kebab_description.sql
```

Examples:

- `202604251430_add_agent_fingerprint.sql`
- `202605011200_backfill_session_actor_id.sql`
- `202605120915_drop_legacy_projection_cache.sql`

`scripts/lint/check-migration-naming.sh` (run via `just check-migrations`)
fails the build if any non-grandfathered file uses the legacy `NNNN_`
prefix, or if a new file does not match `^\d{12}_[a-z0-9_-]+\.sql$`.

## When writing a migration

1. Pick the current UTC minute for your prefix. Must sort after the
   newest existing file.
2. Make every statement idempotent — `IF NOT EXISTS` / `IF EXISTS`.
   Migrations run on fresh databases in tests and against long-lived
   databases in production; both paths must succeed.
3. One concern per file. Split schema + backfill + index into separate
   timestamped files (same minute is fine if they are truly ordered).
4. Never edit a file that has already landed on `main`. If you need to
   correct it, write a follow-up migration.
5. No `BEGIN` / `COMMIT` — sqlx wraps each file in a transaction.

## Interaction with `DataService::open` vs `ensure_db_at`

Both code paths call the same sqlx migrator against this directory, so
ordering is identical in every context (daemon boot, CLI one-shot,
tests that spin up a temp `SHIP_GLOBAL_DIR`). A migration that assumes
a table exists must come strictly after the migration that creates it;
lexicographic order of the timestamp prefix is the only guarantee.

## Common pitfalls

- **Referencing removed tables.** After `0006` dropped the platform
  user-content tables, do not reference them anywhere — not even in a
  `DROP TABLE IF EXISTS` that is already handled.
- **Non-deterministic data.** Do not insert rows keyed by
  `datetime('now')` or `random()` during migration; tests will diverge
  from production.
- **Mixing `data.db` schema.** Per-collection `data.db` uses a
  separate migration system. This directory is platform-only.
- **Forgetting the index.** When adding a column you will filter on,
  add the index in the same migration (or an adjacent one). The index
  is part of the schema change, not a later optimization.

## Tests your migration must include

Add or extend a test under `crates/core/runtime/tests/` that:

1. Opens a fresh `DataService` in a `tempfile::tempdir()`.
2. Asserts the new table / column / index exists via
   `PRAGMA table_info(...)` or `sqlite_master`.
3. Exercises at least one happy-path insert/select through the new
   shape.

Run:

```
cargo test -p runtime
just check-migrations
just build
```

All three must pass before you hand the work off.
