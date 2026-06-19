# Per-scope event db migrations

This directory holds the sqlx migrations for the per-scope event dbs
(`runtime.db` / `account.db` / `spaces/<id>/space.db`). The tolerant
runner in `db::migrate` applies every file here in lexicographic order
on `ensure_db_at_path` (the helper `ensure_events_db_at_path` is the
production entry point) and tracks applied versions in
`_sqlx_migrations`.

> `platform.db` is dead (per-scope arc). All per-scope dbs share this
> same migration set so they self-provision on first write.

## Files

- `0001_schema.sql` — squashed baseline. Defines the current schema in
  one file. Pre-launch you can keep editing this in place + wipe DBs;
  once external installations exist it freezes. **When you edit the
  baseline, dev DBs that already applied the previous checksum will
  hard-error on next start.** Pre-launch, the tolerant runner
  re-baselines drifted checksums with a WARN — see
  `db::migrate::PRE_LAUNCH_REBASELINE`. Post-launch, edits to applied
  migrations are rejected; new migrations only.
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
cargo test -p ship-substrate
just check-migrations
```

Smoke against a real sqlite file:

```
rm -f /tmp/ship-mig.db
sqlx migrate run \
    --source crates/core/ship-substrate/migrations \
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
