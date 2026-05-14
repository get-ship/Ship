# fix-view-controllers — per-view diagnosis

Six views surfaced controller-level errors in `apps/shell2/inventory-probe.mjs`. All
six are view-content / controller bugs (the framework was untouched).

## Common root cause: `route` is `{}` when `uses_route` is false on the template

Shell2's `mountRoute` does not pass `opts.route` to `runtime.mount(...)`. The
compiler only emits the `__routeFactory()` fallback for `$scope.route` when the
**template** references `route` (lower/mod.rs::expr_references_route). For views
whose templates never mention `route` but whose controllers do, the resolver's
`baseScope.route = ctx.route ?? {}` lands on `{}`, and `route.query.X()` /
`route()` throw immediately inside the controller's `scope:` factory.

The fix is to have the controller construct its own `RouteSignals` via the
`route()` factory imported from `@ship/sui-runtime/route`. Every `route()`
instance registers itself with the module-level `INSTANCES` set + the global
`__shipSetRouteUpdate` callback, so multiple instances stay in sync through
`navigate(...)` and popstate.

## Per-view fixes

### `/canvas` — "Mount failed: Cannot read properties of undefined (reading 'query')"
- `scope` factory destructured `route` from ctx and read `route.query.doc()`,
  but `route` was `{}`.
- `mount` used `mount: ({ state }) => …`, but the runtime calls `mount(state, host)`
  (mount.ts:194); the destructure pulled `.state` off the writableProxy ⇒
  `state` was `undefined` ⇒ every `state.X` access threw.
- Template also dereferenced `canvasList().value.length`,
  `canvasList().value.feedback`, etc. without guarding against
  `kind === 'ok' && value === null` (the canvas collection can be empty).
- Fix: import `route as routeFactory` from `@ship/sui-runtime/route`, construct
  inside the factory. Change `mount` signature to `(state)`. Add
  `&& canvasList().value` guards to every `kind === 'ok'` block.

### `/specs` — "Mount failed: Cannot read properties of undefined (reading 'query')"
- Same `route` undefined pattern in scope factory (`route.query.q()`,
  `route.query.id()`).
- No mount section, so only the scope-factory crash applied.
- Fix: same `routeFactory()` pattern.

### `/chat` — "route is not a function" + "Cannot set properties of undefined (setting '_cleanup')"
- Stale API: `(route as any)().query?.workspace_id` called `route()` as a
  function, but `route` is `{}` (post scope-typing #89, the API is
  `route.query.X()`).
- `mount: ({ state }) => …` made `state` undefined; the assignment
  `state._cleanup = …` threw.
- Fix: `routeFactory()` + `route.query.workspace_id()`. Change `mount`/`unmount`
  to take `state` directly.

### `/overview` — "Cannot read properties of undefined (reading 'canvasRef')" + 3× "Cannot read properties of null (reading 'items')"
- `mount: ({ state }, _el) => …` made `state` undefined ⇒ `state.canvasRef`
  threw.
- The template + `depsJson` derived assumed `systems().value.items` was always
  iterable, but `(ship as any).collections.items({...})` can return
  `{kind: 'ok', value: null}` for an empty/unknown collection. Three call
  sites (`{systems().value.items.length}`, `{#if systems().value.items.length === 0}`,
  `{#for item of systems().value.items}`) hit the null.
- Fix: `mount: (state, _el)` + read refs as values (the writableProxy already
  unwraps signal-shaped scope entries, so `state.canvasRef` returns the
  resolved element, no `()` call). Guard the three template sites with
  `kind === 'ok' && systems().value`. Guard `depsJson` derived the same way.

### `/collections` — "Cannot read properties of undefined (reading 'query')" at view.js:9
- Same `route` undefined pattern (`route.query.collection()` inside `derived`).
- Template also dereferenced `collections().value.items` and
  `colDetail().value.schema` without guarding against `value === null`.
- Fix: `routeFactory()` pattern in the controller. Add `&& collections().value`
  / `&& colDetail().value` to every `kind === 'ok'` template block.

### `/theme-studio` — "$scope.tokenCatalog is not a function"
- The template declared `<data name="tokenCatalog" source="ship.themes.tokenCatalog()" />`
  (no `live`). The compiler emits non-live data as
  `$scope.tokenCatalog = data({ initial: undefined, source: ... })`, but
  `data()` returns `{ signal, dispose }` — not a callable signal. The
  template's `tokenCatalog()` call therefore threw.
- Fix: add `live` to the `<data>` directive so the compiler emits
  `$scope.tokenCatalog = live(() => ship.themes.tokenCatalog())`, which is a
  callable `LiveSignal`. The controller's `unwrap(tokenCatalog)` keeps the
  DRAFT_DEFAULTS fallback if the SDK call fails or isn't wired yet.
