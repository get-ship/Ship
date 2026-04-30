// attach() — the SSR-side client runtime.
//
// Walks DOM rendered by the Rust SSR walker (Phase D-1/D-2/D-3),
// finds `data-set-*` markers, looks each up in the compiled module's
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

import { readInitIsland } from './attach/island'
import { indexBindings, walkMarkers } from './attach/manifest'
import { wireMarker, type WireContext } from './attach/wire'

/** Marker `kind` portion of the `data-set-<kind>` attribute family. */
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

/** One entry in the compiled module's `bindings` manifest. */
export interface Binding {
  /** `"b0"`, `"b1"`, … — same shape as the `data-set-*` attribute value. */
  key: string
  /** Marker kind. See {@link BindingKind}. */
  kind: BindingKind
  /** Raw expression source from the IR. Empty when no expression applies. */
  expr: string
  /** Root identifiers the expression reads (de-duplicated, source order). */
  deps: string[]
}

/** Setup-context shape mirrored on `ResolverContext` from the runtime. */
export interface SetupContext {
  ship?: unknown
  route?: { params: Record<string, string>; query: Record<string, string> }
  navigate?: (path: string) => void
  /** Template-declared signals already in scope. */
  scope?: Record<string, unknown>
  /** Names of writable `<state>` declarations. */
  stateKeys?: ReadonlySet<string>
}

/**
 * Subset of a compiled `.html.set.js` module that `attach` requires.
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
  route?: { params: Record<string, string>; query: Record<string, string> }
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

  const baseScope: Record<string, unknown> = {
    ship: opts.ship,
    route: opts.route ?? { params: {}, query: {} },
    navigate: opts.navigate ?? ((_p: string) => {}),
  }
  // Merge state/data signals into the same scope the setup will read.
  const scope: Record<string, unknown> = { ...baseScope, ...init.scope }

  const setupCtx: SetupContext = {
    ship: opts.ship,
    route: opts.route ?? { params: {}, query: {} },
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
    viewName: opts.viewName,
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

function extractLifecycle(ret: unknown): Lifecycle | undefined {
  if (!ret || typeof ret !== 'object') return undefined
  const lc = (ret as Record<string, unknown>)[LIFECYCLE_KEY]
  if (lc && typeof lc === 'object') return lc as Lifecycle
  return undefined
}
