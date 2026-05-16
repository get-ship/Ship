// Shell 2.0 — SUI-native shell.
//
// One Vite app per peer with apps/shell, but everything below the top
// bar is a SUI view mounted via the runtime's mount() function. The
// shell's job:
//
//   1. Read location.pathname → derive the current view name + sub.
//   2. Fetch /_sui/views/<name>[/<sub>] → ViewPayload.
//   3. mount(content, payload) → MountHandle.
//   4. On internal anchor clicks → history.pushState + re-mount.
//   5. On popstate → re-mount.
//   6. Theme: <sui-theme-picker> owns it (mounted in the top bar).
//   7. Rail: <sui-rail> along the left edge; persists collapsed state.
//   8. Status bar: <sui-status-bar> along the bottom; polls workspaces.

import { createClient, type ShipSDK } from '@ship/sdk-client'
import { shipFetch, gitStatus } from '@ship/sdk'
import { route as createRoute, defineRoutes } from '@ship/sui-runtime/route'
import { chrome, notifyViewUnmount } from '@ship/sui-runtime'
import { KernelBridge } from './kernel-bridge'
import { initHmr } from './hmr'
import { initProjectSwitch } from './project-switch'
import { initChromeRegistry } from './chrome/registry'
import { ensureSetupOrRedirect } from './setup-wizard'

interface MountFn {
  (
    target: HTMLElement,
    payload: unknown,
    opts?: {
      cssScope?: 'adjacent' | 'document-head' | 'none'
      ship?: ShipSDK
      module?: unknown
      route?: unknown
      navigate?: (path: string) => void
    },
  ): Promise<{ dispose(): void; element: HTMLElement; scope: Record<string, unknown> }>
}

// Single SDK client instance, shared across every mounted view.
const shipClient: ShipSDK = createClient()

// Pre-register dynamic-route patterns the shell knows about. Views in
// `views/<name>/[id]/` use these to read `route.params.id()`. Views
// can also register their own at controller-import time, but those
// registrations land in the view-bundle's copy of @ship/sui-runtime
// (served via `/_sui/runtime.js`) — a separate module instance from
// the shell's bundle of @ship/sui-runtime, so the shell's
// `routeSignals` proxy never sees them. Registering here at the shell
// layer means params get seeded against the SHAPES the shell knows.
defineRoutes([
  { pattern: '/agents/:id' },
  { pattern: '/skills/:id' },
  { pattern: '/collections/:id' },
  { pattern: '/artifacts/:id' },
  { pattern: '/workspaces/:id' },
  { pattern: '/ship-ui-studio/:component' },
])

// Shared route signal. Views that declare dynamic params read
// `route.params.id()`; the singleton updates on every popstate /
// history.pushState the shell triggers.
const routeSignals = createRoute()

// `navigate(path)` passed to controllers — fire a same-origin link click
// equivalent via history.pushState + the existing internal-anchor flow.
function navigate(path: string): void {
  history.pushState({}, '', path)
  const next = readRouteFromUrl(location.pathname)
  if (next) void mountRoute(next)
}

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
    /** Daemon operating mode, derived from `<meta name="ship-mode">` at
     *  boot. `'Production'` when shipd serves prebuilt shell HTML in
     *  production; `'Development'` everywhere else (Vite dev server,
     *  un-rewritten static index). The status bar renders a DEV badge
     *  whenever this is `'Development'`. */
    __shipMode?: 'Production' | 'Development'
    /** Identifier for the currently-mounted view. Read by
     *  `@ship/sui-runtime/chrome`'s wrapper API to stamp per-view claims
     *  (breadcrumb, status keys, theme overrides, focus push). Set on
     *  each mountRoute and cleared on landing/dispose so claims auto-
     *  clear when shell2 fires `sui:view:unmount`. */
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
  if (segs.length === 0) return { view: 'agents' }
  return { view: segs[0]!, ...(segs.length > 1 ? { sub: segs.slice(1).join('/') } : {}) }
}

function routeToPath(r: Route): string {
  return r.sub ? `/${r.view}/${r.sub}` : `/${r.view}`
}

