// attach() — the SSR-side client runtime.
//
// Walks DOM rendered by the Rust SSR walker (Phase D-1/D-2/D-3),
// finds author-syntax markers, looks each up in the compiled module's
// `bindings` manifest, seeds signals from the `__set_init__` JSON
// island, and wires reactivity for **leaf bindings** (text, attr,
// class, style, bind:, on:). Runs the controller's `mount` after
// attach. See `.artifacts/job-spec.md` (Phase E v1) and
// `docs/ship-adrs/.../ship-set-marker-contract.md` for the contract.
//
// Phase E-2 — control-flow boundaries (if / for / match / await) now
// swap-on-signal-change via a server round-trip. See
// `attach/boundaries.ts` for the swap mechanics; the runtime needs a
// `viewName` option so it knows which fragment-render endpoint to POST
// to. When `viewName` is omitted, boundaries still register reactive
// deps but log an error instead of swapping.
//
// Existing `hydrate` / parse-and-build path stays in place;
// `attach` is a sibling capability targeting SSR'd DOM.

import { route as makeRoute, type RouteSignals } from '@ship/ui-runtime'
import { readInitIsland } from './attach/island'
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
  | 'if'
  | 'for'
  | 'match'
  | 'await'
  | 'while-loading'
  | 'while-error'

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
 * `default` exposes a `setup(ctx)` returning the bindings record (and
 * optionally a non-enumerable `__shipSetLifecycle`); `bindings` is
 * the manifest indexed by `bN` keys.
 */
export interface SetCompiledModule {
  default: { setup: (ctx: SetupContext) => Record<string, unknown> }
  bindings: Binding[]
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
  /** Name of the view (the path segment under `views/`). Required for
   *  Phase E-2 control-flow swap; optional if the page has no boundaries
   *  whose deps can change. */
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
 *   4. Walk markers and dispatch to per-kind wirers.
 *   5. Run `lifecycle.mount(state, root)`.
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
  const lifecycle = extractLifecycle(setupRet)

  const ctx: WireContext = {
    byKey: indexBindings(mod.bindings),
    scope,
    initKeys: new Set([...init.stateKeys, ...init.liveKeys]),
    ...(opts.viewName !== undefined ? { viewName: opts.viewName } : {}),
  }

  const cleanups: Array<() => void> = []
  for (const hit of walkMarkers(root, ctx.byKey)) {
    const c = wireMarker(hit, ctx)
    if (c) cleanups.push(c)
  }

  if (lifecycle?.mount) {
    try {
      lifecycle.mount(lifecycle.state, root)
    } catch (err) {
      console.error('[ship-set/attach] controller mount() threw:', err)
    }
  }

  // SSE channel opening for `<data live>` is Phase F's territory — the
  // JSON island carries the initial envelope and Phase E-2 lets the
  // runtime react to envelope flips, but the actual SSE-→ LiveSignal
  // wiring lives outside attach.

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
