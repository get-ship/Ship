// Shell — SUI-native three-zone chrome.
//
// Layout: 36px <sui-shell-top> · canvas · 28px <sui-shell-bottom>. The
// canvas hosts either the active view's content (mounted into
// #ship-content) or a default <sui-blank-canvas> when no route is open.
// Primary nav lives in the ⌘K command palette — there is no rail in the
// shell mount (the <sui-rail> primitive is still registered for code
// paths that mount one internally, e.g. Studio).
//
// Per-request flow:
//
//   1. Read location.pathname → derive the current view name + sub.
//   2. Fetch /_deck/views/<name>[/<sub>] → ViewPayload.
//   3. mount(content, payload) → MountHandle.
//   4. On internal anchor clicks → history.pushState + re-mount.
//   5. On popstate → re-mount.
//   6. Esc returns to the blank canvas; ⌘K opens the palette;
//      ⌘. navigates to /studio; ⌃` toggles the terminal panel;
//      ⌘? opens help.

import { createClient, DECK_PREFIX, setShipClient, type ShipSDK } from '@ship/sdk-client'
import { ops } from './ops'
// `import './components'` and `import { loadHeavyChrome } from
// './chrome/load-heavy-chrome'` removed. Both source files were deleted in
// commit `ee33c2c9a` (the static-chrome-import drop) but main.ts still
// referenced them, which made the scaffold build (`cargo run -p deck-cli
// -- build --shell-root apps/shell`) fail with rolldown's
// `DiagnosableResolveError` for the dangling specifiers.
//
// The chrome is registered through the per-tag resolver, content-deduped
// (chunking fix `min_share_count: 1` → one chunk per source file). The
// heavy-chrome dynamic chunks were a pre-rolldown chunking era; with
// content-dedup they don't add value.
// Domain packs — each side-effect-registers its custom elements. The pack
// main compiles in-process via the shared SuiPlugin and tree-shakes
// per-tag from the traced app bundle. This is the multi-pack architecture:
// the shell's first-party chrome imports a workspace pack via the same
// shape a third-party pack will use. The umbrella bundle
// (packages/components/ui-components) is gone — each pack loads its
// The 5 side-effect pack imports below (`@ship/sui-charts`, `@ship/pipelines`,
// `@ship/sui-canvas`, `@ship/agents`, `@ship/authority`) are removed. Each
// pack's source `index.ts` imports `<tag>/<tag>.compiled.js` siblings — files
// the deck's in-process rolldown plugin generates on the fly for views, but
// the scaffold's `bundle_app` path (this build) can't synthesize them
// because they only exist on the view-build path. Result: `cargo run -p
// deck-cli -- build --shell-root apps/shell` fails with rolldown
// `DiagnosableResolveError` for `*.compiled.js`.
//
// The chrome these packs register is also registered by the per-tag resolver
// (sui_plugin.rs walks `apps/shell/src/components/` + the workspace
// primitives and emits one chunk per tag). The per-tag resolver is
// content-deduplicated (`min_share_count: 1` in rolldown_options), so the
// chrome primitives register exactly once. The pack imports were
// pre-per-tag-resolver era — they were duplicates that also produced
// `customElements.define called twice` warnings before the chunking fix.
//
// The shell's home view uses `<sui-button>`, `<sui-skeleton>`, etc. — all
// primitives, all covered by the per-tag resolver. The pack imports
// registered *more* tags (charts, pipelines, canvas, agent cards,
// authority cards) that the home view doesn't actually mount. The home
// view's behavior is unchanged.
//
// If a future view needs a pack's tag, it can `import '@ship/sui-charts'`
// from THAT view's controller (the view bundle emits the pack correctly
// through the view build path) — not from the shell's main.ts.
// Primitives pack — generic `<sui-*>` UI elements (button, input, dialog,
// popover, tooltip, tabs, etc.) — are resolved per-tag from workspace source
// that the per-tag resolver cannot trace.
import { route as createRoute, defineRoutes, setRouterPort } from '@ship/sui-runtime/route'
import { DYNAMIC_ROUTE_PATTERNS } from '../routes.gen'
import { chrome, mount, notifyViewUnmount, setHostContext } from '@ship/sui-runtime'
import type { SuiLoader, ViewPayload } from '@ship/sui-runtime'
import { KernelBridge } from './kernel-bridge'
import { initHmr } from './hmr'
import { initProjectSwitch } from './project-switch'
import { initChromeRegistry } from './chrome/registry'
import { initEditModeChrome, setEditModeView, toggleEditMode, toggleAnnotationMode } from './chrome/edit-mode'
import { bridgeDockTerminal, disposeDockTerminal } from './chrome/terminal-bridge'
import { registerKeymap } from './chrome/keymap'
// Seed the static index.html chrome (`<sui-nav-top>` etc.) into the app graph
// so the per-tag resolver compiles + registers them — they live outside any
// compiled `.sui` it scans, so without this the shell paints a blank skeleton.
import './components'


// The daemon's base origin. Empty in a browser/vite context (same-origin; the
// vite proxy forwards /api + /_deck + Connect to the daemon). In the proxy-less
// Tauri webview, the host plants `window.__SHIP_DAEMON_URL__` =
// `http://127.0.0.1:<port>` so every daemon call targets the absolute loopback
// origin directly (no proxy — the doc's Connect-over-loopback model).
function daemonBase(): string {
  return (window as WindowWithSui).__SHIP_DAEMON_URL__?.replace(/\/$/, '') ?? ''
}

