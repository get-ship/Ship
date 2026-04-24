---
title: "Data DB (collections)"
description: "~/.ship/data.db — user content collections. DataService, built-in schemas, custom collections, the platform→data migration."
sidebar:
  label: "Data DB (collections)"
  order: 5
---
Path: `~/.ship/data.db`. Sibling to `platform.db`, different responsibility. Everything authored by the user — specs, ADRs, notes, feedback, annotations, plus any custom collections — lives here. Runtime state (workspaces, sessions, jobs, events) lives in `platform.db`.

Managed by `runtime::data::DataService` (`crates/core/runtime/src/data.rs`). Collection logic on top at `runtime::collections::*`.

## DataService

Single struct wrapping the connection:

```rust
pub struct DataService {
    conn: SqliteConnection,
    db_path: PathBuf,
}
```

Two constructors:

| Constructor | Use |
|---|---|
| `DataService::open(ship_dir)` | Production. Opens connection + runs startup. |
| `DataService::open_raw(ship_dir)` | Opens without startup. Used by the migration module's own tests. |

Connection PRAGMAs (set in `open_connection`): WAL, 5-second busy timeout. No foreign keys pragma here — collection tables are flat, no cross-collection FK constraints today.

Callers interact via two methods:

- `execute(sql, params)` — writes (INSERT / UPDATE / DELETE / DDL).
- `query(sql, params)` — reads.

Parameters use `data::SqlParam`. Callers do not open SQLite connections directly against `data.db`.

## Startup sequence

`DataService::open` calls `open_raw` then `run_startup(ship_dir)`. Startup is idempotent:

1. **Register built-in schemas** — `collections::builtin::register_builtin_schemas(svc)` ensures the 5 built-in collection tables exist: `specs`, `adrs`, `notes`, `feedback`, `annotations`. Registration is `CREATE TABLE IF NOT EXISTS`, safe to call repeatedly.
2. **Migrate legacy content** — `collections::migration::migrate_platform_to_collections(svc, platform_db_path)` copies notes, ADRs, and specs from legacy dedicated tables in `platform.db` into the corresponding collections. Gated by a completion flag (`platform_to_collections_v1`) stored in `data.db`'s `migration_state` table. Runs once, skips on subsequent startups.

Source rows in `platform.db` are not deleted by the migration. They remain as a safety net. A future release will prune them once migration is considered settled.

## Collections

A collection is a named, schema-backed bucket of items. Each collection gets its own SQLite table named `collection_<id>`, with real columns derived from the schema (not a blob of JSON).

### Schema format

```rust
pub struct CollectionSchema {
    pub id: String,
    pub name: Option<String>,
    pub fields: IndexMap<String, FieldDef>,
}

pub struct FieldDef {
    pub field_type: FieldType,
    pub required: bool,
    pub filterable: bool,
    pub default: Option<serde_json::Value>,
    pub values: Option<Vec<String>>,   // enum choices
}

pub enum FieldType {
    String, Markdown, Number, Boolean,
    Enum, Date, File, Array, Object,
}
```

Custom schemas live at `.ship/schemas/<id>.jsonc`, loaded via `schema::load_schemas(ship_dir)`. `schema::parse_schema(jsonc)` parses a single schema and validates it. Validation enforces: non-empty id, non-empty fields, enum fields have `values`, no reserved system-column names (`id`, `created_by`, `created_at`, `updated_at`).

### Built-in collections

`collections::builtin` exports five:

| ID | Schema fn | Purpose |
|---|---|---|
| `specs` | `specs_schema()` | Feature specs. Migrated from the legacy `spec` table on `platform.db`. |
| `adrs` | `adrs_schema()` | Architecture decision records. Migrated from `adr`. |
| `notes` | `notes_schema()` | Human scratchpad. Migrated from `note`. |
| `feedback` | `feedback_schema()` | User feedback on views, canvases, artifacts. |
| `annotations` | `annotations_schema()` | Inline annotations — comments/markers attached to artifacts and canvas sessions. |

`register_builtin_schemas(svc)` calls `ensure_collection_table(svc, &each)` for all five.

### System columns

Every collection table has four columns Ship manages, regardless of schema:

