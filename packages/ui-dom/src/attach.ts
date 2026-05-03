// attach() — the SSR-side client runtime.
//
// Reads the `__set_init__` JSON island, builds a setup context, calls
// the compiled view's `setup(ctx)`, and invokes the unified
// `hydrate(root, ctx)` named property on the setup return. Hydrate is
// the single dispatch surface for every binding kind (text, on:*,
// attr:*, class:*, style:*, bind:*, mutation, while-loading,
// while-error, control-flow boundaries) — the compiler inlines each
// kind directly. There is no manifest, no marker walker, no per-kind
// wirer left in the runtime.

import { route as makeRoute, type RouteSignals } from '@ship/ui-runtime'
import { readInitIsland } from './attach/island'
import { openLiveChannel } from './attach/live-channel'

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
 * `default.setup(ctx)` returns the binding-set record (with
 * `hydrate(root, ctx)` and an optional non-enumerable
 * `__shipSetLifecycle`).
 */
export interface SetCompiledModule {
  default: { setup: (ctx: SetupContext) => Record<string, unknown> }
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
   *  diagnostic output and live-channel routing. */
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
 * Walk SSR'd DOM rooted at `root`, dispatch the compiled
 * `hydrate(root, ctx)`, run the controller's `mount`, and return a
 * disposer that tears the whole thing down.
 *
 * Bootstrap order:
 *
 *   1. Read the `__set_init__` JSON island.
 *   2. Wrap each entry in a `Signal` (or `LiveSignal` for envelopes).
 *   3. Build the setup context and call `module.default.setup(ctx)`.
 *   4. Invoke the setup return's `hydrate(root, ctx)` — single pass
 *      that dispatches every binding kind by direct-emit dispatch.
 *   5. Run `lifecycle.mount(state, root)`.
 *   6. Open the SSE channel if the view has live data.
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
  // setupRet's $scope shadows init.scope's SSR-built signals with
  // duplicates. The controller's writable proxy targets init.scope, so
  // controller mount writes go to init's signals — restore them here so
  // bindings/SSE all read the same signals.
  for (const k of init.stateKeys) {
    if (init.scope[k]) scope[k] = init.scope[k]
  }
  for (const k of init.liveKeys) {
    if (init.scope[k]) scope[k] = init.scope[k]
  }
  const lifecycle = extractLifecycle(setupRet)

  const cleanups: Array<() => void> = []

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