// Inject the daemon's design-token stylesheet against the discovered base, as
// early as module-eval allows. It can't be a static <link> in index.html: the
// daemon origin is only known at runtime (the loopback port in the proxy-less
// Tauri webview), and a root-absolute `/api/...` href would resolve to the
// webview origin there, not the daemon. Same-origin under vite (`daemonBase()`
// is ''), absolute under Tauri. CSP `style-src` admits the loopback origin.
function injectTokenStyles(): void {
  if (document.querySelector('link[data-ship-view-tokens]')) return
  const link = document.createElement('link')
  link.rel = 'stylesheet'
  link.href = `${daemonBase()}/api/assets/view-tokens.css`
  link.setAttribute('data-ship-view-tokens', '')
  document.head.appendChild(link)
}
injectTokenStyles()
// Heavy-chrome chunk fetch (loadHeavyChrome) was here pre-chunking-fix;
// removed. The chrome primitives (sui-*) are now content-deduplicated to
// one chunk each by the bundler's `min_share_count: 1` config, and they
// register at first `<sui-*>` tag mount via the per-tag resolver, no
// separate prefetch needed.
// Single SDK client instance, shared across every mounted view.
const shipClient: ShipSDK = createClient({ baseUrl: daemonBase() })
// Expose the client to `ship-*` packs (e.g. @ship/agents) that import
// the lazy `ship` Proxy from `@ship/sdk-client`. Pure `sui-*` packs do
// not reach for the SDK and are unaffected.
setShipClient(shipClient)

// Host loader injected into every mount() — the `/_deck/` URL scheme lives
// here at the app boundary, not inside the (host-agnostic) SUI runtime.
const suiLoader: SuiLoader = {
  async fetchView(nameOrPath: string): Promise<ViewPayload> {
    // Route pathnames arrive leading-slashed ("/git"); the view-payload route
    // is `/_deck/views/{name}/...` so a raw concat yields a blank `{name}`
    // segment (`/_deck/views//git` → 404). Normalize to the bare view name.
    const seg = nameOrPath.replace(/^\/+/, '')
    const rel = seg.startsWith(`${DECK_PREFIX.replace(/^\//, '')}/views/`) || nameOrPath.startsWith(`${DECK_PREFIX}/views/`)
      ? nameOrPath
      : `${DECK_PREFIX}/views/${seg}`
    const path = `${daemonBase()}${rel}`
    const res = await fetch(path, { credentials: 'same-origin' })
    if (!res.ok) {
      const body = await res.text().catch(() => '')
      throw new Error(`[shell] fetch ${path} ${res.status}: ${body.slice(0, 200)}`)
    }
    return (await res.json()) as ViewPayload
  },
}

// Pre-register dynamic-route patterns the shell knows about. Views in
// `views/<name>/[id]/` use these to read `route.params.id()`. Views
// can also register their own at controller-import time, but those
// registrations land in the view-bundle's copy of @ship/sui-runtime
// (served via `/_deck/runtime.js`) — a separate module instance from
// the shell's bundle of @ship/sui-runtime, so the shell's
// `routeSignals` proxy never sees them. Registering here at the shell
// layer means params get seeded against the SHAPES the shell knows.
// Pre-register the dynamic-route patterns the shell's view tree backs.
// Generated by `deck routes` from every `[param]` dir under
// `apps/shell/src/`. Static routes need no entry — the runtime matches
// them by segment equality. Views can also call `defineRoutes` from
// their own setup; those registrations land in the view-bundle's copy
// of @ship/sui-runtime (a separate module instance from the shell's),
// so the shell's `routeSignals` proxy never sees them. Registering
// here at the host layer means params get seeded against the SHAPES
// the shell itself owns.
defineRoutes(DYNAMIC_ROUTE_PATTERNS)

// Shared route signal. Views that declare dynamic params read
// `route.params.id()`; the singleton updates on every popstate /
// history.pushState the shell triggers.
const routeSignals = createRoute()
// The shell's router. `history.pushState` (or replace) + re-mount. The
// runtime owns the path → param matching via `defineRoutes`; the shell
// just hands the pathname to `mountRoute`. Registered as the runtime's
// `RouterPort` below so `navigate()` from any view bundle routes
// through here and actually mounts.
function navigate(path: string, opts: { replace?: boolean } = {}): void {
  if (opts.replace) history.replaceState({}, '', path)
  else history.pushState({}, '', path)
  void mountRoute(location.pathname)
}

// Make the shell the host router: the runtime's `navigate()` (called from
// any view bundle) delegates here instead of the old signals-only path that
// never mounted. Collapses the runtime/`window.ship.navigate`/`sui:navigate`
// split into one entry point. See ADR `unified-deck-router`.
setRouterPort({
  navigate: (to, o) => navigate(to, o ?? {}),
  back: () => history.back(),
})

// ── window.ship API ──────────────────────────────────────────────────

const kernelBridge = new KernelBridge()

function initShipApi(): void {
  window.ship = {
    navigate(path: string) {
      chrome.navigate(path)
    },
    back() { history.back() },
    current() { return { path: location.pathname } },
    notify(message: string, tone?: 'success' | 'warning' | 'error' | 'info') {
      const opts: Parameters<typeof chrome.notify>[0] = { title: message, duration: 5000 }
      if (tone) opts.tone = tone
      chrome.notify(opts)
    },
    subscribe: kernelBridge.subscribe.bind(kernelBridge),
    command() { chrome.bar.open() },
  }
}

