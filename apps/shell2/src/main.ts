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
//
// No daemon dependency beyond the F1 contract: /_sui/views/* JSON +
// /_sui/{runtime,dom,sdk-client,components}.js. Daemon-coupled SUI
// concerns (mutations, SSE, HMR) flow over the same /_sui/* paths but
// are owned by the framework crate, not the shell.

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
}

function updateNav(r: Route | null): void {
  const nav = document.getElementById('ship-top-nav')
  if (!nav) return
  nav.textContent = r ? `· ${r.view}${r.sub ? ' / ' + r.sub : ''}` : ''
}

// ── Click delegate ──────────────────────────────────────────────────
//
// Intercepts internal `<a href="/...">` clicks so view-internal nav
// updates the URL + swaps the embedded view without a full reload.
// Bypass conditions: modifier keys (open in new tab), middle/right
// click, target=_blank, off-origin, hash-only, data-no-router.

function onAnchorClick(e: MouseEvent): void {
  if (e.defaultPrevented) return
  if (e.button !== 0) return
  if (e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return
  const a = (e.target as Element | null)?.closest?.('a[href]') as HTMLAnchorElement | null
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

// Theme is owned by <sui-theme-picker>. The picker reads the persisted
// theme from localStorage and calls applyTheme() on mount, which sets
// data-theme + the canonical .dark class on <html>. No shell-side
// scaffolding needed beyond mounting the picker in the top bar.

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

document.getElementById('ship-notify-btn')?.addEventListener('click', dispatchDemoToast)
document.addEventListener('click', onAnchorClick)
window.addEventListener('popstate', () => {
  const r = readRoute()
  if (r) void mountRoute(r)
  else {
    const slot = document.getElementById('ship-content')
    if (slot) showLanding(slot)
  }
})

const initial = readRoute()
if (initial) void mountRoute(initial)
else {
  const slot = document.getElementById('ship-content')
  if (slot) showLanding(slot)
}
