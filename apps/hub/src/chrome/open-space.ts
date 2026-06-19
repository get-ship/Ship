// Single seam for "open a space as its OWN process+window" — the only place
// that knows whether the page is running in a Tauri webview or a plain
// browser tab.
//
// Tauri (desktop shell): `invoke('open_space', { appId })` → Rust in
// `apps/host/src/launcher.rs::open_space` spawns the same binary
// with `--app=<id>` (its own PID, its own webview, branded for the space).
// Browser-dev (vite at :PORT, no Tauri global): `window.open('/space/<id>')`
// in a new tab → the daemon serves the same space scaffold at that path.
//
// Both branches end on the same `/space/<id>` URL the hub is wired to:
// one open gesture, one destination, two transports. Hub "inline" switching
// (ship.space.activate) is unchanged — the two coexist (hub swaps inline,
// "open as app" pops a window).
//
// Hub port of apps/shell/src/chrome/open-space.ts — identical body; the
// hub and shell live in different bundles, so each owns its own copy of
// this transport seam. Keeping one canonical source per host means the
// shell cutover doesn't have to negotiate a shared module path.

interface TauriInvoke {
  (cmd: string, args?: Record<string, unknown>): Promise<unknown>
}

interface TauriGlobal {
  core?: { invoke?: TauriInvoke }
  invoke?: TauriInvoke
}

type TauriWindow = Window & {
  __TAURI__?: TauriGlobal
  __TAURI_INTERNALS__?: { invoke?: TauriInvoke }
}

function tauriInvoke(w: TauriWindow): TauriInvoke | null {
  return (
    w.__TAURI__?.core?.invoke ??
    w.__TAURI__?.invoke ??
    w.__TAURI_INTERNALS__?.invoke ??
    null
  )
}

/** Returns the transport actually used; useful in tests + telemetry. */
export type OpenSpaceTransport = 'tauri' | 'browser'

/**
 * Open the space identified by `id` in its own native window (Tauri) or a
 * new browser tab (dev). No-op if `id` is empty — guards against the
 * `?app=` query or `DECK_APP_ID` arriving unset.
 *
 * Throws only on Tauri invoke failures (the browser path can't fail
 * meaningfully — `window.open` returns null on popup-block but a popup
 * block is not an error worth surfacing as a rejection).
 */
export async function openSpace(id: string): Promise<OpenSpaceTransport> {
  if (!id) return 'browser'
  const w = window as TauriWindow
  const invoke = tauriInvoke(w)
  if (invoke) {
    await invoke('open_space', { appId: id })
    return 'tauri'
  }
  window.open(`/space/${encodeURIComponent(id)}`, '_blank', 'noopener,noreferrer')
  return 'browser'
}
