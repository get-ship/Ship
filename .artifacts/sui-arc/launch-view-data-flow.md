# Launch-view data flow audit

Branch: `job/launch-view-data-flow`

The four launch-priority views (Agents, Skills, Collections, Artifacts) now
read real daemon state through the `ship.*` SSE registry. This file
documents the projection / live-op path for each.

## Wire format

All four views consume `GET /_sui/data?expr=ship.<domain>.<op>(<json>?)` SSE
streams. The endpoint (`apps/shipd/src/views/sui_data.rs`) opens the channel
with an initial `set` patch derived from `services::registry::call(...)`,
subscribes to `config.*` kernel events, and re-emits a fresh `set` patch on
every coalesced burst. The browser SDK (`packages/ship-sdk-client`) wraps each
op as a `LiveSource<T>`; controllers `live(() => ship.<domain>.<op>(args))`.

## Agents

| Layer | Path |
| --- | --- |
| View source | `views/agents/page.sui` — `<data source="ship.agents.summaries()" live />` |
| Registry op | `agents.summaries` (`apps/shipd/src/services/registry/agents.rs`) |
| Projection  | `crate::services::agents::view_state::list_agents(ship_dir)` |
| SDK type    | `packages/sui-runtime/src/ship.ts` — `agents.summaries(): LiveSource<AgentsSummariesResponse>` |
| SDK client  | `packages/ship-sdk-client/src/index.ts` — already wired |

Shape: `{ items: AgentSummary[] }` where each `AgentSummary` carries id,
name, description, providers, tags, source label, plus the badge counts
the rail renders. Empty array is a valid response (no agents defined).

## Skills

| Layer | Path |
| --- | --- |
| View source | `views/skills/page.sui` (uses `ship.skills.summaries(...)` for the rail and `ship.skills.detail({id})` for the focus pane); `ship.skills.list()` is now also exposed for plain index callers |
| Registry op | `skills.list` (existed pre-change); types + client wiring added this branch |
| Projection  | `crate::services::skills::state::list(files)` |
| SDK type    | `packages/sui-runtime/src/ship.ts` — `skills.list(): LiveSource<{items: SkillIndexEntry[]}>` (new) |
| SDK client  | `packages/ship-sdk-client/src/index.ts` — `skills: { list: () => sse('skills', 'list') }` (new) |

Shape: `{ items: SkillIndexEntry[] }` with `{id, scope, name, description}`.
This is the minimal browser surface — `skills.summaries(...)` keeps the
filtered/grouped explorer payload the existing view uses.

## Collections

| Layer | Path |
| --- | --- |
| View source | `views/collections/page.sui` — `<data source="ship.collections.list()" live />` plus `ship.collections.detail({ id })` for the selected schema |
| Registry op | `collections.list`, `collections.detail`, `collections.items` (all added this branch in `apps/shipd/src/services/registry/collections.rs`) |
| Projection  | `ship_platform::collections::{builtin, schema}` for schemas; `ship_platform::collections::store::query_items` for rows |
| SDK type    | `packages/sui-runtime/src/ship.ts` — typed `collections.list / detail / items` shapes |
| SDK client  | `packages/ship-sdk-client/src/index.ts` — already exported `list`, `detail`, `items` |

Shape:
- `collections.list` → `{ items: [{id, name, field_count, fields: [{name, type, required, values, default}], builtin}] }`
- `collections.detail({id})` → `{ schema: {...}, items: Row[], total_items: number }`
- `collections.items({collection})` → `{ items: Row[], total_items: number }`

`collections.state` is retained (it powers a different template/minijinja
flow with tuple-shaped fields). The new ops carry record-shaped fields so
view controllers can read `f.name`, `f.type`, `f.required` directly.

## Artifacts

| Layer | Path |
| --- | --- |
| View source | n/a (no view yet — SDK + projection only) |
| Registry op | `artifacts.list` (new file `apps/shipd/src/services/registry/artifacts.rs`) |
| Projection  | `ship_platform::artifacts::ArtifactService::new(ship_dir).list(svc, req)` — opens `data.db` and queries the built-in `artifacts` collection sync |
| SDK type    | `packages/sui-runtime/src/ship.ts` — `artifacts.list(args?): LiveSource<{items, count, scope}>` (new) |
| SDK client  | `packages/ship-sdk-client/src/index.ts` — `artifacts: { list }` (new) |

Args mirror the REST `/api/v1/artifacts` shape: `scope` (default `"project"`),
`mime`, `collection_id`, `item_id`, `owner_actor`, `content_hash`, `sort`
(default `-created_at`), `limit`, `offset`. The underlying `ArtifactService`
is async on writes but sync on reads — we use the sync `list` so the
registry keeps its non-blocking contract.

## Schema catalog

All new ops carry descriptors in `apps/shipd/src/services/schema.rs::OPS`
so `/api/v1/ship/schema`, TS codegen, MCP auto-registration, and lint stay
in sync.

## Seeding

`just seed` (`scripts/dev/seed.sh`) posts one of each:

1. `POST /api/v1/agents` → `demo-agent`
2. `POST /api/v1/skills` → `demo-skill`
3. `POST /api/v1/collections` → `demo_notes` (+ one row)
4. `POST /api/v1/artifacts` (multipart) → `welcome.txt`

The script reads the daemon port + token from `daemon_dir/port` and
`daemon_dir/token` (resolved via `scripts/dev/_lib.sh`), so it targets the
worktree-scoped daemon rather than a global one. The previous registry
seeder is preserved as `just seed-registry`.

## Tests

`cargo test -p shipd --lib services::registry::tests` covers:

- `launch_view_ops_are_registered` — the four launch ops appear in `names()`.
- `collections_list_returns_user_schema_items` — built-in schemas surface
  with the new record-shaped fields.
- `artifacts_list_returns_empty_when_unused` — fresh daemon yields an empty
  list and reports scope correctly.
- `skills_list_returns_array` — registry call returns `{items: [...]}`.

`pnpm --filter @ship/sdk-client test` and `pnpm --filter @ship/sui-runtime
test` continue to pass.

## Out of scope

- No `views/`, `packages/ui-components/`, `packages/sui-build/`, or
  `packages/sui-compiler/` changes — those layers consume the SDK
  unchanged.
- No new top-level deps.
- No OpenAPI regen — all new ops live on the `_sui/data` registry stream,
  not the REST surface.