function payloadUrl(r: Route): string {
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

  const w = window as WindowWithSui
  const runtime = await (w.__suiRuntimeReady ?? Promise.resolve(w.__suiRuntime))
  if (!runtime?.mount) throw new Error('shell: SUI runtime did not load')

  try {
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
    const payload = await res.json()

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
    const nextViewId = nextId()
    window.__suiCurrentViewId = nextViewId
    const handle = await runtime.mount(slot, payload, {
      cssScope: 'adjacent',
      // SUI views call `ship.<domain>.<op>()` inside `<data live>`
      // declarations. Without a `ship` SDK client wired to ctx,
      // `ship` is undefined and the data lambda throws — visible
      // as views stuck on `Loading…` because the patch never lands.
      ship: shipClient,
      // Shared `route` signal so views with `/x/:id`-style patterns
      // can read `route.params.id()`. Without this, mount.ts wires
      // `route: undefined` into setupCtx and dynamic-route controllers
      // crash with `route is not a function`.
      route: routeSignals,
      navigate,
      // HMR path passes a cache-busted, freshly-imported view module.
      // Without this, dynamic-import returns the cached old module and
      // the rebuilt .sui never takes effect until a hard reload.
      ...(opts.module !== undefined ? { module: opts.module } : {}),
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
    publishScope(handle.scope as unknown as Record<string, unknown>)
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
  slot.innerHTML = `
    <div class="shell-empty">
      <h2>Ship</h2>
      <p>Open a view: <a href="/agents">/agents</a> · <a href="/skills">/skills</a> · <a href="/collections">/collections</a> · <a href="/artifacts">/artifacts</a></p>
    </div>
  `
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
  const sameView = currentRoute && currentRoute.view === next.view && (currentRoute.sub ?? '') === (next.sub ?? '')
  if (sameView) return
  history.pushState({}, '', urlPath + a.search + a.hash)
  void mountRoute(next)
}

function readRouteFromUrl(pathname: string): Route | null {
  const segs = pathname.split('/').filter(Boolean)
  if (segs.length === 0) return { view: 'agents' }
  return { view: segs[0]!, ...(segs.length > 1 ? { sub: segs.slice(1).join('/') } : {}) }
}

// ── Rail visibility persistence ──────────────────────────────────────
//
// Two states only: visible (icon-only, 48px wide) or hidden (display:none).
// The arrow toggle on the rail was removed — the top-nav menu button
// dispatches `sui:rail:toggle` on document; this listener flips the
// `[collapsed]` attribute on the rail. Persisted under `ship-rail-open`:
// presence of the key means "visible". Default is visible on first load
// so users see the nav before being asked to discover the menu button.

const RAIL_OPEN_KEY = 'ship-rail-open'

function initRail(): void {
  const rail = document.getElementById('ship-rail')
  if (!rail) return

  // Default to visible. Users who explicitly hide the rail get an
  // explicit `0` written (we don't rely on key-absence-as-default since
  // that fights the "default visible" intent).
  let open = true
  try {
    const v = localStorage.getItem(RAIL_OPEN_KEY)
    if (v === '0') open = false
  } catch { /* best-effort */ }
  applyRail(rail, open)

  // sui:rail:toggle — fired by sui-nav-top's menu button (toggle request)
  document.addEventListener('sui:rail:toggle', () => {
    const nextOpen = rail.hasAttribute('collapsed') // currently hidden → show it
    applyRail(rail, nextOpen)
    persistRailOpen(nextOpen)
  })
}

function applyRail(rail: HTMLElement, open: boolean): void {
  if (open) {
    rail.removeAttribute('collapsed')
    document.body.classList.add('rail-open')
    document.body.classList.remove('rail-hidden')
  } else {
    rail.setAttribute('collapsed', '')
    document.body.classList.remove('rail-open')
    document.body.classList.add('rail-hidden')
  }
}

function persistRailOpen(open: boolean): void {
  try {
    localStorage.setItem(RAIL_OPEN_KEY, open ? '1' : '0')
  } catch { /* best-effort */ }
}

// ── Status bar ───────────────────────────────────────────────────────
//
// Polls /api/v1/workspaces every 15s and updates <sui-status-bar> props.
// Reflects the current theme on every tick (set by sui-theme-picker via
// applyTheme which writes data-theme on <html>).

const statusBar = document.getElementById('ship-status-bar') as HTMLElement | null
let daemonOnline = false

const shipMode = readShipMode()
window.__shipMode = shipMode
if (statusBar) {
  // Lowercase mode value so status-bar can compare with `attr === 'development'`.
  statusBar.setAttribute('mode', shipMode.toLowerCase())
}
document.dispatchEvent(new CustomEvent('ship:mode', { detail: { mode: shipMode } }))

async function updateStatus(): Promise<void> {
  if (!statusBar) return
  statusBar.setAttribute('theme', document.documentElement.dataset.theme ?? 'ship-light')

  // Fetch the active project's actual git branch via /api/v1/git/status.
  // The previous implementation pulled `workspaces[0].name` which returned
  // the FIRST registered workspace globally — not necessarily the
  // worktree the daemon is serving — so the status bar mis-labeled the
  // branch on linked worktrees.
  try {
    const envelope = await shipFetch<{ data?: { branch?: string }; branch?: string }>(gitStatus())
    const branch = envelope?.data?.branch ?? envelope?.branch ?? 'unknown'
    statusBar.setAttribute('branch', branch)
    statusBar.setAttribute('agents', '')
    if (!daemonOnline) {
      statusBar.setAttribute('online', '')
      daemonOnline = true
    }
  } catch {
    markOffline()
  }
}

function markOffline(): void {
  if (daemonOnline && statusBar) {
    statusBar.removeAttribute('online')
    statusBar.setAttribute('branch', 'offline')
    statusBar.setAttribute('agents', '')
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
// ⌘⇧E/Ctrl+Shift+E → body.edit-mode toggle (sui-edit-overlay listens)
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
  // shortcuts trigger regardless of caller. Keep ⌘⇧E gated on Shift via
  // its dedicated branch below.
  const key = e.key.toLowerCase()

  if (mod && key === 'j') {
    e.preventDefault()
    getDock()?.toggle()
    return
  }

  if (mod && key === 'k') {
    e.preventDefault()
    getCommandBar()?.toggle('find')
    return
  }

  if (mod && key === 'p') {
    e.preventDefault()
    getCommandBar()?.show('find')
    return
  }

  if (mod && e.shiftKey && key === 'e') {
    e.preventDefault()
    document.body.classList.toggle('edit-mode')
    return
  }

  if (mod && key === 'b') {
    e.preventDefault()
    document.dispatchEvent(new CustomEvent('sui:rail:toggle', { bubbles: true }))
    return
  }

  if (e.key === 'Escape') {
    // Native <dialog> handles Esc already. Backstop: close any open
    // sui-dialog or sui-drawer that may not be using the native dialog element.
    for (const el of document.querySelectorAll<HTMLElement & { close(): void }>(
      'sui-dialog[open], sui-drawer[open]',
    )) {
      try { el.close() } catch { /* ignore — native handles it */ }
    }
  }
})

// Wire close event from dock.
document.addEventListener('sui:dock:close', () => getDock()?.hide(), true)

// ── Chrome event wiring ─────────────────────────────────────────────
//
// Bridges component-dispatched events from <sui-nav-top> and <sui-rail>
// to the corresponding chrome elements that already live in the shell.
// Without these, clicking the menu / CLI / launcher / "+" buttons fired
// the event but no surface listened (pre-fix shell2 had no plumbing).

// Top-nav CLI / debug-panel button → toggle the panel-dock. The dock's
// `toggle()` method flips `body.dock-open`, which layout.css uses to
// reveal the dock surface.
document.addEventListener('sui:dock:toggle', () => getDock()?.toggle())

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
// calls openBar('create'). In shell2 the bar is `<sui-command-bar>`,
// which exposes the same imperative surface (`show(mode)`).
document.addEventListener('ship:create:open', () => {
  getCommandBar()?.show('create')
})

initProjectSwitch(kernelBridge, {
  statusBar,
  remount: () => { if (currentRoute) void mountRoute(currentRoute) },
  refreshStatus: () => updateStatus(),
})

document.addEventListener('click', onAnchorClick)
window.addEventListener('popstate', () => {
  const r = readRoute()
  if (r) void mountRoute(r)
  else {
    const slot = document.getElementById('ship-content')
    if (slot) showLanding(slot)
  }
})

// External navigation requests — components that need to navigate dispatch
// `sui:navigate` with `{ to: path }` (window.ship.navigate, sui-launcher,
// sui-command-bar palette items). They push history state themselves, but
// without this listener nothing remounts the view, so picking a result in
// the cmd-bar or the views picker updated the URL silently and the page
// kept showing the old view. mountRoute's own dispatches use either no
// detail or fire after currentRoute is updated, so the sameView guard
// prevents a remount loop.
document.addEventListener('sui:navigate', (e: Event) => {
  const detail = (e as CustomEvent).detail as { to?: string } | undefined
  if (!detail?.to) return
  const next = readRouteFromUrl(detail.to)
  if (!next) return
  const sameView = currentRoute
    && currentRoute.view === next.view
    && (currentRoute.sub ?? '') === (next.sub ?? '')
  if (sameView) return
  void mountRoute(next)
})

initChromeRegistry()
initShipApi()
initRail()
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

// First-run check: if setup hasn't been completed, rewrite the URL to
// `/setup` so the wizard takes over. `ensureSetupOrRedirect` returns
// true when it applied a redirect, in which case the boot route is
// recomputed before mount.
ensureSetupOrRedirect()
const initial = readRoute()
if (initial) {
  // Bare `/` resolves to the agents view; rewrite the URL so the rail
  // (which derives the active item from `location.pathname`) can
  // highlight Agents on first paint. Use replaceState so back-button
  // history doesn't pick up the synthetic redirect.
  if (location.pathname === '/' || location.pathname === '') {
    history.replaceState({}, '', '/' + initial.view)
  }
  void mountRoute(initial)
} else {
  const slot = document.getElementById('ship-content')
  if (slot) showLanding(slot)
}
