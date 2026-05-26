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
//   2. Fetch /_sui/views/<name>[/<sub>] → ViewPayload.
//   3. mount(content, payload) → MountHandle.
//   4. On internal anchor clicks → history.pushState + re-mount.
//   5. On popstate → re-mount.
//   6. Esc returns to the blank canvas; ⌘K opens the palette;
//      ⌘. navigates to /studio; ⌃` toggles the terminal panel;
//      ⌘? opens help.

import { createClient, type ShipSDK } from '@ship/sdk-client'
import { ops } from './ops'
// Charts pack — single workspace import registers every <sui-*-chart>,
// <sui-sparkline>, <sui-donut>, <sui-gauge>, <sui-meter>, <sui-heatmap>
// before the first view mounts. Charts moved out of @ship/ui-components
// into @ship/charts (packages/components/charts/) as the first pack
// extracted under the multi-pack architecture; consumers depend on the
// pack via standard workspace deps, exactly the same shape any future
// first- or third-party pack will use.
//
// The `/bundle` subpath points at sui-build's pre-built artifact
// (`dist/sui-charts.js`) — the same shape the daemon uses to serve
// `<script src="/_sui/components.js">` for the umbrella. Importing
// `@ship/charts` directly would route through the source tree's
// `.compiled.js` virtual imports, which only the sui-build / vitest
// resolvers understand. The bundle path is a real on-disk module
// Vite can ship as-is. Requires `bun run --filter @ship/charts build`
// to have produced `dist/sui-charts.js`.
import '@ship/charts/bundle'
// Pipelines pack — sui-pipeline-* primitives (card/dag/step/run/history/
// trigger/output/definition). Same prebuilt-bundle path as charts; requires
// `bun run --filter @ship/pipelines build` to have produced dist/sui-pipelines.js.
import '@ship/pipelines/bundle'
// Primitives pack — generic `<sui-*>` UI elements (button, input, dialog,
// popover, tooltip, tabs, etc.) — is registered by the eager
// `/_sui/components.js` umbrella loaded in index.html (its `@ship/ui-components`
// entry side-effect-imports `@ship/primitives`). That bundle loads in <head>
// before this module mounts, so a second `import '@ship/primitives/bundle'`
// here only re-ran every `customElements.define()` as a no-op — the source of
// ~90 "called twice" warnings. Custom elements upgrade lazily, so there's no
// need to register them a second time from the shell entry.
import { route as createRoute, defineRoutes, setRouterPort } from '@ship/sui-runtime/route'
import { STUDIO_ROUTE_PATTERNS } from './studio/routes.gen'
import { chrome, notifyViewUnmount, setHostContext } from '@ship/sui-runtime'
import type { SuiLoader, ViewPayload } from '@ship/sui-runtime'
import { KernelBridge } from './kernel-bridge'
import { initHmr } from './hmr'
import { initProjectSwitch } from './project-switch'
import { initChromeRegistry } from './chrome/registry'
import { initEditModeChrome, setEditModeView, toggleEditMode, toggleAnnotationMode } from './chrome/edit-mode'
import { bridgeDockTerminal, disposeDockTerminal } from './chrome/terminal-bridge'

interface MountFn {
  (
    target: HTMLElement,
    payload: unknown,
    opts?: {
      cssScope?: 'adjacent' | 'document-head' | 'none'
      ship?: ShipSDK
      loader?: SuiLoader
      module?: unknown
      route?: unknown
      navigate?: (path: string) => void
    },
  ): Promise<{ dispose(): void; element: HTMLElement; scope: Record<string, unknown> }>
}

// The daemon's base origin. Empty in a browser/vite context (same-origin; the
// vite proxy forwards /api + /_sui + Connect to the daemon). In the proxy-less
// Tauri webview, the host plants `window.__SHIP_DAEMON_URL__` =
// `http://127.0.0.1:<port>` so every daemon call targets the absolute loopback
// origin directly (no proxy — the doc's Connect-over-loopback model).
function daemonBase(): string {
  return (window as WindowWithSui).__SHIP_DAEMON_URL__?.replace(/\/$/, '') ?? ''
}

