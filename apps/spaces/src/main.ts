// Space scaffold entry — the thin frontend a Tauri `--app=<id>` window loads.
// Deliberately tiny: NO hub chrome, marketplace, settings, auth, or studio. It
// reads the launched space id, talks to the daemon over loopback (Connect), and
// mounts the space's surface. Everything it loads (chrome + views) is the only
// trust line in a space — so confinement (deck-host SES) layers on here, not in
// the hub.

import { createClient, DECK_PREFIX, type ShipSDK } from '@ship/sdk-client'
import type { SuiLoader, ViewPayload } from '@ship/sui-runtime'

interface SuiRuntime {
  mount: (
    target: HTMLElement,
    payload: unknown,
    opts?: { ship?: ShipSDK; loader?: SuiLoader; cssScope?: string },
  ) => Promise<{ dispose(): void }>
}

type SpaceWindow = Window & {
  __suiRuntime?: SuiRuntime
  __suiRuntimeReady?: Promise<SuiRuntime>
  DECK_APP_ID?: string
  __SHIP_DAEMON_URL__?: string
}

const w = window as SpaceWindow

// Daemon origin: planted by the Tauri host in the proxy-less webview; empty in a
// browser/vite context (same-origin → the dev proxy forwards to the daemon).
function daemonBase(): string {
  return w.__SHIP_DAEMON_URL__?.replace(/\/$/, '') ?? ''
}

// The space this window was launched for. The hub opens a window per space via
// `open_space(app_id)` → `--app=<id>` → planted on `DECK_APP_ID` by the
// daemon scaffold injection. The `?app=` query param is the browser/dev-
// addressable fallback (a space is reachable by URL, not only by native launch).
const appId =
  w.DECK_APP_ID ?? new URLSearchParams(location.search).get('app') ?? ''

const ship: ShipSDK = createClient({ baseUrl: daemonBase() })

const loader: SuiLoader = {
  async fetchView(nameOrPath: string): Promise<ViewPayload> {
    const rel = nameOrPath.startsWith(`${DECK_PREFIX}/views/`) ? nameOrPath : `${DECK_PREFIX}/views/${nameOrPath}`
    const res = await fetch(`${daemonBase()}${rel}`, { credentials: 'same-origin' })
    if (!res.ok) throw new Error(`[space] fetch ${rel} → ${res.status}`)
    return (await res.json()) as ViewPayload
  },
  loadComponent: (tag: string): Promise<void> =>
    import(/* @vite-ignore */ `${daemonBase()}${DECK_PREFIX}/sui-components/${tag}.js`).then(() => undefined),
}

function placeholder(slot: HTMLElement, msg: string): void {
  slot.innerHTML = `<div class="space-empty"><h1>Space</h1><p>${msg}</p></div>`
}

async function boot(): Promise<void> {
  const runtime = await (w.__suiRuntimeReady ?? Promise.resolve(w.__suiRuntime))
  const slot = document.getElementById('space-content')
  if (!slot) return
  if (!runtime?.mount) {
    placeholder(slot, 'SUI runtime did not load.')
    return
  }
  if (!appId) {
    // No --app id (e.g. opened directly): nothing to route to yet.
    placeholder(slot, 'No space id. Launch a space from the Ship hub.')
    return
  }
  // First cut: the space id IS the view name (a daemon-served view). A later
  // pass reads the space's config (chosen chrome + views) and confines them.
  try {
    const payload = await loader.fetchView(appId)
    await runtime.mount(slot, payload, { ship, loader, cssScope: 'adjacent' })
    document.title = `Ship Space — ${appId}`
  } catch (err) {
    placeholder(slot, `No surface for "${appId}" yet (${(err as Error).message}).`)
  }
}

void boot()
