// Shell HMR glue. Two reload paths, gated by daemon mode:
//
//   Dev (ship-mode=development): the Deck live-reload client (served at
//     `/_sui/live-reload.js`) subscribes to the bundler firehose
//     (`/_sui/dev/hmr`) and reloads on any target change. Always on, no
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
// Bundler target shape (see packages/deck/bundler/index.ts):
//   views/<name>/view      → one bundle per views/<name>/page.sui
//   sui-components/<name>  → separate-bundle component
//   sui-runtime, sui-dom, sui-components, …
//
// On a change to the *current* view → cache-bust import the rebuilt
// view.js and re-mount in place. Any other target → full reload (custom
// elements can't be re-registered without one).

interface MountModule {
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

// Cache-bust import the rebuilt view bundle and re-mount in place. `bust`
// is the bundler byte count (dev) or the compile timestamp (prod) — either
// changes per build, defeating the ESM import cache.
async function remountView(deps: HmrDeps, view: string, bust: string): Promise<void> {
  const url = `/_sui/views/${view}/view.js?v=${encodeURIComponent(bust)}`
  try {
    const m = (await import(/* @vite-ignore */ url)) as MountModule
    await deps.remount(m)
    console.info(`[hmr] re-mounted ${view}`)
  } catch (err) {
    console.error(`[hmr] re-mount of ${view} failed; reloading`, err)
    window.location.reload()
  }
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
    const clientUrl = '/_sui/live-reload.js'
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