declare global {
  interface Window {
    ship?: {
      navigate(path: string): void
      back(): void
      current(): { path: string }
      notify(message: string, tone?: 'success' | 'warning' | 'error' | 'info'): void
      subscribe(namespace: string, handler: (event: { event_type: string; entity: string; timestamp: string; payload: unknown; [k: string]: unknown }) => () => void): () => void
      command(): void
    }
    /** Devtools surface: scope of the currently-mounted view. Re-published
     *  on every successful mountRoute and cleared when landing/dispose
     *  runs. `<sui-devtools-signals>` reads this to render the live tree —
     *  a corresponding `sui:scope:change` event fires on document so
     *  consumers can re-anchor without polling. */
    __suiCurrentScope?: Record<string, unknown> | null
    /** Identifier for the currently-mounted view. Read by
     *  `@ship/sui-runtime/chrome`'s wrapper API to stamp per-view claims
     *  (breadcrumb, status keys, theme overrides, focus push). Set on
     *  each mountRoute and cleared on landing/dispose so claims auto-
     *  clear when shell fires `sui:view:unmount`. */
    __suiCurrentViewId?: string | null
  }
}

/** Read `<meta name="ship-mode">` and normalize. Anything that isn't
 *  `production` falls back to development so the badge errs on the side
 *  of being shown. */
function readShipMode(): 'Production' | 'Development' {
  const meta = document.querySelector('meta[name="ship-mode"]') as HTMLMetaElement | null
  const raw = (meta?.content ?? '').trim().toLowerCase()
  return raw === 'production' ? 'Production' : 'Development'
}

function publishScope(scope: Record<string, unknown> | null): void {
  window.__suiCurrentScope = scope
  document.dispatchEvent(new CustomEvent('sui:scope:change', { detail: { scope } }))
}
// Window globals planted by the Tauri host (pre-load init script) and the
// daemon's HTML scaffold. Cast at the call sites to avoid `declare
// global` in a non-ambient module file. (The shell imports the SUI
type WindowWithSui = Window & {
  // Planted by the Tauri host (pre-load init script) so the proxy-less
  // webview targets the daemon's absolute loopback origin. Undefined in a
  // browser/vite context → '' → same-origin relative URLs (the vite
  // proxy resolves them).
  __SHIP_DAEMON_URL__?: string
  // The space this window was launched for (`--app=<id>`); undefined = the hub.
  DECK_APP_ID?: string
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
}

// One mount path. The URL is a path; the runtime matches it against
// the registered `RouteShape[]` (codegened from the shell's view
// tree) and seeds `route.params.<name>()`. The shell hands the raw
// pathname to `runtime.mount`; the loader fetches the payload; the
// runtime does the rest. If the loader throws (resolver 404), the
// shell shows the not-registered state for that pathname and doesn't
// re-fetch. No imperative route table, no chrome map, no manual
// name→tag mapping.

// ── Mount lifecycle ───────────────────────────────────────────────
//
// One mount path. `mountRoute(pathname)` runs on every navigation:
//   1. Bump the latestMountToken; the previous in-flight mount self-
//      disposes on its return if it lost the race.
//   2. Resolve the runtime, hand it the slot + pathname + opts. The
//      runtime calls `suiLoader.fetchView(pathname)`; the loader
//      hits the daemon's resolver. If the resolver 404s, the loader
//      throws, the runtime re-throws, and the catch below shows the
//      not-registered state. The name is NOT cached — every navigation
//      asks the daemon fresh, so a view that didn't exist at 09:00 but
//      was authored at 09:01 starts working on the next navigation.
//   3. The mount returns a `MountHandle` with dispose(). The previous
//      handle disposes so SSE channels + style elements don't leak.

let currentHandle: { dispose(): void; scope?: Record<string, unknown> } | null = null
let latestMountToken = 0
let currentPathname: string | null = null
let currentViewId: string | null = null
let viewIdSeq = 0

function nextId(): string {
  viewIdSeq += 1
  return `view-${viewIdSeq}`
}

function disposeCurrentMount(): void {
  const prevId = currentViewId
  currentHandle?.dispose()
  currentHandle = null
  currentViewId = null
  window.__suiCurrentViewId = null
  if (prevId) notifyViewUnmount(prevId)
}

/** First path segment, lowercased. Used as the body[data-view] hook
 *  so chrome (nav-top, etc.) can scope to the active page. Empty
 *  pathname → '' (landing). */
function viewTagOf(pathname: string): string {
  const seg = pathname.split('/').filter(Boolean)[0]
  return (seg ?? '').toLowerCase()
}

// ── SES confinement (opt-in) ───────────────────────────────────
// Deck confines view code via @ship/deck-host. OFF by default; the confined
// path lazy-loads deck-host (+ ses) only when enabled, so the default shell is
// unchanged (no ses in the bundle's hot path). Enable with localStorage
// `ship-ses-confine` = '1' (a follow-up defaults this to production mode). #165.
function confineEnabled(): boolean {
  try {
    return localStorage.getItem('ship-ses-confine') === '1'
  } catch {
    return false
  }
}

