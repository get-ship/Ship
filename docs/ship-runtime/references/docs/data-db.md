---
group: Runtime
order: 5
title: Data DB (collections)
description: ~/.ship/machine/db/data.db — user content collections. DataService, built-in schemas, custom collections.
audience: internal
---

# Data DB

Path: `~/.ship/machine/db/data.db`. Sibling to `platform.db`, different responsibility. Everything authored by the user — specs, ADRs, notes, feedback, annotations, plus any custom collections — lives here. Runtime state (sessions, jobs, events) lives in `platform.db`.

Managed by `ship_substrate::data::DataService` (`crates/core/ship-substrate/src/data.rs`). Collection logic on top at `ship_platform::collections::*`.

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
| `DataService::open_raw(ship_dir)` | Opens without startup. Used by tests that need the bare connection. |

Connection PRAGMAs (set in `open_connection`): WAL, 5-second busy timeout. No foreign keys pragma here — collection tables are flat, no cross-collection FK constraints today.

Callers interact via two methods:

- `execute(sql, params)` — writes (INSERT / UPDATE / DELETE / DDL).
- `query(sql, params)` — reads.

Parameters use `data::SqlParam`. Callers do not open SQLite connections directly against `data.db`.

## Startup sequence

`DataService::open` calls `open_raw` then `run_startup(ship_dir)`. Startup is idempotent:

1. **Register built-in schemas** — `collections::builtin::register_builtin_schemas(svc)` ensures the 5 built-in collection tables exist: `specs`, `adrs`, `notes`, `feedback`, `annotations`. Registration is `CREATE TABLE IF NOT EXISTS`, safe to call repeatedly.

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
| `specs` | `specs_schema()` | Feature specs. |
| `adrs` | `adrs_schema()` | Architecture decision records. |
| `notes` | `notes_schema()` | Human scratchpad. |
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

## Fresh-install detection

`apps/shipd/src/daemon/first_run.rs::is_first_run(ship_dir)` checks whether `data.db` is absent. The presence of `data.db` is the canonical signal that Ship has been run at least once on this machine. Fresh installs populate built-in schemas on first startup.

## Why not one database

Two-file split is deliberate:

- **Backup asymmetry.** `data.db` is user content — pricelessly user-authored. `platform.db` is runtime state — regeneratable from event log + filesystem. Backup policy differs.
- **Migration surface.** `platform.db` migrations move fast and sometimes clobber; `data.db` schema is additive-only. Splitting means platform schema churn doesn't threaten user content.
- **Audit / export.** Shipping user content (exports, cloud sync) means shipping `data.db`. A single combined file would pull in runtime state no third party should see.

## Caveats

- **No FK constraints across collections.** A collection item can reference another item's ID in its data, but nothing enforces referential integrity. Callers validate or accept dangling refs.

## Related

- [Platform DB](/runtime/platform-db/) — the other SQLite file, runtime state.
- [Persistence](/runtime/persistence/) — event log under `platform.db`.
- [Projections](/runtime/projections/) — read models for runtime state; distinct from collections, which are user content.
