# platform.db migrations

This directory holds the sqlx migrations for `platform.db`, Ship's single
platform-level database. sqlx runs every file here in lexicographic order on
`DataService::open` / `ensure_db_at`.

## Naming convention (effective 2026-04-24)

All new migrations use a UTC timestamp prefix:

```
YYYYMMDDHHMM_kebab_description.sql
```

- `YYYYMMDDHHMM` — 12 digits, UTC, minute resolution. Must be monotonic
  relative to the newest existing file.
- `kebab_description` — lowercase ASCII, digits, `-`, `_`.
- `.sql` — no other extensions.

Example: `202604251430_add_agent_fingerprint.sql`.

The CI guard (`scripts/check-migration-naming.sh`, wired into
`just check-migrations`) enforces this pattern.

## Frozen history — do not edit

The original numeric-prefixed series is **frozen** as of 2026-04-24:

| File                                    | Status                    |
| --------------------------------------- | ------------------------- |
| `0001_schema.sql`                       | frozen                    |
| `0002_specs.sql`                        | frozen                    |
| `0003_drop_event_log.sql`               | frozen                    |
| `0004_compiled_asset_cache.sql`         | frozen                    |
| `0005_git_commit_drafts.sql`            | frozen                    |
| `0006_drop_platform_user_tables.sql`    | frozen — final 0001 edit  |

These files are history. Downstream installations have already applied them
under their current hashes; editing them breaks those installations. New
schema changes — even corrections to tables defined in `0001` — land as
**new** timestamp-named migrations.

## Authoring rules

1. **Idempotent.** Use `CREATE TABLE IF NOT EXISTS`, `DROP TABLE IF EXISTS`,
   `CREATE INDEX IF NOT EXISTS`. A re-run on a partially-applied database
   must not fail.
2. **Reversible where practical.** If the change is destructive (drop
   column, drop table), note the backup/rollback story in a comment at the
   top of the file. sqlx does not run down-migrations automatically, but
   the next author needs to know what was lost.
3. **Small and focused.** One concern per file. Prefer two migrations over
   one that does schema + backfill + index rebuild.
4. **No data.db here.** Per-collection `data.db` schemas live in a
   separate migration system; do not mix them in.
5. **Name describes the change.** `add_`, `drop_`, `rename_`, `backfill_`,
   `index_`. The reader should not need to open the file to understand
   intent.

## Testing locally

```
cargo test -p runtime --test migrations
just check-migrations
```

`cargo test -p runtime` boots a fresh in-memory `platform.db`, runs every
migration in this directory, and asserts the resulting schema. Expected
output ends with `test result: ok`. `just check-migrations` is the
fast (<1s) naming-convention guard.

To smoke-test against a real sqlite file:

```
rm -f /tmp/ship-mig.db
sqlx migrate run \
    --source crates/core/runtime/migrations \
    --database-url sqlite:///tmp/ship-mig.db
sqlite3 /tmp/ship-mig.db ".schema"
```

## Worked example

File: `202604251430_add_agent_fingerprint.sql`

```sql
-- Add a stable fingerprint column to agents so the orchestrator can
-- dedupe across restarts. Idempotent; safe to re-run.

ALTER TABLE agents
    ADD COLUMN fingerprint TEXT;

CREATE INDEX IF NOT EXISTS idx_agents_fingerprint
    ON agents(fingerprint);
```

That is the whole file. No transaction wrapper (sqlx handles it), no
up/down sections, no TODOs.