// Single SDK client instance, shared across every mounted view.
const shipClient: ShipSDK = createClient({ baseUrl: daemonBase() })

// Host loader injected into every mount() — the `/_sui/` URL scheme lives
// here at the app boundary, not inside the (host-agnostic) SUI runtime.
const suiLoader: SuiLoader = {
  async fetchView(nameOrPath: string): Promise<ViewPayload> {
    const rel = nameOrPath.startsWith('/_sui/views/') ? nameOrPath : `/_sui/views/${nameOrPath}`
    const path = `${daemonBase()}${rel}`
    const res = await fetch(path, { credentials: 'same-origin' })
    if (!res.ok) {
      const body = await res.text().catch(() => '')
      throw new Error(`[shell] fetch ${path} ${res.status}: ${body.slice(0, 200)}`)
    }
    return (await res.json()) as ViewPayload
  },
  loadComponent: (tag: string): Promise<void> =>
    import(/* @vite-ignore */ `${daemonBase()}/_sui/sui-components/${tag}.js`).then(() => undefined),
}

// Pre-register dynamic-route patterns the shell knows about. Views in
// `views/<name>/[id]/` use these to read `route.params.id()`. Views
// can also register their own at controller-import time, but those
// registrations land in the view-bundle's copy of @ship/sui-runtime
// (served via `/_sui/runtime.js`) — a separate module instance from
// the shell's bundle of @ship/sui-runtime, so the shell's
// `routeSignals` proxy never sees them. Registering here at the shell
// layer means params get seeded against the SHAPES the shell knows.
defineRoutes([
  // Daemon-view dynamic params (no studio [param] dir backs these yet —
  // a follow-up can codegen views/ too). Studio's nested tool routes are
  // generated from the `[param]` dir tree (SSOT). See ADR unified-deck-router.
  { pattern: '/artifacts/:id' },
  { pattern: '/workspaces/:id' },
  ...STUDIO_ROUTE_PATTERNS,
])

// Shared route signal. Views that declare dynamic params read
// `route.params.id()`; the singleton updates on every popstate /
// history.pushState the shell triggers.
const routeSignals = createRoute()

