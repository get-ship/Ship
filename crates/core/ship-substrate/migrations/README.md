# platform.db migrations

This directory holds the sqlx migrations for `platform.db`. sqlx runs
every file here in lexicographic order on `DataService::open` /
`ensure_db_at` and tracks applied versions in `_sqlx_migrations`.

## Files

- `0001_schema.sql` — squashed baseline. Defines the current schema in
  one file. Pre-launch you can keep editing this in place + wipe DBs;
  once external installations exist it freezes.
- `YYYYMMDDHHMM_kebab_description.sql` — every schema change after the
  baseline. UTC timestamp prefix, monotonic relative to the newest
  existing file. Lowercase ASCII + digits + `-`/`_` in the slug.

## Conventions

1. **SQL DDL only.** Idempotent (`IF NOT EXISTS` / `IF EXISTS`).
   sqlx wraps each file in a transaction.
2. **No Rust "migrator" code.** Don't write helpers that read from
   the old shape and write to the new shape, then live in the codebase
   forever guarding against re-running. If a schema change needs data
   reshape, do it in SQL inside the migration. If that's not feasible,
   reset the affected tables — pre-launch makes that cheap.
3. **No data.db here.** Per-collection `data.db` schemas live in
   `runtime::collections::builtin::register_builtin_schemas`; do not
   mix them in.

The CI guard (`scripts/lint/check-migration-naming.sh`, wired into
`just check-migrations`) enforces the active naming pattern.

## Testing locally

```
cargo test -p runtime
just check-migrations
```

Smoke against a real sqlite file:

```
rm -f /tmp/ship-mig.db
sqlx migrate run \
    --source crates/core/runtime/migrations \
    --database-url sqlite:///tmp/ship-mig.db
sqlite3 /tmp/ship-mig.db ".schema"
```

## Worked example

```sql
-- 202604271630_add_agent_fingerprint.sql
ALTER TABLE actors
    ADD COLUMN fingerprint TEXT;

CREATE INDEX IF NOT EXISTS idx_actors_fingerprint
    ON actors(fingerprint);
```