// Confined modules are cached per view name — the descriptor is stateless
// (setup runs fresh per mount), so reuse matches the unconfined path's reliance
// on the browser's dynamic-import cache. Without this, every remount (e.g. live
// re-render) re-fetched + re-confined the source. Promise-valued so concurrent
// mounts share one in-flight build.
const confinedCache = new Map<string, Promise<unknown>>()

// Build the confined module for a daemon-served view: fetch its compiled source,
// endow the SHARED runtime (the importmap's @ship/sui-runtime module
// instance, NOT a second copy, so signals stay one universe per #149)
// + dom + policy-gated ship/route/navigate + the real document, and
// confine under lockdown. Returns undefined on any failure (incl. non-
// daemon views like bundled studio, whose view.js 404s) → caller
// falls back to the normal unconfined path.
//
// The runtime is imported as a namespace import here so the SES
// endowments pass the SAME module instance that `mount` (line 42)
// uses — the importmap resolves both to /_deck/runtime.js.
import * as runtimeNs from '@ship/sui-runtime'
const suiRuntime = runtimeNs as unknown as Record<string, unknown>

function confinedModuleFor(viewName: string): Promise<unknown> {
  const cached = confinedCache.get(viewName)
  if (cached) return cached
  const built = buildConfinedModule(viewName)
  confinedCache.set(viewName, built)
  return built
}

async function buildConfinedModule(viewName: string): Promise<unknown> {
  try {
    const [{ buildViewEndowments, confineViewModule }, dom] = await Promise.all([
      import('@ship/deck-host'),
      import('@ship/sui-dom'),
    ])
    const res = await fetch(`${daemonBase()}${DECK_PREFIX}/views/${viewName}/view.js`, { credentials: 'same-origin' })
    if (!res.ok) return undefined
    const source = await res.text()
    const endow = buildViewEndowments({
      runtime: suiRuntime,
      dom: dom as unknown as Record<string, unknown>,
      ship: shipClient,
      route: routeSignals,
      navigate,
      doc: document,
    })
    // confineViewModule returns the `{ default }` module shape mount() accepts.
    return confineViewModule(source, endow)
  } catch (err) {
    console.warn('[shell] SES confinement failed; mounting unconfined:', err)
    return undefined
  }
}

async function mountRoute(pathname: string, opts: { module?: unknown } = {}): Promise<void> {
  const slot = document.getElementById('ship-content')
  if (!slot) throw new Error('shell: #ship-content missing from index.html')

  const token = ++latestMountToken
  // Notify devtools tabs of navigation.
  document.dispatchEvent(new Event('sui:navigate'))

  if (!pathname || pathname === '/') {
    // Landing page replaces the slot directly — dispose the previous
    // SUI handle since its bindings reference DOM we're about to wipe.
    disposeCurrentMount()
    publishScope(null)
    currentPathname = pathname || '/'
    document.body.dataset.view = ''
    return showLanding(slot)
  }

  // The runtime is imported directly from @ship/sui-runtime via the
  // importmap (see the top of main.ts) — the importmap is the single
  // source of truth, no window.__suiRuntime indirection. The earlier
  // IIFE bootstrap that put the runtime on window has been removed.

  // Reserve the viewId before the try so the catch can release it on a failed
  // mount — declaring it inside try left `nextViewId` out of scope in catch,
  // turning any mount failure into a ReferenceError instead of the error UI.
  const nextViewId = nextId()
  window.__suiCurrentViewId = nextViewId

  // Top-level route name (first path segment) for SES confinement and
  // body[data-view]. SES confinement is opt-in (localStorage), keyed
  // by the top-level name because each view's source lives at
  // `/_deck/views/<name>/view.js`.
  const viewName = viewTagOf(pathname)

  try {
    // Explicit override (HMR) wins; otherwise, when SES confinement is on,
    // produce the confined view module to pass to mount().
    let moduleOverride: unknown = opts.module
    if (moduleOverride === undefined && confineEnabled() && viewName) {
      moduleOverride = await confinedModuleFor(viewName)
    }

    // Mount the next view BEFORE disposing the previous. mount() writes
    // `target.innerHTML = payload.html` and inserts the new <style> as
    // a sibling of the slot — the OLD view's <style> stays adjacent
    // until we dispose, so there's never a frame between the old DOM
    // being cleared and the new CSS being attached. (Pre-fix: dispose
    // ran first, removing the old <style> immediately, leaving the
    // old DOM unstyled until the fetch + mount completed — visible as
    // CSS-loss flicker on every navigation, especially under load.)
    const previous = currentHandle
    const previousViewId = currentViewId
    const handle = await mount(slot, pathname, {
      cssScope: 'adjacent',
      // SUI views call `ship.<domain>.<op>()` inside `<data live>`
      // declarations. Without a `ship` SDK client wired to ctx,
      // `ship` is undefined and the data lambda throws — visible
      // as views stuck on `Loading…` because the patch never lands.
      ship: shipClient,
      // Keyed clients map — the shell's host identity is `ship`, so the
      // SDK client is injected under that key for views that declare
      // `<client name="ship" />`. (The legacy `ship` option above keeps
      // undeclared views on the ambient compatibility default.)
      clients: { ship: shipClient },
      // Host loader — the runtime is host-agnostic and doesn't know
      // the `/_deck/` URL scheme; this adapter supplies by-name or
      // by-path payload fetch (the daemon's resolver handles the
      // [param] walk).
      loader: suiLoader,
      // Shared `route` signal so views with `/x/:id`-style patterns
      // can read `route.params.id()`. Without this, mount.ts wires
      // `route: undefined` into setupCtx and dynamic-route controllers
      // crash with `route is not a function`.
      route: routeSignals,
      navigate,
      // HMR path passes a cache-busted, freshly-imported view module so
      // dynamic-import doesn't return the cached old module and leave the
      // rebuilt .sui stale until a hard reload.
      ...(moduleOverride !== undefined ? { module: moduleOverride as never } : {}),
    })

    // Race guard for rapid navigation: if a later mountRoute already
    // bumped the token, we've been superseded. The fresh mount we just
    // created is detached as soon as that later call's mount() runs
    // its `target.innerHTML = …`, but its SSE connection + <style>
    // would leak unless we dispose right now.
    if (token !== latestMountToken) {
      handle.dispose()
      // Superseded mid-flight — the later mountRoute call has already
      // bumped __suiCurrentViewId to its own id, so don't touch it here.
      return
    }

    currentHandle = handle
    currentPathname = pathname
    currentViewId = nextViewId
    // Tag the body with the active view's top-level route name so
    // chrome (nav-top, dock) can scope styles without parsing the
    // URL itself.
    document.body.dataset.view = viewName
    publishScope(handle.scope as unknown as Record<string, unknown>)
    // View-authoring chrome subscribes per (project, view). Pass the
    // top-level view name; the chrome reads the persisted edit-mode
    // signal and shows the dev panel + tabs if it's on for this view.
    setEditModeView(viewName)
    // The new mount has replaced slot.innerHTML, so the previous view's
    // DOM nodes are detached. Run its dispose to close SSE + remove the
    // stale <style> element. Dispose is idempotent.
    previous?.dispose()
    if (previousViewId) notifyViewUnmount(previousViewId)
  } catch (err) {
    // Mount failed (most often: loader 404). The runtime's `mount`
    // re-throws; we render the not-registered state and clear the
    // slot. No retry, no cache — a fresh navigation will ask the
    // daemon again.
    if (window.__suiCurrentViewId === nextViewId) {
      window.__suiCurrentViewId = currentViewId
    }
    notifyViewUnmount(nextViewId)
    disposeCurrentMount()
    currentPathname = pathname
    document.body.dataset.view = viewName
    publishScope(null)
    slot.innerHTML = `<div class="shell-error">
  <h2>Page not found</h2>
  <p>The path <code>${escapeHtml(pathname)}</code> isn't registered in this app.</p>
  <a href="/">← Home</a>
</div>`
  }
  updateNav(pathname)
}

