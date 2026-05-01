// route.ts — URL state as reactive signals.
//
// `route.params.<name>` and `route.query.<name>` are signals: reading
// them inside an `effect`, `derived`, or a `live()` source thunk
// registers the current observer as a dependency. Navigation
// (popstate or `navigate(path)`) re-evaluates the URL and updates the
// signals; subscribers re-fire just like any other signal write.
//
// Param names are not declared at construct time. The `params` and
// `query` records are JS Proxies that lazily create a signal the first
// time a key is read. The initial value of each signal is computed
// from the current URL: `params` matches the pathname against the
// shapes registered via `defineRoutes(...)`; `query` reads
// `URL.searchParams`.
//
// Single source of truth for navigation: `navigate(path)` calls
// `history.pushState` (or `replaceState`) and triggers a global
// `__shipSetRouteUpdate` callback. Every `route()` instance registers
// itself with that updater on construction so all attached views see
// the same URL state.

import { signal, type Signal } from './signal'

/**
 * The reactive URL handle exposed to view scope as `route`.
 *
 * Each `params.<name>` and `query.<name>` is a `Signal<string |
 * undefined>` — call it to read the current value (and subscribe the
 * surrounding reactive scope). Writing to either record throws;
 * mutate via `navigate(path)` instead.
 *
 * Param names are discovered lazily: reading `route.params.id` for the
 * first time creates the signal and seeds it from the current pathname.
 * If no registered route shape matches the pathname (or the shape
 * doesn't capture `id`), the signal's value is `undefined` and stays
 * that way until a future navigation introduces a matching segment.
 */
export interface RouteSignals {
  /** Path params (e.g. `/agents/:id` → `params.id`). Lazy proxy. */
  params: Record<string, Signal<string | undefined>>
  /** Query params (e.g. `?q=foo` → `query.q`). Lazy proxy. */
  query: Record<string, Signal<string | undefined>>
  /** Pathname. Updates on every navigation. */
  pathname: Signal<string>
  /** Hash fragment, including the leading `#`. */
  hash: Signal<string>
}

/**
 * A registered route pattern (e.g. `/agents/:id`). Pattern segments
 * starting with `:` are captured as named params on `route.params`.
 */
export interface RouteShape {
  /** Path pattern, e.g. `'/agents/:id'`. */
  pattern: string
  /** Optional name for debugging. */
  name?: string
}

interface CompiledShape extends RouteShape {
  regex: RegExp
  keys: string[]
}

interface RouteInstance {
  update: () => void
}

// Module-level registry of route shapes. Shared across every
// `route()` instance so a `defineRoutes` call from one view's setup
// affects param matching for any subsequently constructed route.
const SHAPES: CompiledShape[] = []

// Live `route()` instances. The global `__shipSetRouteUpdate`
// callback walks this set on every navigation.
const INSTANCES: Set<RouteInstance> = new Set()

let popstateBound = false
let updaterInstalled = false

/**
 * Register one or more route shapes whose `:param` segments should
 * populate `route.params`. Safe to call any number of times; later
 * calls append to the registry. Triggers an immediate refresh so
 * already-constructed `route()` instances re-seed their params from
 * the new shapes.
 *
 * Per-view convention: call `defineRoutes` from the view's `view.ts`
 * (or controller setup) before reading any params. The compiler may
 * lift this to a `<route pattern="..." />` directive in a follow-up;
 * for now it's an imperative call.
 *
 * ```ts
 * import { defineRoutes } from '@ship/ui-runtime'
 * defineRoutes([{ pattern: '/agents/:id' }])
 * ```
 */
export function defineRoutes(shapes: RouteShape[]): void {
  for (const s of shapes) SHAPES.push(compilePattern(s))
  // Re-fire the global updater so existing route() instances pick up
  // the new shapes for any params already lazily read.
  globalThis.__shipSetRouteUpdate?.()
}

/**
 * Construct a `RouteSignals` from the current URL.
 *
 * @param opts.url Explicit URL (used by tests). Defaults to
 *   `window.location.href` in the browser; falls back to
 *   `http://localhost/` outside the browser.
 * @param opts.routes Convenience — calls {@link defineRoutes} before
 *   constructing, so the returned `params` are seeded against these
 *   shapes immediately.
 */
