# Artifacts UI — Component & View Brief

**For:** ship-designer + operator (visual/interaction decisions are theirs).
**Provides:** components, purpose, RPC methods. No layout, no HTML.
**Backend:** `ArtifactsService` — `List` / `Patch` / `Promote` / `Delete` (unary) + `Watch` (stream, replaced `ListLive`). Contract: `proto/ship/ops/v1/artifacts.proto`. Shape: `README.md`.
**Demo data:** `fixtures/artifacts.demo.json` (decoded, designer-readable).

## What artifacts is
Content-addressed blob store + metadata projection. The UI is a **file browser with previews** — list blobs, filter, preview by mime, promote scratch→durable, delete. Reactive (workflow-produced artifacts appear live). `views/artifacts/page.sui` exists (legacy wire); this brief is the modernized component-pack version over the typed `Watch` service.

## Component pack — `sui-artifact-*`

### `sui-artifact-list`
- **What:** the browser — rows/cards with mime/size/actor/age, filterable, live.
- **Calls:** `List` (page) + `Watch` (live).
- **Use:** primary surface; applies Watch patches in place.

### `sui-artifact-preview`
- **What:** render one artifact by mime — image inline, text/markdown rendered, JSON tree, binary → download.
- **Calls:** metadata from `List`/`Watch` (`viewer_url`); bytes via the artifact URL.
- **Use:** drill-in. mime→viewer mapping is the core logic.

### `sui-artifact-filters`
- **What:** filter by mime / owner actor / age / collection_id.
- **Calls:** none (filters the result, or passes filter to `List`).

### `sui-artifact-actions`
- **What:** Promote (scratch→durable), Delete (confirm), rename (Patch).
- **Calls:** `Promote`, `Delete`, `Patch`. Promote + Delete are artifacts-specific verbs.

### `sui-artifact-upload`
- **What:** drop/select → create (legacy view has this).
- **Calls:** create path (confirm wire at build).

### `sui-watch-badge` (shared with collections)
- Live/reconnecting indicator on any reactive surface.

### States: loading · empty · empty-filtered · error · ok · live. `views/artifacts/page.sui` (legacy) is the flow + tone reference.

## Built-in view — `views/artifacts/`
Exists against legacy wire. Modernize: recompose from `sui-artifact-*` over typed `Watch`. **Must do** (arrangement is visual): browse/filter/preview live · promote/delete/rename · upload · drill into one artifact.

## RPC ↔ action
| Action | RPC |
|---|---|
| list | `List` |
| live | `Watch` |
| preview metadata | `List`/`Watch` → `viewer_url` |
| rename / reassign collection | `Patch` |
| promote | `Promote` |
| delete | `Delete` |

## Fixtures (`fixtures/artifacts.demo.json`)
5 artifacts (image/markdown/json/binary/text), Patch/Promote/Delete responses, Watch snapshot→append→remove, empty + error states.

## Deferred
- `BlobStore` substrate port (blob bytes crate-internal until substrate move) — no UI impact.
- Upload-wire modernization if still REST — confirm at build.
