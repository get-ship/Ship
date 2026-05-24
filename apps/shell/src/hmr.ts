// Shell HMR glue — delegates to the one Deck live-reload client served at
// `/_sui/live-reload.js`. HMR is a Deck host concern, not a SUI-runtime one;
// the shell and the third-party view host share this single client.
//
// The client self-gates on `<meta name="ship-mode" content="development">`
// (shell's index.html ships the tag; shipd rewrites it per daemon mode), so
// there's no separate dev check here. It subscribes to the bundler firehose
// (`/_sui/dev/hmr`) and calls back on change.
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

export function initHmr(deps: HmrDeps): { close(): void } {
  let inner: { close(): void } | null = null
  let closed = false

  void (async () => {
    if (typeof window === 'undefined') return
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
      onViewChanged: async (e) => {
        const changed = e.target.split('/')[1]
        if (!changed || deps.getCurrentView() !== changed) return
        const url = `/_sui/${e.target}.js?v=${e.bytes ?? Date.now()}`
        try {
          const m = (await import(/* @vite-ignore */ url)) as MountModule
          await deps.remount(m)
          console.info(`[hmr] re-mounted ${changed}`)
        } catch (err) {
          console.error(`[hmr] re-mount of ${changed} failed; reloading`, err)
          window.location.reload()
        }
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