export function route(opts?: { url?: URL; routes?: RouteShape[] }): RouteSignals {
  if (opts?.routes) defineRoutes(opts.routes)

  let currentUrl: URL = opts?.url ?? readWindowUrl()

  const pathname = signal(currentUrl.pathname)
  const hash = signal(currentUrl.hash)
  const paramSigs = new Map<string, Signal<string | undefined>>()
  const querySigs = new Map<string, Signal<string | undefined>>()

  const seedParam = (key: string): string | undefined =>
    matchPathname(currentUrl.pathname)[key]
  const seedQuery = (key: string): string | undefined =>
    currentUrl.searchParams.get(key) ?? undefined

  const params = makeProxy(paramSigs, seedParam, 'params')
  const query = makeProxy(querySigs, seedQuery, 'query')

  const update = (): void => {
    currentUrl = readWindowUrl()
    pathname.set(currentUrl.pathname)
    hash.set(currentUrl.hash)
    const matched = matchPathname(currentUrl.pathname)
    for (const [k, s] of paramSigs) s.set(matched[k])
    for (const [k, s] of querySigs) {
      s.set(currentUrl.searchParams.get(k) ?? undefined)
    }
  }

  const instance: RouteInstance = { update }
  INSTANCES.add(instance)
  ensureGlobalUpdater()
  ensurePopstateListener()

  return { params, query, pathname, hash }
}

/**
 * SPA-aware navigation. Pushes (or replaces) `path` into history and
 * fires the global route updater so every attached `route()` instance
 * re-evaluates its signals against the new URL.
 *
 * Outside the browser (no `window.history`), this is a no-op.
 *
 * ```ts
 * navigate('/agents/42')
 * navigate('/login', { replace: true })
 * ```
 */
export function navigate(path: string, opts: { replace?: boolean } = {}): void {
  if (typeof window === 'undefined' || !window.history) return
  if (opts.replace) {
    window.history.replaceState({}, '', path)
  } else {
    window.history.pushState({}, '', path)
  }
  globalThis.__shipSetRouteUpdate?.()
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

function readWindowUrl(): URL {
  if (typeof window !== 'undefined' && window.location?.href) {
    return new URL(window.location.href)
  }
  return new URL('http://localhost/')
}

function compilePattern(shape: RouteShape): CompiledShape {
  const keys: string[] = []
  const segments = shape.pattern.split('/').map((seg) => {
    if (seg.startsWith(':')) {
      keys.push(seg.slice(1))
      return '([^/]+)'
    }
    return escapeRegExp(seg)
  })
  return {
    ...shape,
    regex: new RegExp('^' + segments.join('/') + '/?$'),
    keys,
  }
}

const escapeRegExp = (s: string): string =>
  s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

function matchPathname(pathname: string): Record<string, string | undefined> {
  for (const shape of SHAPES) {
    const m = shape.regex.exec(pathname)
    if (!m) continue
    const out: Record<string, string | undefined> = {}
    shape.keys.forEach((k, i) => {
      out[k] = m[i + 1]
    })
    return out
  }
  return {}
}

/**
 * Build the lazy Proxy backing `params` or `query`. `proxy[key]`
 * returns a `Signal<string|undefined>`, created on first access from
 * `seed(key)`. `proxy.toJSON()` serializes currently-known keys via
 * `.peek()` (untracked) so `JSON.stringify` works. `set`/
 * `deleteProperty` throw — change the URL via `navigate(path)`.
 * `Object.keys` reflects only keys that have been read at least once.
 */
function makeProxy(
  store: Map<string, Signal<string | undefined>>,
  seed: (key: string) => string | undefined,
  label: string,
): Record<string, Signal<string | undefined>> {
  const ensure = (key: string): Signal<string | undefined> => {
    let s = store.get(key)
    if (!s) {
      s = signal<string | undefined>(seed(key))
      store.set(key, s)
    }
    return s
  }
  const target: Record<string, Signal<string | undefined>> = Object.create(null)
  return new Proxy(target, {
    get(_t, key) {
      if (typeof key !== 'string') return undefined
      if (key === 'toJSON') {
        return () => {
          const out: Record<string, string | undefined> = {}
          for (const [k, s] of store) out[k] = s.__peek()
          return out
        }
      }
      return ensure(key)
    },
    set(_t, key) {
      throw new Error(
        `[ship-set] route.${label}.${String(key)} is read-only — call navigate(path) to change the URL`,
      )
    },
    deleteProperty(_t, key) {
      throw new Error(
        `[ship-set] cannot delete route.${label}.${String(key)} — call navigate(path) to change the URL`,
      )
    },
    has(_t, key) {
      return typeof key === 'string' && store.has(key)
    },
    ownKeys() {
      return [...store.keys()]
    },
    getOwnPropertyDescriptor(_t, key) {
      if (typeof key !== 'string') return undefined
      const s = store.get(key)
      if (!s) return undefined
      return { configurable: true, enumerable: true, writable: false, value: s }
    },
  })
}

function ensureGlobalUpdater(): void {
  if (updaterInstalled) return
  updaterInstalled = true
  globalThis.__shipSetRouteUpdate = (): void => {
    for (const inst of INSTANCES) inst.update()
  }
}

function ensurePopstateListener(): void {
  if (popstateBound) return
  if (typeof window === 'undefined' || !window.addEventListener) return
  popstateBound = true
  window.addEventListener('popstate', () => {
    globalThis.__shipSetRouteUpdate?.()
  })
}

declare global {
  // eslint-disable-next-line no-var
  var __shipSetRouteUpdate: (() => void) | undefined
}