// The shell's router. `history.pushState` (or replace) + match + mount-or-
// update. Registered as the runtime's `RouterPort` below, so a single
// `navigate()` from any view bundle routes through here and actually mounts.
function navigate(path: string, opts: { replace?: boolean } = {}): void {
  if (opts.replace) history.replaceState({}, '', path)
  else history.pushState({}, '', path)
  const next = readRouteFromUrl(location.pathname)
  if (!next) return
  if (currentRoute && currentRoute.view === next.view && SAME_SHELL_VIEWS.has(next.view)) {
    currentRoute = next
    notifySameShellRouteChange()
    updateNav(next)
    return
  }
  void mountRoute(next)
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

interface SuiRuntime {
  mount: MountFn
}

// Window globals set by index.html's IIFE bootstrap. Cast at the call
// sites to avoid `declare global` in a non-ambient module file.
type WindowWithSui = Window & {
  __suiRuntime?: SuiRuntime
  __suiRuntimeReady?: Promise<SuiRuntime>
  // Planted by the Tauri host (pre-load init script) so the proxy-less webview
  // targets the daemon's absolute loopback origin. Undefined in a browser/vite
  // context → '' → same-origin relative URLs (the vite proxy resolves them).
  __SHIP_DAEMON_URL__?: string
  // The space this window was launched for (`--app=<id>`); undefined = the hub.
  __SHIP_APP_ID__?: string
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
}

// ── Routing ─────────────────────────────────────────────────────────

interface Route {
  /** First path segment, e.g. "ship-ui-studio". */
  view: string
  /** Remaining path segments joined by "/", e.g. "sui-button". */
  sub?: string
}

// Empty path → agents. Launch priorities (Agents · Skills · Collections ·
// Artifacts · Workspaces · Settings) put Agents first on the rail, so
// landing on `/` mounts the agents view rather than the deprioritized
// dashboard. Pre-shift this returned `{ view: 'dashboard' }`.
function readRoute(): Route | null {
  const segs = location.pathname.split('/').filter(Boolean)
  // `/` is home — the brand mark in nav-top links here. No separate
  // /home URL needed; the home view is mounted at the root path.
  if (segs.length === 0) return { view: 'home' }
  // Legacy `/home` URLs (and the redirect from the daemon) still need
  // to map to the home view, just rendered at the same surface.
  if (segs[0] === 'home' && segs.length === 1) return { view: 'home' }
  return { view: segs[0]!, ...(segs.length > 1 ? { sub: segs.slice(1).join('/') } : {}) }
}

function routeToPath(r: Route): string {
  return r.sub ? `/${r.view}/${r.sub}` : `/${r.view}`
}

// Views whose root payload (no `sub`) renders the entire surface for every
// `/<view>/<id>` URL too. The detail is just a route param read by the
// view's controller via `route.params.id()`, so navigating between
// list/detail (or sibling detail rows) doesn't need a full mount-swap —
// the URL changes, routeSignals fire, the controller reacts in place.
// Without this, every sibling click re-fetches the payload + rebuilds
// the entire DOM, which the user perceives as a "page reload" flash.
// `studio` is intentionally NOT in this set: it's a deck view whose tool
// routes (/studio/agents, /studio/agents/:id, …) are distinct daemon
// payloads resolved from `views/studio/**`, so each nav is a normal
// mountRoute — the shared rail (<sui-studio-rail>) re-renders against the
// fresh URL. Same-shell would suppress that fetch and pin the first leaf.
// Empty since skills moved to deck-routed /studio/skills (list + [id] detail);
// no view now needs the same-shell no-remount hack.
const SAME_SHELL_VIEWS = new Set<string>([])

function payloadUrl(r: Route): string {
  if (SAME_SHELL_VIEWS.has(r.view)) return `/_sui/views/${r.view}`
  return r.sub ? `/_sui/views/${r.view}/${r.sub}` : `/_sui/views/${r.view}`
}

// ── Mount lifecycle ─────────────────────────────────────────────────

// Each `mountRoute` call gets a monotonic token. When its mount resolves,
// we check the token against `latestMountToken`: if equal, we're the
// latest navigation and become the current handle; otherwise we were
// superseded mid-flight and immediately dispose ourselves so SSE
// connections + style elements don't leak.
let currentHandle: { dispose(): void; scope?: Record<string, unknown> } | null = null
let latestMountToken = 0
let currentRoute: Route | null = null
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

// ── SES confinement (opt-in) ─────────────────────────────────────────────────
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
// endow the SHARED runtime (window.__suiRuntime — the importmap instance, NOT a
// second copy, so signals stay one universe per #149) + dom + policy-gated
// ship/route/navigate + the real document, and confine under lockdown. Returns
// undefined on any failure (incl. non-daemon views like bundled studio, whose
// view.js 404s) → caller falls back to the normal unconfined path.
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
    const res = await fetch(`${daemonBase()}/_sui/views/${viewName}/view.js`, { credentials: 'same-origin' })
    if (!res.ok) return undefined
    const source = await res.text()
    const w = window as WindowWithSui
    const endow = buildViewEndowments({
      runtime: (w.__suiRuntime ?? {}) as unknown as Record<string, unknown>,
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

async function mountRoute(r: Route, opts: { module?: unknown } = {}): Promise<void> {
  const slot = document.getElementById('ship-content')
  if (!slot) throw new Error('shell: #ship-content missing from index.html')

  const token = ++latestMountToken

  // Notify devtools tabs of navigation.
  document.dispatchEvent(new Event('sui:navigate'))

  if (!r) {
    // Landing page replaces the slot directly — dispose the previous
    // SUI handle since its bindings reference DOM we're about to wipe.
    disposeCurrentMount()
    publishScope(null)
    return showLanding(slot)
  }

  // First-party CHROME surfaces (apps/shell/src/components/*) — rendered
  // directly as registered custom elements, NOT daemon-served views. They
  // self-mount via customElements + the SUI runtime. Map of route → tag.
  // Each first-party chrome surface declares a chrome `layout`. The shell
  // applies it via body[data-layout], which layout.css honors — e.g. "desktop"
  // hides the status strip (home owns its own dock). This is the first-party
  // layout contract (GAP-5): only these declare a layout; daemon/3rd-party
  // views are always "default" and get no chrome privilege.
  const chromeSurface = !r.sub
    ? ({
        home:     { tag: 'sui-home',        layout: 'desktop' },
        activity: { tag: 'sui-activity' },
        account:  { tag: 'sui-account' },
        library:  { tag: 'sui-library' },
        setup:    { tag: 'sui-setup' },
        settings: { tag: 'sui-settings' },
        canvas:   { tag: 'sui-canvas-app' },
      } as Record<string, { tag: string; layout?: string }>)[r.view]
    : undefined
  if (chromeSurface) {
    disposeCurrentMount()
    publishScope(null)
    slot.innerHTML = `<${chromeSurface.tag}></${chromeSurface.tag}>`
    document.body.dataset.view = r.view
    document.body.dataset.layout = chromeSurface.layout ?? 'default'
    currentRoute = r
    currentViewId = null
    return
  }

  const w = window as WindowWithSui
  const runtime = await (w.__suiRuntimeReady ?? Promise.resolve(w.__suiRuntime))
  if (!runtime?.mount) throw new Error('shell: SUI runtime did not load')

  // Reserve the viewId before the try so the catch can release it on a failed
  // mount — declaring it inside try left `nextViewId` out of scope in catch,
  // turning any mount failure into a ReferenceError instead of the error UI.
  const nextViewId = nextId()
  window.__suiCurrentViewId = nextViewId

  try {
    // Explicit override (HMR) wins; otherwise, when SES confinement is on,
    // produce the confined view module to pass to mount().
    let moduleOverride: unknown = opts.module
    if (moduleOverride === undefined && confineEnabled()) {
      moduleOverride = await confinedModuleFor(r.view)
    }
    const res = await fetch(payloadUrl(r), { credentials: 'same-origin' })
    if (!res.ok) {
      disposeCurrentMount()
      const viewPath = escapeHtml(`${r.view}${r.sub ? '/' + r.sub : ''}`)
      slot.innerHTML = `<div class="shell-error">
  <h2>Page not found</h2>
  <p>The view <code>${viewPath}</code> isn't registered.</p>
  <a href="/">← Home</a>
</div>`
      return
    }
    const payload: unknown = await res.json()

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
    const handle = await runtime.mount(slot, payload, {
      cssScope: 'adjacent',
      // SUI views call `ship.<domain>.<op>()` inside `<data live>`
      // declarations. Without a `ship` SDK client wired to ctx,
      // `ship` is undefined and the data lambda throws — visible
      // as views stuck on `Loading…` because the patch never lands.
      ship: shipClient,
      // Host loader — the SUI runtime no longer knows the `/_sui/` URL scheme;
      // this adapter supplies by-name payload fetch + component-island loading.
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
      ...(moduleOverride !== undefined ? { module: moduleOverride } : {}),
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
    currentRoute = r
    currentViewId = nextViewId
    // Tag the body with the active view; daemon-served views get the "default"
    // chrome layout (no privilege). First-party chrome surfaces declare their
    // own layout above. (When the daemon surfaces manifest `layout` in the
    // payload, read it here to extend the contract to first-party daemon views
    // — PLATFORM-GAPS GAP-5 part b.)
    document.body.dataset.view = r.view
    document.body.dataset.layout = 'default'
    publishScope(handle.scope as unknown as Record<string, unknown>)
    // View-authoring chrome subscribes per (project, view). Pass the
    // view name; the chrome reads the persisted edit-mode signal and
    // shows the dev panel + tabs if it's on for this view.
    setEditModeView(r.view)
    // The new mount has replaced slot.innerHTML, so the previous view's
    // DOM nodes are detached. Run its dispose to close SSE + remove the
    // stale <style> element. Dispose is idempotent.
    previous?.dispose()
    if (previousViewId) notifyViewUnmount(previousViewId)
  } catch (err) {
    // Mount failed: release the viewId we reserved so the registry
    // doesn't keep claims from a half-mounted view alive.
    if (window.__suiCurrentViewId === nextViewId) {
      window.__suiCurrentViewId = currentViewId
    }
    notifyViewUnmount(nextViewId)
    slot.innerHTML = `<div class="shell-error">
  <h2>Mount failed</h2>
  <p>${escapeHtml((err as Error).message)}</p>
  <a href="/">← Home</a>
</div>`
  }
  updateNav(r)
}

function showLanding(slot: HTMLElement): void {
  // <sui-blank-canvas> paints the dotted-grid empty state with the brand
  // mark, "Blank canvas" wordmark, sub-copy, and four hint pills. Its
  // hints dispatch the same events the global hotkeys do.
  slot.innerHTML = '<sui-blank-canvas></sui-blank-canvas>'
  currentRoute = null
  updateNav(null)
}

function updateNav(r: Route | null): void {
  const path = r ? routeToPath(r) : '/'
  document.dispatchEvent(new CustomEvent('sui:navigate', { detail: { to: path } }))
}

// ── Click delegate ──────────────────────────────────────────────────
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

  const next = readRouteFromUrl(a.pathname)
  if (!next) return
  e.preventDefault()
  // Preserve the anchor's own pathname for the URL (so `href="/"` stays
  // as `/` and survives a reload), while resolving the route to a view
  // name for the SUI mount call.
  const urlPath = a.pathname || '/'
  const sameRoute = currentRoute && currentRoute.view === next.view && (currentRoute.sub ?? '') === (next.sub ?? '')
  if (sameRoute) return
  history.pushState({}, '', urlPath + a.search + a.hash)
  // Same-shell views (see SAME_SHELL_VIEWS) just need a route-signal kick
  // so the in-place controller reacts; no payload re-fetch, no mount swap.
  if (currentRoute && currentRoute.view === next.view && SAME_SHELL_VIEWS.has(next.view)) {
    currentRoute = next
    notifySameShellRouteChange()
    updateNav(next)
    return
  }
  void mountRoute(next)
}

// Same-shell navigations (see SAME_SHELL_VIEWS) don't remount the view —
// the URL changes via pushState, but the view's controller is still the
// one set up at the original mount, which means its compiled `derived`
// reads of `route.params.id()` were registered against the SHELL bundle's
// signal observer. The VIEW bundle has its own copy of @ship/sui-runtime
// (served as `/_sui/runtime.js`) with its own CURRENT_OBSERVER closure,
// so cross-bundle signal subscription doesn't fire view-bundle effects.
// We compensate by both:
//   (a) firing shell-bundle routeSignals' update — for any derived that
//       happens to live in the shell bundle (rare, but defensible);
//   (b) dispatching a synthetic `popstate` event, which the view bundle's
//       route module listens for and uses to call its own update()
//       against the view bundle's signal subscribers.
function notifySameShellRouteChange(): void {
  globalThis.__shipSetRouteUpdate?.()
  window.dispatchEvent(new PopStateEvent('popstate'))
}

function readRouteFromUrl(pathname: string): Route | null {
  const segs = pathname.split('/').filter(Boolean)
  if (segs.length === 0) return { view: 'home' }
  if (segs[0] === 'home' && segs.length === 1) return { view: 'home' }
  return { view: segs[0]!, ...(segs.length > 1 ? { sub: segs.slice(1).join('/') } : {}) }
}

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

async function updateStatus(): Promise<void> {
  if (!shellBottom) return
  shellBottom.setAttribute('theme', document.documentElement.dataset.theme ?? 'ship-light')

  // Fetch the active project's actual git branch via GitService.Overview
  // (Connect). Branch lives on the overview, not the status buckets.
  try {
    const overview = await ops.git.overview({})
    const branch = overview.branch ?? 'unknown'
    shellBottom.setAttribute('branch', branch)
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

// ── Keyboard map ────────────────────────────────────────────────────
//
// ⌘K/Ctrl+K  → command-bar toggle (find mode)
// ⌘P/Ctrl+P  → command-bar search (find mode)
// ⌘J/Ctrl+J  → panel-dock toggle
// Alt+E → per-view edit-mode toggle (was ⌘⇧E; collided with browser
//   tab-search / devtools shortcuts on multiple platforms).
// ⌘B/Ctrl+B  → rail toggle (via sui:rail:toggle)
// Esc        → close open sui-dialog / sui-drawer (backstop)

interface CommandBar extends HTMLElement {
  show(mode?: string, query?: string): void
  hide(): void
  toggle(mode?: string): void
  setMode(mode: string): void
}
function getCommandBar(): CommandBar | null {
  return document.getElementById('ship-command-bar') as CommandBar | null
}

document.addEventListener('keydown', (e: KeyboardEvent) => {
  const mod = e.metaKey || e.ctrlKey
  // `e.key` reports the produced character: Chromium yields lowercase
  // 'b' for Ctrl+B without Shift, but some automation drivers (and a few
  // platform keyboard layouts) emit uppercase. Normalize so the
  // shortcuts trigger regardless of caller.
  const key = e.key.toLowerCase()

  if (mod && key === 'j') {
    e.preventDefault()
    getDock()?.toggle()
    return
  }

  if (mod && key === 'k') {
    e.preventDefault()
    // ⌘K is the command palette — the shell's primary action surface. It
    // and the blank-canvas hint emit the bare `sui-open-command-palette`
    // intent, which the shell routes to the one `<sui-command-bar>` (see
    // its listener near init). ⌘P opens the same bar in `find` mode.
    document.dispatchEvent(new CustomEvent('sui-open-command-palette', { bubbles: true }))
    return
  }

  if (mod && key === 'p') {
    e.preventDefault()
    getCommandBar()?.show('find')
    return
  }

  // ⌘. / Ctrl+. — open Studio (the views/studio deck view).
  if (mod && e.key === '.') {
    e.preventDefault()
    navigate('/studio')
    return
  }

  // ⌘? / Ctrl+? — open help (stub event; help surface is a future job).
  if (mod && (e.key === '?' || (e.shiftKey && key === '/'))) {
    e.preventDefault()
    document.dispatchEvent(new CustomEvent('sui-open-help', { bubbles: true }))
    return
  }

  // ⌃` — toggle terminal panel. Uses ctrlKey explicitly (not Meta) per
  // the spec: ⌃ is the macOS Control key, not Command.
  if (e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey && e.key === '`') {
    e.preventDefault()
    document.dispatchEvent(new CustomEvent('sui-toggle-terminal', { bubbles: true }))
    return
  }

  if (e.altKey && !mod && !e.shiftKey && key === 'e') {
    e.preventDefault()
    // Per-view edit mode: flip the persisted signal for the current
    // view. The edit-mode chrome (see chrome/edit-mode.ts) reacts and
    // swaps the dock tabs + projects the view-authoring surfaces.
    // Alt+E replaced the prior ⌘⇧E/Ctrl+Shift+E binding because of
    // browser collisions (Firefox tab search, Chrome devtools toggle).
    if (currentRoute) toggleEditMode(currentRoute.view)
    return
  }

  if (e.altKey && e.shiftKey && !mod && key === 'e') {
    e.preventDefault()
    // Annotation mode (sub-toggle inside edit mode). No-op when edit
    // mode is off — the runtime guards this; the keyboard binding
    // just relays.
    if (currentRoute) toggleAnnotationMode(currentRoute.view)
    return
  }

  if (mod && key === 'b') {
    e.preventDefault()
    document.dispatchEvent(new CustomEvent('sui:rail:toggle', { bubbles: true }))
    return
  }

  if (mod && key === 'n') {
    e.preventDefault()
    document.dispatchEvent(new CustomEvent('ship:create:open', { bubbles: true }))
    return
  }

  if (e.key === 'Escape') {
    // Native <dialog> handles Esc already. Backstop: close any open
    // sui-dialog or sui-drawer that may not be using the native dialog element.
    let closedAny = false
    for (const el of document.querySelectorAll<HTMLElement & { close(): void }>(
      'sui-dialog[open], sui-drawer[open]',
    )) {
      try { el.close(); closedAny = true } catch { /* ignore — native handles it */ }
    }
    // If nothing dismissable was open and a view is currently mounted,
    // return to the blank canvas. The spec calls for Esc to clear the
    // active view's content from the canvas slot when no other modal
    // surface is consuming the key.
    if (!closedAny && currentRoute) {
      e.preventDefault()
      history.pushState({}, '', '/')
      const slot = document.getElementById('ship-content')
      if (slot) {
        disposeCurrentMount()
        publishScope(null)
        showLanding(slot)
      }
    }
  }
})

// Bridge `sui-toggle-terminal` → panel-dock toggle. The terminal lives
// inside the dock; the canonical "terminal" UX is the dock with its
// terminal tab activated. activate() is idempotent.
document.addEventListener('sui-toggle-terminal', () => {
  const dock = getDock()
  if (!dock) return
  dock.activate('terminal')
  dock.toggle()
})

// Wire close event from dock.
document.addEventListener('sui:dock:close', () => getDock()?.hide(), true)

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
// the EventSource ↔ sui-terminal IO. Idempotent — only spawns once.
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
  remount: () => { if (currentRoute) void mountRoute(currentRoute) },
  refreshStatus: () => updateStatus(),
})

document.addEventListener('click', onAnchorClick)
window.addEventListener('popstate', () => {
  const r = readRoute()
  if (!r) {
    const slot = document.getElementById('ship-content')
    if (slot) showLanding(slot)
    return
  }
  // Same-shell views stay mounted; routeSignals already update from the
  // shared popstate listener in @ship/sui-runtime/route, so the in-place
  // controller reacts to the new params without a payload re-fetch.
  if (currentRoute && currentRoute.view === r.view && SAME_SHELL_VIEWS.has(r.view)) {
    currentRoute = r
    updateNav(r)
    return
  }
  void mountRoute(r)
})

// Legacy `sui:navigate {to}` trigger receiver. Navigation now flows through
// the runtime `navigate()` → `RouterPort` (registered above), so first-party
// callers no longer dispatch this as a trigger — `sui:navigate` is the
// route-change *notification* the rails/nav-top/devtools subscribe to. This
// listener stays as a compatibility shim for any external `{to}` dispatch;
// the `sameRoute` guard makes the notification dispatches (no detail, or
// fired after currentRoute updates) no-ops, so there is no remount loop.
document.addEventListener('sui:navigate', (e: Event) => {
  const detail = (e as CustomEvent).detail as { to?: string } | undefined
  if (!detail?.to) return
  const next = readRouteFromUrl(detail.to)
  if (!next) return
  const sameRoute = currentRoute
    && currentRoute.view === next.view
    && (currentRoute.sub ?? '') === (next.sub ?? '')
  if (sameRoute) return
  if (currentRoute && currentRoute.view === next.view && SAME_SHELL_VIEWS.has(next.view)) {
    currentRoute = next
    notifySameShellRouteChange()
    updateNav(next)
    return
  }
  void mountRoute(next)
})

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
  getCurrentView: () => currentRoute?.view ?? null,
  // mountRoute re-fetches the payload (dev mode bypasses SSR cache) and,
  // when given `module`, mounts using the cache-busted view bundle in
  // place of the cached one — so .sui edits land without a full reload.
  remount: (module) => {
    const r = currentRoute
    if (!r) return Promise.resolve()
    return module !== undefined ? mountRoute(r, { module }) : mountRoute(r)
  },
})
setInterval(() => void updateStatus(), 15000)
void updateStatus()

// Wire active project name into the bottom strip's workspace label. The
// nav-top and old status-bar are gone from the shell mount; the new
// shell-bottom carries the workspace name as a prop.
async function syncProject(): Promise<void> {
  try {
    const body = JSON.parse((await ops.project.listProjects({})).json) as {
      projects?: Array<{ id: string; name?: string; label?: string }>
      active?: string
    }
    const proj = body.projects?.find((p) => p.id === body.active) ?? null
    const name = proj?.label || proj?.name || ''
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
  const initial = readRoute()
  if (initial) {
    // `/` IS home — keep the URL bare. The rail no longer carries a
    // separate Home item (the brand mark in nav-top is the home affordance),
    // so there's no rail-tile to highlight for the home route.
    void mountRoute(initial)
  } else {
    const slot = document.getElementById('ship-content')
    if (slot) showLanding(slot)
  }
}
void boot()