function showLanding(slot: HTMLElement): void {
  // <sui-blank-canvas> paints the dotted-grid empty state with the brand
  // mark, "Blank canvas" wordmark, sub-copy, and four hint pills. Its
  // hints dispatch the same events the global hotkeys do.
  slot.innerHTML = '<sui-blank-canvas></sui-blank-canvas>'
  updateNav('/')
}

function updateNav(pathname: string): void {
  const path = pathname || '/'
  document.dispatchEvent(new CustomEvent('sui:navigate', { detail: { to: path } }))
}

// ── Click delegate ──────────────────────────────────────────────
//
// Intercepts internal `<a href="/...">` clicks so view-internal nav
//
// Intercepts internal `<a href="/...">` clicks so view-internal nav
// updates the URL + swaps the embedded view without a full reload.
// Uses composedPath() to catch anchors inside shadow DOM (e.g. those
// inside <sui-rail-item> shadow roots).

function findAnchor(e: MouseEvent): HTMLAnchorElement | null {
  // composedPath walks through shadow roots — catches shadow DOM links.
  if (typeof e.composedPath === 'function') {
    for (const el of e.composedPath()) {
      if (el instanceof HTMLAnchorElement) return el
    }
  }
  return (e.target as Element | null)?.closest?.('a[href]') as HTMLAnchorElement | null
}

