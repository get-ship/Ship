// Shell HMR glue. Two reload paths, gated by daemon mode:
//
//   Dev (ship-mode=development): the Deck live-reload client (served at
//     `/_deck/live-reload.js`) subscribes to the bundler firehose
//     (`/_deck/dev/hmr`) and reloads on any target change. Always on, no
//     toggle — this is the Ship-team dev loop.
//
//   Prod: live-reload is coupled to *edit mode* (the product authoring
//     toggle). The edit-mode chrome opens the per-view SSE and re-dispatches
//     `sui:view:compiled` as a document event; here we react to it and
//     re-mount the view being authored. No reload outside edit mode.
//
// HMR is a Deck host concern, not a SUI-runtime one; the shell and the
// third-party view host share the same Deck client for the dev path.
//
// Bundler target shape (see crates/deck/deck-bundler/src/targets.rs):
//   views/<name>/view      → one bundle per views/<name>/page.sui
//   sui-components/<name>  → separate-bundle component
//   sui-runtime, sui-dom, sui-components, …
//
// On a change to the *current* view → cache-bust import the rebuilt
// view.js and re-mount in place. Any other target → full reload (custom
// elements can't be re-registered without one).

import { DECK_PREFIX } from '@ship/sdk-client'

export interface MountModule {
  default: { setup: (...args: unknown[]) => unknown }
}


export interface HmrDeps {
  getCurrentView(): string | null
  remount(module?: MountModule): Promise<void>
}

interface LiveReloadModule {
  attachLiveReload(opts: {
    viewName?: string
    onViewChanged?: (e: { target: string; bytes?: number }) => void | Promise<void>
    onOtherChanged?: (e: { target: string }) => void | Promise<void>
    onError?: (e: { message: string; file?: string; line?: number; col?: number }) => void
  }): { close(): void }
}

function isDevMode(): boolean {
  if (typeof document === 'undefined') return false
  return document.querySelector('meta[name="ship-mode"]')?.getAttribute('content') === 'development'
}

/** HMR failure stages, surfaced distinctly so the cause is never hidden. */
export type HmrPhase = 'fetch' | 'import' | 'hydrate'

// Injectable IO for `remountView`: fetch the rebuilt bundle, then evaluate it
// as a module. Split out so the orchestration (per-phase error handling,
// keep-last-good) is unit-testable without a network or a real blob-URL dynamic
// import (which jsdom can't run). Production wires `defaultIo` below.
export interface RemountIo {
  /** Fetch the rebuilt bundle source. Throws on network / non-2xx. */
  fetchSource(url: string): Promise<string>
  /** Evaluate bundle source to its module. Throws on a compile/syntax error. */
  evalModule(src: string): Promise<MountModule>
}

// Why fetch+Blob URL instead of `await import(url)`:
//   dynamic `import(url)` registers the module in the host realm's ESM
//   registry under the cache-busted URL. The bundler rewrites that
//   importee's own bare specifier imports to relative paths, so it does
//   NOT pull fresh @ship/sui-runtime with it — that part is OK. But the
//   per-compile module graph still accumulates in the registry: every
//   `rebuild` from the watcher creates a new entry that nothing ever
//   evicts, so a runaway watcher (issue #129/#130) is GB-fast. fetch +
//   Blob URL evaluates the same source as a one-off script and registers
//   nothing — the URL vanishes with its blob the moment
//   `URL.revokeObjectURL` is called. This is dev-only; the production
//   `sui:view:compiled` path hits the same code and gets the same fix.
const defaultIo: RemountIo = {
  async fetchSource(url) {
    const res = await fetch(url, { credentials: 'same-origin' })
    if (!res.ok) throw new Error(`view bundle HTTP ${res.status}`)
    return res.text()
  },
  async evalModule(src) {
    const blob = new Blob([src], { type: 'text/javascript' })
    const blobUrl = URL.createObjectURL(blob)
    try {
      return (await import(/* @vite-ignore */ blobUrl)) as MountModule
    } finally {
      URL.revokeObjectURL(blobUrl)
    }
  },
}

