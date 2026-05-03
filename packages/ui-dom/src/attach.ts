// attach() — the SSR-side client runtime.
//
// Walks DOM rendered by the Rust SSR walker, wires every reactive
// binding listed in `opts.module.bindings`, and runs the controller's
// `mount`. See `docs/ship-adrs/.../sui-marker-contract.md` for the
// marker contract.
//
// Phase 2 — control-flow boundaries (`if` / `for` / `match` / `await`)
// are wired client-side without server round-trip. The compiler emits
// per-boundary `__b<N>_attach` named handlers on the setup return; we
// dispatch SSR'd `<sui-{kind} key="b<N>">` wrappers to those handlers
// before walking leaf markers, so SSR'd content inside replaced
// boundaries doesn't get pointlessly re-wired.

import { route as makeRoute, type RouteSignals } from '@ship/ui-runtime'
import { readInitIsland } from './attach/island'
import { openLiveChannel } from './attach/live-channel'
import { indexBindings, walkMarkers } from './attach/manifest'
import { wireMarker, type WireContext } from './attach/wire'

/** Marker `kind` for each binding (matches the author-syntax attribute prefix). */
export type BindingKind =
  | 'text'
  | `on:${string}`
  | 'bind:value'
  | 'bind:checked'
  | `attr:${string}`
  | 'attr-dynamic'
  | `class:${string}`
  | `style:${string}`
  | 'while-loading'
  | 'while-error'
  | 'mutation'
  | 'bind:this'

/** One entry in the compiled module's `bindings` manifest. */
export interface Binding {
  /** `"b0"`, `"b1"`, … — the opaque counter-allocated binding key. */
  key: string
  /** Marker kind. See {@link BindingKind}. */
  kind: BindingKind
  /** Raw expression source from the IR. Empty when no expression applies. */
  expr: string
  /** Root identifiers the expression reads (de-duplicated, source order). */
  deps: string[]
  /** Name of the render callback on the setup return object, for while-loading / while-error. */
  renderKey?: string
}

/** Setup-context shape mirrored on `ResolverContext` from the runtime. */
export interface SetupContext {
  ship?: unknown
  /**
   * Reactive URL proxy. Always normalised to `RouteSignals` by
   * `attach()`; if the caller passed the legacy `{params,query}`
   * record shape, it is upgraded to a fresh `RouteSignals` whose
   * params/query signals are seeded from those entries.
   */
  route?: RouteSignals
  navigate?: (path: string) => void
  /** Template-declared signals already in scope. */
  scope?: Record<string, unknown>
  /** Names of writable `<state>` declarations. */
  stateKeys?: ReadonlySet<string>
}

/**
 * Subset of a compiled `.sui.js` module that `attach` requires.
 * `default` exposes a `setup(ctx)` returning the binding-set record
 * (with `hydrate(root, ctx)` and an optional non-enumerable
 * `__shipSetLifecycle`). `bindings` is legacy from pre-3b.5 emit and
 * is no longer populated by the current compiler — kept here for
 * backwards compatibility with bundles produced by older versions.
 */
export interface SetCompiledModule {
  default: { setup: (ctx: SetupContext) => Record<string, unknown> }
  bindings?: Binding[]
}

export interface AttachOptions {
  module: SetCompiledModule
  ship?: unknown
  /**
   * Reactive URL proxy. Pass a `RouteSignals` (constructed via
   * `route()` from `@ship/ui-runtime`) when the caller wants the view
   * to be route-reactive — typically the production boot script does
   * this. The legacy `{params,query}` record shape is still accepted
   * for backward compatibility with existing call sites and is
   * upgraded internally; a one-time `console.warn` flags the migration.
   * When omitted, `attach()` constructs a fresh `route()` from
   * `window.location`.
   */
  route?:
    | RouteSignals
    | { params: Record<string, string>; query: Record<string, string> }
  navigate?: (path: string) => void
  /** Name of the view (the path segment under `views/`). Retained for
   *  diagnostic output and live-channel routing; no longer used for
   *  control-flow swap (Phase 2 went client-side). */
  viewName?: string
}

export type AttachDisposer = () => void

const LIFECYCLE_KEY = '__shipSetLifecycle'

interface Lifecycle {
  state: Record<string, unknown>
  mount?: (state: Record<string, unknown>, el: HTMLElement) => void
  unmount?: (state: Record<string, unknown>) => void
}

/**
 * Walk SSR'd DOM rooted at `root`, wire every reactive binding listed
 * in `opts.module.bindings`, run the controller's `mount`, and return
 * a disposer that tears the whole thing down.
 *
 * Bootstrap order:
 *
 *   1. Read the `__set_init__` JSON island.
 *   2. Wrap each entry in a `Signal` (or `LiveSignal` for envelopes).
 *   3. Build the setup context and call `module.default.setup(ctx)`.
 *   4. Wire control-flow boundaries — `__b<N>_attach` handlers
 *      replace each SSR'd `<sui-{kind}>` wrapper with client-rendered
 *      content driven by the signal layer.
 *   5. Walk surviving manifest markers and dispatch to per-kind wirers.
 *   6. Run `lifecycle.mount(state, root)`.
 *
 * The disposer is idempotent.
 */
