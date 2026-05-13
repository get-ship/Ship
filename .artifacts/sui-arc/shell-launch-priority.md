# Shell2 rail — launch-priority reshape

Branch: `job/shell-launch-priority`

## Decision

Reshape shell2's left rail to match the product's launch priorities:
**Agents · Skills · Collections · Artifacts · Workspaces · Settings**.

Deprioritized items (Dashboard, Home, Pipelines, Pipelines-board, Mesh,
Git, Events) are **unlinked from the rail but kept on disk**. Their view
directories under `views/` remain compileable and reachable by direct
URL — they were not deleted because "deprioritize" ≠ "delete", and
preserving them keeps in-progress work and existing deep links live.

Default landing route is now `/agents`. Pre-shift, bare `/` resolved
to `dashboard`; that view stays on disk but is no longer the implicit
home.

## Changes

- `apps/shell2/index.html` — rail item list trimmed to the six
  launch-priority destinations. Icons:
  `bot / layers / database / file / folder-tree / settings`.
- `apps/shell2/src/main.ts`
  - `readRoute` / `readRouteFromUrl`: empty path → `agents` (was
    `dashboard`).
  - On boot, when `location.pathname === '/'`, `replaceState` to
    `/agents` so `sui-rail`'s active-item derivation (which reads
    `location.pathname`) highlights Agents on first paint.
  - Landing fallback links updated to point at launch-priority views.
- `views/artifacts/` — new view (manifest + page.sui + controller +
  style). Lists artifact records returned by `GET /api/v1/artifacts`,
  with skeleton / empty / error states.

## SDK gap — `ship.artifacts.list()` is not registered

The artifacts REST surface exists on the daemon (`GET /api/v1/artifacts`
handled by `routes/artifacts_api.rs::list_artifacts`), backed by
`ship_platform::artifacts::ArtifactService::list`. The response shape is
`{ ok, data: ArtifactRecord[], count }`.

But the SDK catalog at `apps/shipd/src/services/schema.rs::OPS` has **no
`artifacts.*` entry** — so `ship.artifacts.list()` is not callable from
`<data live>` declarations or from `ship as ShipSDK` accessors. The
generated `ShipSDK` type in `packages/sui-runtime/src/ship.ts` also has
no `artifacts` domain.

For the launch-priority scaffold, the artifacts controller calls the
REST endpoint directly via `fetch('/api/v1/artifacts')`. This is a
one-shot fetch (no SSE), so the view will not auto-refresh when new
artifacts are uploaded — explicit `refresh()` only.

### Follow-up to close the gap

When promoting `/artifacts` past scaffold:
1. Add an `artifacts.list` entry to `OPS` in
   `apps/shipd/src/services/schema.rs` (args: `scope?`, `mime?`,
   `collection_id?`, `limit?`, `offset?`).
2. Wire the daemon's `ship.<domain>.<op>` dispatcher to invoke
   `artifacts_api::list_artifacts` (or call the service layer directly).
3. Add a live projection so the view can subscribe via
   `<data source="ship.artifacts.list()" live />` and react to
   `artifact.uploaded` / `artifact.deleted` events.
4. Re-generate the SDK (`just gen-sdk`) so `ShipSDK.artifacts.list()`
   appears in `packages/sui-runtime/src/ship.ts`.
5. Replace the REST fetch in `views/artifacts/controller.ts` with a
   `LiveSource` consumed by a `<data>` directive.

The `packages/sui-runtime/src/ship.ts` type was **not** extended in this
job. Adding `artifacts: { list(): LiveSource<…> }` to `ShipSDK` ahead of
daemon registration would create a dead surface — calls would fail at
runtime against the un-routed op name. Document-the-gap was the safer
half-step.

## Files preserved (not deleted)

Per spec: `views/{home,dashboard,pipelines,pipelines-board,mesh}/`
remain on disk and continue to compile. The daemon's view server still
resolves them; only the rail link was removed. Direct navigation to
`/dashboard`, `/pipelines`, etc. still mounts the corresponding view.

## Inventory probe

The rail now resolves to six destinations; smoke-mount each via
`apps/shell2/inventory-probe.mjs` (run with daemon up) to verify:
- `/agents` — `ship.agents.summaries()`
- `/skills` — `ship.skills.summaries()`
- `/collections` — `ship.collections.list()`
- `/artifacts` — REST `GET /api/v1/artifacts`
- `/workspaces` — `ship.workspaces.list()`
- `/settings` — `ship.settings.state()`