| Column | Type | Semantics |
|---|---|---|
| `id` | TEXT PRIMARY KEY | ULID. Caller-provided or generated. |
| `created_by` | TEXT | Identity of creator. |
| `created_at` | TEXT (rfc3339) | Set once at insert. |
| `updated_at` | TEXT (rfc3339) | Touched on every update. |

Schema fields cannot use these names. `validate_schema` rejects them.

### DDL generation

`store::ensure_collection_table(svc, &schema)`:

1. Check if `collection_<id>` exists via `sqlite_master`.
2. If missing: `build_create_table(schema)` emits DDL via `sea_query::Table` builder, `svc.execute` runs it.
3. If present: diff columns. For each field in the schema not yet a column, run `ALTER TABLE ... ADD COLUMN`. Schema changes are **additive only** — removing or renaming fields is not supported; that would break existing rows.

Indexes: fields with `filterable: true` get a single-column index. Built via `sea_query::Index`.

## Query path

`collections::query` converts a `QueryRequest` (filters, sort, limit) into parameterized SQL against the collection table. Callers don't write raw SQL — they build a `QueryRequest` and the query module translates to SQL with bind params.

Used by:

- MCP tools that expose `collections.list`, `collections.get`.
- Views that render collection content (specs browser, ADR list, annotation layer).
- Triggers filtering on collection events.

## Service layer

`collections::service` wraps store CRUD with event emission. Each mutation (create, update, delete) writes to `data.db` then emits an event through the kernel. Events are idempotent — item IDs are stable, re-emitting the same `{type, entity_id}` pair is harmless for projections.

Event types follow the pattern `collection.{action}` with item ID as `entity_id`:

- `collection.item_created`
- `collection.item_updated`
- `collection.item_deleted`

Triggers and views subscribe to `collection.*` via the kernel to stay live.

## Migration details

`migrate_platform_to_collections` reads from `platform.db` (read-only connection via `open_platform_readonly`). For each legacy table:

| Platform table | Target collection | Mapped fields |
|---|---|---|
| `spec` | `specs` | Standard + spec-specific fields |
| `adr` | `adrs` | Status, decision, context |
| `note` | `notes` | Body, associated workspace |

Report fields:

```rust
pub struct MigrationReport {
    pub already_migrated: bool,
    pub source_missing: bool,          // no platform.db at all
    pub notes_migrated: usize,
    pub adrs_migrated: usize,
    pub specs_migrated: usize,
}
```

Completion flag: `migration_state` table in `data.db`, keyed by `MIGRATION_KEY = "platform_to_collections_v1"`. Only one migration key exists today; future migrations would add keys.

## Fresh-install detection

`apps/shipd/src/first_run.rs::is_first_run(ship_dir)` checks `!ship_dir.join("data.db").exists()`. The presence of `data.db` is the canonical signal that Ship has been run at least once on this machine. Fresh installs skip migration (no source), populate built-in schemas, and mark the migration key complete immediately so subsequent startups don't probe the filesystem.

## Why not one database

Two-file split is deliberate:

- **Backup asymmetry.** `data.db` is user content — pricelessly user-authored. `platform.db` is runtime state — regeneratable from event log + filesystem. Backup policy differs.
- **Migration surface.** `platform.db` migrations move fast and sometimes clobber; `data.db` schema is additive-only. Splitting means platform schema churn doesn't threaten user content.
- **Audit / export.** Shipping user content (exports, cloud sync) means shipping `data.db`. A single combined file would pull in runtime state no third party should see.

## Caveats

- **Stale function name.** `collections::store::open_docs_db(ship_dir)` is still named `open_docs_db` despite the file being `data.db`. Returns a `DataService`. Renaming is a public API change; hasn't been done.
- **Migration source rows remain.** The `spec`, `adr`, `note` tables in `platform.db` are populated AND the collections are populated after migration. Double-source is a safety net, not a feature — queries should hit collections, not the legacy tables.
- **No FK constraints across collections.** A collection item can reference another item's ID in its data, but nothing enforces referential integrity. Callers validate or accept dangling refs.

## Related

- [Platform DB](/reference/runtime/platform-db/) — the other SQLite file, runtime state.
- [Persistence](/reference/runtime/persistence/) — event log under `platform.db`.
- [Projections](/reference/runtime/projections/) — read models for runtime state; distinct from collections, which are user content.