export function attach(root: HTMLElement, opts: AttachOptions): AttachDisposer {
  if (!root) {
    throw new Error('[ship-set/attach] attach() requires a root element')
  }
  const { module: mod } = opts
  if (!mod || !mod.default || typeof mod.default.setup !== 'function') {
    throw new Error(
      '[ship-set/attach] attach() requires a compiled module with default.setup()',
    )
  }
  const init = readInitIsland(root)

  const routeSignals = normalizeRoute(opts.route)

  const baseScope: Record<string, unknown> = {
    ship: opts.ship,
    route: routeSignals,
    navigate: opts.navigate ?? ((_p: string) => {}),
  }
  // Merge state/data signals into the same scope the setup will read.
  const scope: Record<string, unknown> = { ...baseScope, ...init.scope }

  const setupCtx: SetupContext = {
    ship: opts.ship,
    route: routeSignals,
    navigate: opts.navigate ?? ((_p: string) => {}),
    scope: { ...scope },
    stateKeys: init.stateKeys,
  }
  const setupRet = mod.default.setup(setupCtx)
  if (setupRet && typeof setupRet === 'object') {
    Object.assign(scope, setupRet)
  }
  // The compiled view's setup creates its own `$scope.<state> = signal(...)`
  // for every <state> declaration and `$scope.<data> = live(...)` for every
  // <data> declaration. Object.assign(scope, setupRet) above shadows
  // init.scope's SSR-built signals with these duplicates. The controller's
  // writable proxy targets init.scope (via setupCtx.scope), so controller
  // mount writes go to init's signals — but bindings would read the duplicates
  // and miss the updates. Restore init's signals (both state AND live) here so
  // all readers agree, and so the SSE patch-signals channel updates the same
  // signal the boundary effects are watching.
  for (const k of init.stateKeys) {
    if (init.scope[k]) scope[k] = init.scope[k]
  }
  for (const k of init.liveKeys) {
    if (init.scope[k]) scope[k] = init.scope[k]
  }
  const lifecycle = extractLifecycle(setupRet)

  const cleanups: Array<() => void> = []

  // Phase 3b.5 (gate H): hydrate now drives the entire view — text,
  // every leaf binding kind, control-flow boundaries (`if` / `for` /
  // `match` / `await`), and `while-*` are all dispatched from the
  // compiled view's `hydrate(root, ctx)`. The legacy manifest walker
  // (`walkMarkers`, `wireMarker`) is invoked only for bundles compiled
  // by pre-3b.5 versions that still ship a populated `bindings` array.
  if (mod.bindings && mod.bindings.length > 0) {
    const ctx: WireContext = { byKey: indexBindings(mod.bindings), scope }
    for (const hit of walkMarkers(root, ctx.byKey)) {
      const c = wireMarker(hit, ctx)
      if (c) cleanups.push(c)
    }
  }

  const hydrate = (setupRet as Record<string, unknown> | null)?.['hydrate']
  if (typeof hydrate === 'function') {
    try {
      const dispose = (hydrate as (r: HTMLElement, c: SetupContext) => unknown)(
        root,
        setupCtx,
      )
      if (typeof dispose === 'function') cleanups.push(dispose as () => void)
    } catch (err) {
      console.error('[ship-set/attach] compiled hydrate() threw:', err)
    }
  }

  if (lifecycle?.mount) {
    try {
      lifecycle.mount(lifecycle.state, root)
    } catch (err) {
      console.error('[ship-set/attach] controller mount() threw:', err)
    }
  }

  // Open one SSE EventSource per view when the compiler emitted a subscribe
  // URL (i.e. the view has at least one `<data live />` declaration, OR the
  // view participates in agent-pushed `patch-elements` frames). Route each
  // `patch-signals` event by data name to the matching LiveSignal, and each
  // `patch-elements` event to the view-scoped DOM applier.
  if (init.subscribe) {
    const es = openLiveChannel(init.subscribe, init.scope, init.liveKeys, root)
    cleanups.push(() => es.close())
  }

  let disposed = false
  return () => {
    if (disposed) return
    disposed = true
    if (lifecycle?.unmount) {
      try {
        lifecycle.unmount(lifecycle.state)
      } catch (err) {
        console.error('[ship-set/attach] controller unmount() threw:', err)
      }
    }
    for (const fn of cleanups) {
      try {
        fn()
      } catch (err) {
        console.error('[ship-set/attach] cleanup threw:', err)
      }
    }
  }
}

let warnedLegacyRoute = false

/**
 * Normalise an `AttachOptions['route']` to a `RouteSignals`.
 *
 *  - `undefined` → fresh `route()` from `window.location`.
 *  - already-`RouteSignals` (detected by callable `pathname`) → pass through.
 *  - legacy `{ params, query }` record → emit a one-time deprecation
 *    warning and construct a `RouteSignals` whose params/query signals
 *    are seeded from the record entries. Future `navigate()` /
 *    popstate updates are still tracked because the constructed
 *    instance registers with the global updater like any other.
 */
function normalizeRoute(
  input: AttachOptions['route'],
): RouteSignals {
  if (input === undefined) return makeRoute()
  if (isRouteSignals(input)) return input
  if (!warnedLegacyRoute) {
    warnedLegacyRoute = true
    console.warn(
      '[ship-set/attach] AttachOptions.route received the legacy { params, query } record shape; ' +
        'pass a RouteSignals (from runtime `route()`) instead. The record will be upgraded for now.',
    )
  }
  const r = makeRoute()
  for (const [k, v] of Object.entries(input.params ?? {})) {
    if (typeof v === 'string') r.params[k]!.set(v)
  }
  for (const [k, v] of Object.entries(input.query ?? {})) {
    if (typeof v === 'string') r.query[k]!.set(v)
  }
  return r
}

function isRouteSignals(v: unknown): v is RouteSignals {
  if (!v || typeof v !== 'object') return false
  const o = v as { pathname?: unknown }
  return typeof o.pathname === 'function'
}

function extractLifecycle(ret: unknown): Lifecycle | undefined {
  if (!ret || typeof ret !== 'object') return undefined
  const lc = (ret as Record<string, unknown>)[LIFECYCLE_KEY]
  if (lc && typeof lc === 'object') return lc as Lifecycle
  return undefined
}
