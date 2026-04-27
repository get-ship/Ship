# platform.db migrations

This directory holds the sqlx migrations for `platform.db`, Ship's
single platform-level database. sqlx runs every file here in
lexicographic order on `DataService::open` / `ensure_db_at`.

## Pre-launch convention

Single file: `0001_schema.sql`. When the schema changes, edit `0001`
in place and wipe + rebuild `platform.db` on existing machines. We're
pre-launch with a small, known set of installations — incremental
migration chains are debt we don't need yet.

When pre-launch ends, this convention switches to:

```
YYYYMMDDHHMM_kebab_description.sql
```

The CI guard (`scripts/lint/check-migration-naming.sh`, wired into
`just check-migrations`) enforces the active convention.

## Authoring rules

1. **Idempotent.** Use `CREATE TABLE IF NOT EXISTS`, `DROP TABLE IF
   EXISTS`, `CREATE INDEX IF NOT EXISTS`. A re-run on a partially-built
   database must not fail.
2. **No data.db here.** Per-collection `data.db` schemas live in a
   separate path (`runtime::collections::builtin::register_builtin_schemas`);
   do not mix them in.
3. **No data migration.** SQL DDL only. Pre-launch wipes are cheaper
   than data-migration code.

## Testing locally

```
cargo test -p runtime --test migrations
just check-migrations
```

`cargo test -p runtime` boots a fresh in-memory `platform.db`, runs
every migration in this directory, and asserts the resulting schema.
Expected output ends with `test result: ok`. `just check-migrations` is
the fast (<1s) naming-convention guard.

To smoke-test against a real sqlite file:

```
rm -f /tmp/ship-mig.db
sqlx migrate run \
    --source crates/core/runtime/migrations \
    --database-url sqlite:///tmp/ship-mig.db
sqlite3 /tmp/ship-mig.db ".schema"
```