// Re-evaluate the rebuilt view bundle in place. `bust` is the bundler byte count
// (dev) or the compile timestamp (prod) — either changes per build, so the
// cache-busting query forces a fresh fetch.
//
// P0-3: a mid-edit failure must NOT blanket-`window.location.reload()` — that
// discards all signal state and hides the cause. Each stage (fetch → import →
// hydrate) is caught separately and the real error is surfaced (console + dev
// overlay) while the LAST-GOOD mount is kept: a compile error fails at fetch
// (the daemon serves the diagnostic as non-2xx) or import (a syntax error in the
// new bundle) BEFORE `remount` runs, so the live view is untouched and fixing
// the typo re-fires this path and recovers cleanly — no reload, no lost state.
export async function remountView(
  deps: HmrDeps,
  view: string,
  bust: string,
  io: RemountIo = defaultIo,
): Promise<void> {
  const url = `${DECK_PREFIX}/views/${view}/view.js?v=${encodeURIComponent(bust)}`
  let src: string
  try {
    src = await io.fetchSource(url)
  } catch (err) {
    reportHmrError(view, 'fetch', err)
    return
  }
  let m: MountModule
  try {
    m = await io.evalModule(src)
  } catch (err) {
    reportHmrError(view, 'import', err)
    return
  }
  try {
    await deps.remount(m)
    clearHmrError()
    console.info(`[hmr] re-mounted ${view}`)
  } catch (err) {
    // The new module imported fine but remount threw mid-hydrate. We can't
    // guarantee the prior mount survived, but a blanket reload would still lose
    // state and hide the stack — surface it and let the next good build recover.
    reportHmrError(view, 'hydrate', err)
  }
}

const HMR_OVERLAY_ID = 'sui-hmr-error'

// Surface an HMR failure: log the real error (classified by phase, never
// swallowed) and paint a dismissable dev overlay. The last-good mount is left
// untouched, so the page keeps working while the author fixes the source.
function reportHmrError(view: string, phase: HmrPhase, err: unknown): void {
  const detail = err instanceof Error ? (err.stack ?? err.message) : String(err)
  console.error(`[hmr] ${phase} error in ${view} — keeping last-good view:`, err)
  if (typeof document === 'undefined') return
  let el = document.getElementById(HMR_OVERLAY_ID)
  if (!el) {
    el = document.createElement('div')
    el.id = HMR_OVERLAY_ID
    el.setAttribute('role', 'alert')
    el.style.cssText =
      'position:fixed;left:0;right:0;bottom:0;z-index:2147483647;max-height:40vh;' +
      'overflow:auto;margin:0;padding:12px 16px;white-space:pre-wrap;' +
      'font:12px/1.5 ui-monospace,SFMono-Regular,Menlo,monospace;' +
      'color:#fff;background:#7f1d1d;border-top:2px solid #ef4444'
    el.addEventListener('click', clearHmrError)
    document.body.appendChild(el)
  }
  el.dataset.phase = phase
  el.textContent = `HMR ${phase} error in ${view} (last-good view kept — click to dismiss)\n\n${detail}`
}

// Remove the HMR error overlay (a build succeeded, or the user dismissed it).
function clearHmrError(): void {
  if (typeof document === 'undefined') return
  document.getElementById(HMR_OVERLAY_ID)?.remove()
}

export function initHmr(deps: HmrDeps): { close(): void } {
  let inner: { close(): void } | null = null
  let closed = false

  if (typeof window === 'undefined') return { close() {} }

  // Prod: live-reload is coupled to edit mode. The edit-mode chrome opens
  // the per-view SSE and re-dispatches `sui:view:compiled` on document; we
  // re-mount the view being authored. No firehose, no reload outside edit
  // mode. (Dev takes the firehose path below instead.)
  if (!isDevMode()) {
    const onCompiled = (e: Event) => {
      const d = (e as CustomEvent).detail as { view_name?: string; timestamp?: string } | null
      const view = d?.view_name
      if (!view || deps.getCurrentView() !== view) return
      void remountView(deps, view, d?.timestamp ?? String(Date.now()))
    }
    document.addEventListener('sui:view:compiled', onCompiled)
    const close = () => document.removeEventListener('sui:view:compiled', onCompiled)
    window.addEventListener('beforeunload', close)
    return { close }
  }

  // Dev: the bundler firehose drives reload for any target.
  void (async () => {
    // Runtime URL served by the daemon (Deck). Held in a variable so TS
    // treats it as a dynamic specifier rather than resolving it at build.
    const clientUrl = `${DECK_PREFIX}/live-reload.js`
    let mod: LiveReloadModule
    try {
      mod = (await import(/* @vite-ignore */ clientUrl)) as LiveReloadModule
    } catch (err) {
      console.warn('[hmr] live-reload client unavailable:', err)
      return
    }
    if (closed) return
    // No pinned viewName — the shell navigates, so filter per-change against
    // the live current route inside onViewChanged.
    inner = mod.attachLiveReload({
      onViewChanged: (e) => {
        const changed = e.target.split('/')[1]
        if (!changed || deps.getCurrentView() !== changed) return
        void remountView(deps, changed, String(e.bytes ?? Date.now()))
      },
      onOtherChanged: (e) => {
        console.info(`[hmr] ${e.target} changed; reload`)
        window.location.reload()
      },
      onError: (err) => {
        const loc = err.file ? ` (${err.file}${err.line ? `:${err.line}` : ''})` : ''
        console.error(`[hmr] compile error${loc}:`, err.message)
      },
    })
    window.addEventListener('beforeunload', () => inner?.close())
  })()

  return {
    close() {
      closed = true
      inner?.close()
    },
  }
}
