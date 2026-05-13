# Artifacts view buildout

**Branch:** `job/artifacts-buildout`
**View:** `views/artifacts/`

The scaffolded `/artifacts` page (10 elements, bare empty state) replaced
with: real `<sui-table>`, filter row (search + mime + collection), row-click
detail drawer with preview pane, top-right upload dialog, and a polished
empty state that points users at the Agents view.

## Data flow

- **List**: `GET /api/v1/artifacts?limit=200` via imperative `fetch()` in
  the controller. After every upload the controller re-issues the request
  to refresh the table. Live SDK (`ship.artifacts.list()`) is registered
  in the daemon's OPS catalog and typed in `packages/sui-runtime/src/ship.ts`,
  but the REST response shape (`{ data, count }`) differs from the live
  shape (`{ items, count, scope }`), so the existing REST flow stayed in
  place — switching to live-only is a follow-up that can drop the manual
  refresh after upload.
- **Collections** (for filter dropdown + upload dialog): `live(() =>
  ship.collections.list())` — read via `unwrap(collectionsLive)` so the
  drop-downs re-render when a collection lands.
- **Upload**: `POST /api/v1/artifacts` as `multipart/form-data` with a
  `file` part and optional `collection_id`. The endpoint accepts a single
  file; the dialog enforces that by submitting `uploadFiles[0]`.
- **Download / preview source**: `GET /api/v1/artifacts/{id}` streams the
  bytes. The detail drawer uses this URL for `<img src>`, `<iframe src
  sandbox>`, and the download anchor. For `text/*` and JSON, the
  controller `fetch`-and-`text()`s the bytes (capped at 50 KB before a
  truncation marker) so we don't try to embed binary in a `<pre>`.

## Renderable-subset constraints

The SUI templates reject standalone function calls with arguments. Three
pieces of pre-computation happen in the controller's `decorate()`:

- `size_label` — human-readable bytes (e.g. `12.3 KB`)
- `id_short` — first 12 chars of the artifact id
- `created_short` — locale `MMM d, HH:mm`

All other reactive transforms (filter predicate, collection options,
preview kind, detail URL) live in `derived()` signals so the template
only reads accessors.

## Component choices

- **Table**: `<sui-table variant="slot">`. The slot variant lets us emit
  real `<tr slot="head">` + body rows with `attr:data-row-id={a.id}`. The
  table's mount handler emits `sui-row-click` on any `<td>` with a
  `data-row-id` ancestor, so the detail-drawer wiring is a single
  `on:sui-row-click={(e) => openDetail(e.detail.id)}` on the host. The
  download anchor in the action column calls `e.stopPropagation()` so a
  download click does not also open the drawer.
- **Drawer for detail**: `<sui-drawer size="lg" side="right">` slides over
  the table without unmounting it, matching the pattern in
  `views/collections/page.sui`. Preview pane swaps on the controller's
  `previewKind()` derived: `image | html | text | binary`.
- **Dialog for upload**: `<sui-dialog size="md">` — modal entry that
  blocks navigation while uploading. Cancel + Escape both close it; both
  routes also clear the staged file list. Footer disables both buttons
  while `uploading()` is true so a slow upload can't be doubled-tapped.
- **Empty states**: two — `data-test="empty"` (no artifacts at all,
  surfaces an Upload CTA + a link to `/agents`) and `data-test=
  "empty-filtered"` (some artifacts exist but none match filters,
  surfaces a clear-filters action).

## File layout

```
views/artifacts/
├── controller.ts   230 lines  (under 300 cap)
├── manifest.jsonc   unchanged
├── page.sui        239 lines  (template only)
└── style.css       250 lines

apps/shell2/artifacts-probe.mjs   158 lines
.artifacts/sui-arc/artifacts-buildout.md   (this file)
tests/e2e/visual/views.json   +1 line   ("artifacts" added)
```

`packages/sui-runtime/src/ship.ts` did **not** need to change — the
`ArtifactRecord` shape + `ship.artifacts.list()` op were already typed.

## Probe

`apps/shell2/artifacts-probe.mjs` is a Playwright smoke that asserts:

1. The view root + title render.
2. A primary surface (table | empty | error) is visible.
3. The Upload button is present.
4. If there are rows: the three filter controls render; typing a junk
   query narrows the table (or flips to the filtered-empty state).
5. Clicking Upload opens the dialog; the dialog contains a file picker,
   a collection select, and a submit button; Escape closes it.
6. If there is a row: clicking it opens the detail drawer with a preview
   region.
7. Zero console errors (favicon/network noise filtered).

The probe targets `http://127.0.0.1:${VITE_PORT||3270}`. It expects the
shell2 vite dev server + the per-worktree shipd daemon to be up. It is
not wired into CI here — adding it to `views.spec.ts` is the natural
next step once we have a stable per-worktree daemon-port assignment on
the build machine.

## Compile evidence

```
$ sui-compiler build views/artifacts/page.sui
compiled 1 file(s)
```

Emitted artifacts in `.ship/sui-out/views/artifacts/`:
`page.compiled.js`, `page.schema.json`, `page.ssr.html`, `page.sui.d.ts`,
`page.sui.meta.json`.

## Follow-ups

- Drop the imperative `refresh()` after upload by switching to the
  live SDK once the daemon's REST response shape is aligned with
  `ship.artifacts.list()` (or once a `useLive()` helper accepts the
  REST envelope directly).
- Sortable columns: the `<sui-table>` light-DOM CSS already styles
  `th[data-sort-key]`, and the host emits `sui-sort`. Wiring it would
  add ~15 lines to the controller (sort signal + comparator).
- The visual-baseline PNGs (paper + ship-dark) for `/artifacts` need
  to be captured by the visual-tests harness on a host with a live
  daemon — adding `"artifacts"` to `tests/e2e/visual/views.json` is
  enough to bring the route into the baseline sweep on next run.
