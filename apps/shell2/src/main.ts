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

interface MountFn {
  (
    target: HTMLElement,
    payload: unknown,
    opts?: { cssScope?: 'adjacent' | 'document-head' | 'none' },
  ): Promise<{ dispose(): void; element: HTMLElement }>
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

// ── Routing ─────────────────────────────────────────────────────────

interface Route {
  /** First path segment, e.g. "ship-ui-studio". */
  view: string
  /** Remaining path segments joined by "/", e.g. "sui-button". */
  sub?: string
}

function readRoute(): Route | null {
  const segs = location.pathname.split('/').filter(Boolean)
  if (segs.length === 0) return null
  return { view: segs[0]!, ...(segs.length > 1 ? { sub: segs.slice(1).join('/') } : {}) }
}

function routeToPath(r: Route): string {
  return r.sub ? `/${r.view}/${r.sub}` : `/${r.view}`
}

function payloadUrl(r: Route): string {
  return r.sub ? `/_sui/views/${r.view}/${r.sub}` : `/_sui/views/${r.view}`
}

// ── Mount lifecycle ─────────────────────────────────────────────────

let currentHandle: { dispose(): void } | null = null
let currentRoute: Route | null = null

async function mountRoute(r: Route): Promise<void> {
  const slot = document.getElementById('ship-content')
  if (!slot) throw new Error('shell: #ship-content missing from index.html')

  // Tear down the previous mount before swapping. mount() owns DOM
  // injection — clearing slot.innerHTML before the next call would
  // race with the new mount's own innerHTML write.
  currentHandle?.dispose()
  currentHandle = null

  // Notify devtools tabs of navigation.
  document.dispatchEvent(new Event('sui:navigate'))

  if (!r) return showLanding(slot)

  const w = window as WindowWithSui
  const runtime = await (w.__suiRuntimeReady ?? Promise.resolve(w.__suiRuntime))
  if (!runtime?.mount) throw new Error('shell: SUI runtime did not load')

  try {
    const res = await fetch(payloadUrl(r), { credentials: 'same-origin' })
    if (!res.ok) {
      slot.innerHTML = `<div class="shell-error">${res.status} ${r.view}${r.sub ? '/' + r.sub : ''}\n${await res.text()}</div>`
      return
    }
    const payload = await res.json()
    const handle = await runtime.mount(slot, payload, { cssScope: 'adjacent' })
    currentHandle = handle
    currentRoute = r
  } catch (err) {
    slot.innerHTML = `<div class="shell-error">mount failed: ${(err as Error).message}</div>`
  }
  updateNav(r)
  updateRailActive(r)
}

function showLanding(slot: HTMLElement): void {
  slot.innerHTML = `
    <div class="shell-empty">
      <h2>Ship</h2>
      <p>Open a view: <a href="/ship-ui-studio">/ship-ui-studio</a> · <a href="/counter">/counter</a> · <a href="/sui-demo">/sui-demo</a></p>
    </div>
  `
  currentRoute = null
  updateNav(null)
  updateRailActive(null)
}

function updateNav(r: Route | null): void {
  const path = r ? routeToPath(r) : '/'
  document.dispatchEvent(new CustomEvent('sui:navigate', { detail: { to: path } }))
}

// Mark the matching rail item active; clear all others.
function updateRailActive(r: Route | null): void {
  const items = document.querySelectorAll<HTMLElement>('sui-rail-item[data-view]')
  const viewName = r?.view ?? ''
  for (const item of items) {
    if (item.dataset.view === viewName) item.setAttribute('active', '')
    else item.removeAttribute('active')
  }
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
  const path = routeToPath(next)
  if (currentRoute && path === routeToPath(currentRoute)) return
  history.pushState({}, '', path)
  void mountRoute(next)
}

function readRouteFromUrl(pathname: string): Route | null {
  const segs = pathname.split('/').filter(Boolean)
  if (segs.length === 0) return null
  return { view: segs[0]!, ...(segs.length > 1 ? { sub: segs.slice(1).join('/') } : {}) }
}

// ── Rail collapse persistence ────────────────────────────────────────

const RAIL_COLLAPSED_KEY = 'ship-rail-collapsed'

function initRail(): void {
  const rail = document.getElementById('ship-rail')
  if (!rail) return

  // Restore persisted collapsed state before first paint.
  try {
    if (localStorage.getItem(RAIL_COLLAPSED_KEY)) rail.setAttribute('collapsed', '')
  } catch { /* best-effort */ }

  // sui-rail:toggle — fired by the rail's own collapse button
  rail.addEventListener('sui-rail:toggle', (e: Event) => {
    const collapsed = (e as CustomEvent<{ collapsed: boolean }>).detail?.collapsed
    try {
      if (collapsed) localStorage.setItem(RAIL_COLLAPSED_KEY, '1')
      else localStorage.removeItem(RAIL_COLLAPSED_KEY)
    } catch { /* best-effort */ }
  })

  // sui:rail:toggle — fired by sui-nav-top's menu button (toggle request)
  document.addEventListener('sui:rail:toggle', () => {
    const next = !rail.hasAttribute('collapsed')
    if (next) rail.setAttribute('collapsed', '')
    else rail.removeAttribute('collapsed')
    try {
      if (next) localStorage.setItem(RAIL_COLLAPSED_KEY, '1')
      else localStorage.removeItem(RAIL_COLLAPSED_KEY)
    } catch { /* best-effort */ }
  })
}

// ── Status bar ───────────────────────────────────────────────────────
//
// Polls /api/v1/workspaces every 15s and updates <sui-status-bar> props.
// Reflects the current theme on every tick (set by sui-theme-picker via
// applyTheme which writes data-theme on <html>).

const statusBar = document.getElementById('ship-status-bar') as HTMLElement | null
let daemonOnline = false

async function updateStatus(): Promise<void> {
  if (!statusBar) return
  statusBar.setAttribute('theme', document.documentElement.dataset.theme ?? 'ship-light')

  try {
    const res = await fetch('/api/v1/workspaces', { credentials: 'same-origin' })
    if (res.ok) {
      const envelope = await res.json()
      const ws = envelope?.data?.workspaces?.[0]
      statusBar.setAttribute('branch', ws?.name ?? ws?.id ?? 'no workspace')
      statusBar.setAttribute('agents', ws?.active_agent ? `agent: ${ws.active_agent}` : '')
      if (!daemonOnline) {
        statusBar.setAttribute('online', '')
        daemonOnline = true
      }
    } else {
      markOffline()
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
  container.show({
    tone: detail.tone,
    title: detail.title ?? detail.message,
    description: detail.description,
    duration: detail.duration,
    dismissible: detail.dismissible,
    showProgress: detail.showProgress,
    actionHtml: detail.actionHtml,
  })
}

document.addEventListener('sui:notify', onSuiNotify)

function dispatchDemoToast(): void {
  document.dispatchEvent(new CustomEvent('sui:notify', {
    detail: {
      tone: 'success',
      title: 'Ship notification',
      description: 'Toast bridge is working end to end.',
      duration: 5000,
    },
    bubbles: true,
  }))
}

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

document.addEventListener('keydown', (e: KeyboardEvent) => {
  if ((e.metaKey || e.ctrlKey) && e.key === 'j') {
    e.preventDefault()
    getDock()?.toggle()
  }
})

// Wire close event from dock.
document.addEventListener('sui:dock:close', () => getDock()?.hide(), true)
document.addEventListener('click', onAnchorClick)
window.addEventListener('popstate', () => {
  const r = readRoute()
  if (r) void mountRoute(r)
  else {
    const slot = document.getElementById('ship-content')
    if (slot) showLanding(slot)
  }
})

initRail()
setInterval(() => void updateStatus(), 15000)
void updateStatus()

const initial = readRoute()
if (initial) void mountRoute(initial)
else {
  const slot = document.getElementById('ship-content')
  if (slot) showLanding(slot)
}