function onAnchorClick(e: MouseEvent): void {
  if (e.defaultPrevented) return
  if (e.button !== 0) return
  if (e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return
  const a = findAnchor(e)
  if (!a) return
  if (a.target && a.target !== '' && a.target !== '_self') return
  if (a.hasAttribute('download')) return
  if (a.dataset.noRouter !== undefined) return
  if (a.getAttribute('href')?.startsWith('#')) return
  if (a.origin !== location.origin) return
  if (!a.pathname) return
  e.preventDefault()
  const url = a.pathname + a.search + a.hash
  // No "same route" guard here: the runtime's `mount` is idempotent
  // for the same path (its `previous` is the current handle, dispose
  // is a no-op if it already ran) and the same-shell no-remount
  // hack is gone — every navigation goes through one mount path.
  history.pushState({}, '', url)
  void mountRoute(a.pathname)
}
// (Old `readRouteFromUrl`, `notifySameShellRouteChange`, and the
// `SAME_SHELL_VIEWS` no-remount hack are gone. The URL is a path;
// every navigation re-mounts through one path. The runtime's route
// module updates its params signal on `popstate` and `pushState`
// uniformly — there's no special-case for "same view, new sub".)


// ── Favicon theme swap ───────────────────────────────────────────────
//
// The in-app brand mark (<sui-brand-mark> in the top bar) is token-
// painted, so it tracks every theme automatically. The browser-tab
// favicon lives outside any CSS scope, so we swap its `href` between
// the two available SVG variants on each theme change:
//   ship-logo.svg            — amber/gold (light themes)
//   ship-logo-secondary.svg  — purple     (dark themes)
//
// applyTheme() dispatches `ship:theme:change` with { dark } on every
// theme application — including the boot-time call from
// <sui-theme-picker>. We only need to listen.

function initFavicon(): void {
  const link = document.querySelector<HTMLLinkElement>('link[rel="icon"][type="image/svg+xml"]')
  if (!link) return
  const apply = (dark: boolean) => {
    const href = dark ? '/ship-logo-secondary.svg' : '/ship-logo.svg'
    if (link.getAttribute('href') !== href) link.setAttribute('href', href)
  }
  document.addEventListener('ship:theme:change', (e: Event) => {
    const detail = (e as CustomEvent).detail as { dark?: boolean } | undefined
    if (typeof detail?.dark === 'boolean') apply(detail.dark)
  })
}

// ── Bottom strip ─────────────────────────────────────────────────────
//
// <sui-shell-bottom> is glance-only: connection · branch · workspace ·
// tools · theme · version · mode. Branch updates every 15s from the
// daemon's /api/v1/git/status endpoint. Theme attribute reflects the
// current data-theme on <html> so theme switches re-paint the label
// without a poll. Version comes from <meta name="ship-version"> when
// shipd serves the HTML; static dev defaults to the empty string.

const shellBottom = document.getElementById('ship-shell-bottom') as HTMLElement | null
let daemonOnline = false

const shipMode = readShipMode()
if (shellBottom) {
  shellBottom.setAttribute('mode', shipMode.toLowerCase())
  const versionMeta = document.querySelector('meta[name="ship-version"]') as HTMLMetaElement | null
  if (versionMeta?.content) shellBottom.setAttribute('version', versionMeta.content)
}
document.dispatchEvent(new CustomEvent('ship:mode', { detail: { mode: shipMode } }))

// Branch label: live from the abstract `git.overview` registry op over the
// multiplex. Git is an L0 library behind the registry — there is no
// GitService on the wire; the status bar shares the same subscription the
// git view uses (deduped by the sdk-client's shared source).
{
  const gitSig = runtimeNs.signal<{ kind: string; value?: { branch?: string | null } }>({
    kind: 'loading',
  })
  shipClient.git.overview().subscribe(gitSig as never)
  runtimeNs.effect(() => {
    const s = gitSig()
    if (s.kind === 'ok' && shellBottom) {
      shellBottom.setAttribute('branch', s.value?.branch ?? 'unknown')
    }
  })
}

async function updateStatus(): Promise<void> {
  if (!shellBottom) return
  shellBottom.setAttribute('theme', document.documentElement.dataset.theme ?? 'ship-light')

  // Online indicator: the daemon health check. (The branch label updates
  // live via the git.overview subscription above, not this poll.)
  try {
    const res = await fetch(`${daemonBase()}/api/health`, { credentials: 'same-origin' })
    if (!res.ok) throw new Error(String(res.status))
    if (!daemonOnline) {
      shellBottom.setAttribute('online', '')
      daemonOnline = true
    }
  } catch {
    markOffline()
  }
}

function markOffline(): void {
  if (daemonOnline && shellBottom) {
    shellBottom.removeAttribute('online')
    shellBottom.setAttribute('branch', 'offline')
    daemonOnline = false
  }
}

// ── Toast bridge ────────────────────────────────────────────────────

interface SuiNotifyDetail {
  tone?: string
  title?: string
  message?: string
  description?: string
  duration?: number
  dismissible?: boolean
  showProgress?: boolean
  actionHtml?: string
}

interface ToastContainer extends HTMLElement {
  show(opts: {
    tone?: string
    title?: string
    description?: string
    duration?: number
    dismissible?: boolean
    showProgress?: boolean
    actionHtml?: string
  }): HTMLElement
}

function getToastContainer(): ToastContainer | null {
  return document.getElementById('sui-toast-container') as ToastContainer | null
}

function onSuiNotify(e: Event): void {
  const detail = ((e as CustomEvent).detail ?? {}) as SuiNotifyDetail
  const container = getToastContainer()
  if (!container) return
  const opts: Parameters<ToastContainer['show']>[0] = {}
  if (detail.tone !== undefined) opts.tone = detail.tone
  const title = detail.title ?? detail.message
  if (title !== undefined) opts.title = title
  if (detail.description !== undefined) opts.description = detail.description
  if (detail.duration !== undefined) opts.duration = detail.duration
  if (detail.dismissible !== undefined) opts.dismissible = detail.dismissible
  if (detail.showProgress !== undefined) opts.showProgress = detail.showProgress
  if (detail.actionHtml !== undefined) opts.actionHtml = detail.actionHtml
  container.show(opts)
}

document.addEventListener('sui:notify', onSuiNotify)

// ── Boot ────────────────────────────────────────────────────────────

// ── Panel dock ──────────────────────────────────────────────────────
//
// ⌘J / Ctrl+J toggles the dock. The dock element exposes show/hide/toggle
// as imperative methods; the shell owns open/closed state via body.dock-open.

interface PanelDock extends HTMLElement {
  show(): void
  hide(): void
  toggle(): void
  activate(tab: string): void
}

function getDock(): PanelDock | null {
  return document.getElementById('ship-panel-dock') as PanelDock | null
}

// `getCommandBar` is also used by `chrome/keymap.ts` (which has its own
// local copy for isolation); the rail "+" / status-bar code paths below
// call this one to open the bar in non-keymap-triggered modes.
interface CommandBar extends HTMLElement {
  show(mode?: string, query?: string): void
  hide(): void
  toggle(mode?: string): void
  setMode(mode: string): void
}
function getCommandBar(): CommandBar | null {
  return document.getElementById('ship-command-bar') as CommandBar | null
}

// ── Keyboard map ────────────────────────────────────────────────────
//
// The global keymap + the chrome-event bridges that pair with it
// (sui-toggle-terminal, sui:dock:close) live in `chrome/keymap.ts` —
// they're state-free relative to the shell's mount lifecycle, so they
// take a context bag and stay isolated from this file. See bindings
// table in chrome/keymap.ts.
registerKeymap({
  navigate: (path) => navigate(path),
  getCurrentViewName: () => viewTagOf(currentPathname ?? ''),
  toggleEditMode,
  toggleAnnotationMode,
  disposeCurrentMount,
  publishScope,
  showLanding,
  getDock,
})

// ── Chrome event wiring ─────────────────────────────────────────────
//
// Bridges component-dispatched events from <sui-nav-top> and <sui-rail>
// to the corresponding chrome elements that already live in the shell.
// Without these, clicking the menu / CLI / launcher / "+" buttons fired
// the event but no surface listened (pre-fix shell had no plumbing).

// Top-nav CLI / debug-panel button → toggle the panel-dock. The dock's
// `toggle()` method flips `body.dock-open`, which layout.css uses to
// reveal the dock surface.
document.addEventListener('sui:dock:toggle', () => getDock()?.toggle())

// First time the user opens the dock OR activates the Terminal tab,
// spawn a real shell PTY session via /api/v1/sessions/spawn + wire
// the EventSource ↔ ship-terminal IO. Idempotent — only spawns once.
// Listening on both because the dock may boot already-open OR already
// pointed at the terminal tab (persisted state), in which case
// `sui:dock:activate` won't refire.
function maybeBridgeTerm(): void {
  const dock = document.getElementById('ship-panel-dock') as (HTMLElement & { tabs?: Array<{id:string}> }) | null
  if (!dock?.hasAttribute('data-active') && !document.body.classList.contains('dock-open')) return
  // Spawn now whether terminal is active or not — connection is cheap
  // and the user can switch to Terminal expecting a live shell.
  void bridgeDockTerminal()
}
document.addEventListener('sui:dock:activate', (e) => {
  const tab = (e as CustomEvent<{ tab?: string }>).detail?.tab
  if (tab === 'terminal') void bridgeDockTerminal()
})
// Closing the dock or popping it out → tear down the PTY so the shell
// process doesn't orphan and a re-open spawns a fresh prompt. Pre-fix
// nothing called disposeDockTerminal, so the second open returned
// instantly via the `bridged` guard with no live session attached.
document.addEventListener('sui:dock:close', () => {
  document.body.classList.remove('dock-open')
  void disposeDockTerminal()
})
document.body.addEventListener('classList' as never, maybeBridgeTerm) // placeholder; real trigger below
const dockClassObserver = new MutationObserver(() => {
  if (document.body.classList.contains('dock-open')) {
    void bridgeDockTerminal()
  } else {
    void disposeDockTerminal()
  }
})
dockClassObserver.observe(document.body, { attributes: true, attributeFilter: ['class'] })
window.addEventListener('beforeunload', () => { void disposeDockTerminal() })

// Rail launcher button → open <sui-launcher>. The launcher listens for
// `sui:launcher:open` on document; the rail dispatches the legacy
// `ship:launcher:open` for parity with apps/shell wiring. Bridge so
// both names work without forcing the rail to know about the SUI
// island's listener name.
document.addEventListener('ship:launcher:open', () => {
  document.dispatchEvent(new CustomEvent('sui:launcher:open', { bubbles: true }))
})

// Rail "+" button → open the command bar in create mode. Parity with
// apps/shell where ship-command-bar listens for `ship:create:open` and
// calls openBar('create'). In shell the bar is `<sui-command-bar>`,
// which exposes the same imperative surface (`show(mode)`).
document.addEventListener('ship:create:open', () => {
  getCommandBar()?.show('create')
})

initProjectSwitch(kernelBridge, {
  // Cross-project chip mount-point was the legacy <sui-status-bar>'s
  // `end` slot. <sui-shell-bottom> doesn't expose a slot — wiring the
  // chip into the new bottom strip is a separate follow-up; for now
  // the kernel subscription still drives remount + refresh.
  statusBar: null,
  remount: () => { if (currentPathname) void mountRoute(currentPathname) },
  refreshStatus: () => updateStatus(),
})

document.addEventListener('click', onAnchorClick)
window.addEventListener('popstate', () => {
  // The runtime's route module updates its params signal on popstate;
  // the shell just re-mounts. No "same-shell" branch: every navigation
  // goes through one mount path, and `mount`'s previous-handle dispose
  // is idempotent.
  void mountRoute(location.pathname)
})
// (The old `sui:navigate {to}` *trigger* receiver is gone. The event is
// now a route-change *notification* dispatched by the shell itself in
// `updateNav`; the runtime's `navigate()` and the `setRouterPort`
// registration above are the trigger entry points. External code that
// still dispatches `sui:navigate {to}` will trigger a notification
// with no `to` (the listener here used to parse `to` and re-mount;
// `updateNav` fires after `mountRoute` already ran, so re-mounting on
// the same path is a no-op).)
initChromeRegistry()
// Command palette — there is ONE command surface: the `<sui-command-bar>`
// slotted into `<sui-nav-top>` (id `ship-command-bar`). It owns its own
// dialog, hosts `<sui-command-palette>`/`<sui-command-chat>` via shared
// scope, loads its own SDK data, and self-navigates through `sui:navigate`.
// ⌘K and the blank-canvas hint emit `sui-open-command-palette`; route that
// intent to the bar's `command` mode rather than mounting a second, parent-
// less palette on <body> (which threw because its `useShared` lookup found
// no `sui-command-bar` ancestor).
document.addEventListener('sui-open-command-palette', () => {
  getCommandBar()?.show('command')
})
// Help overlay — mount the `<sui-help-overlay>` chrome element once on
// body and toggle it from the `sui-open-help` intent the ⌘? binding (and
// the shell-top help affordance) emit. The element owns Esc / backdrop
// dismissal; we only flip its reflected `open` attribute open.
initHelpOverlay()
// View-authoring chrome — installs the real __viewMcp + edit-mode
// signal wiring. The prior initViewMocks() + mocks/ dir was deleted
// when the backend (view_edit_tool_api) shipped — its leftover
// setInterval was firing a fake annotation every 5s.
initEditModeChrome()
initShipApi()
initFavicon()
initHmr({
  getCurrentView: () => viewTagOf(currentPathname ?? '') || null,
  // mountRoute re-fetches the payload (dev mode bypasses SSR cache) and,
  // when given `module`, mounts using the cache-busted view bundle in
  // place of the cached one — so .sui edits land without a full reload.
  remount: (module) => {
    if (!currentPathname) return Promise.resolve()
    return module !== undefined ? mountRoute(currentPathname, { module }) : mountRoute(currentPathname)
  },
})
setInterval(() => void updateStatus(), 15000)
void updateStatus()

// Wire active project name into the bottom strip's workspace label. The
// nav-top and old status-bar are gone from the shell mount; the new
// shell-bottom carries the workspace name as a prop.
async function syncProject(): Promise<void> {
  try {
    const resp = await ops.space.list({})
    const proj = resp.spaces.find((s) => s.id === resp.activeId) ?? null
    const name = proj?.name || ''
    if (shellBottom) shellBottom.setAttribute('workspace', name)
    // Provide the active project to the runtime via the HostContext seam so
    // per-project runtime state (e.g. edit-mode keys) scopes correctly —
    // replaces the dead window.__shipProjectId global. See host-context.ts.
    if (proj?.name) setHostContext({ spaceId: proj.name })
    // Project identity in the top bar is now owned by <sui-project-switcher>
    // (it fetches + renders the active project itself); nav-top no longer takes
    // a `project` breadcrumb prefix.
  } catch { /* leave workspace blank when the API is unreachable */ }
}
void syncProject()
document.addEventListener('project.switched', () => void syncProject())

// Click bridges from the new chrome — same event names the legacy status-
// bar dispatched, but the project switcher / theme picker hosts now live
// elsewhere. Project switcher routes through the command bar; theme
// picker has no current host in the shell mount and so falls back to a
// document-level handler that the next theme-picker job can replace.
interface OpenableHost extends HTMLElement {
  toggle?: () => void
  open?: () => void
}
document.addEventListener('sui:project-switcher:open', () => {
  // The rail used to host the project switcher; without a rail, route
  // the request through the command bar where the user can pick a
  // project by name.
  getCommandBar()?.show('find')
})
document.addEventListener('sui:theme-picker:open', () => {
  // The shell no longer mounts a sui-theme-picker host. Look for one
  // mounted by a view (e.g. settings) and toggle it; otherwise no-op.
  const picker = document.querySelector<OpenableHost>('sui-theme-picker')
  if (picker?.toggle) { picker.toggle(); return }
  const trigger = picker?.shadowRoot?.querySelector<HTMLButtonElement>('.trigger')
  trigger?.click()
})

// Help overlay wiring — mount `<sui-help-overlay>` once on body, then
// open it on the `sui-open-help` intent. The element reflects its own
// `open` state and dismisses itself on Esc / backdrop click, so the
// shell only has to flip it open. Idempotent for HMR: the element is
// reused if already present.
function initHelpOverlay(): void {
  const HELP_ID = 'ship-help-overlay'
  let el = document.getElementById(HELP_ID)
  if (!el) {
    el = document.createElement('sui-help-overlay')
    el.id = HELP_ID
    document.body.appendChild(el)
  }
  const host = el
  document.addEventListener('sui-open-help', () => host.setAttribute('open', ''))
}

async function boot(): Promise<void> {
  // SES: lock the realm BEFORE the first mount so confineController's harden()
  // works (harden requires lockdown). Lazy + gated — default-off shells never
  // import deck-host/ses. Failure degrades to unconfined rather than blanking.
  if (confineEnabled()) {
    try {
      const { lockdownRealm } = await import('@ship/deck-host')
      lockdownRealm()
    } catch (err) {
      console.warn('[shell] lockdown failed; continuing unconfined:', err)
    }
  }
  // `/` IS home — keep the URL bare. The rail no longer carries a
  // separate Home item (the brand mark in nav-top is the home affordance).
  void mountRoute(location.pathname || '/')
}
void boot()