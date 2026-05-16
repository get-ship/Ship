// Browser-side HMR subscriber for the Bun bundler worker.
//
// `subscribeHmr()` opens an EventSource to `/_sui/dev/hmr` (shipd's SSE
// endpoint) and reacts to `module-changed` and `compile-error` events.
//
// Only call this in dev mode (import.meta.env.DEV or a query flag) — the
// production runtime never imports this file.
//
// On `module-changed`: dynamic-imports the new bundle URL with a cache-bust
// query (`?v=<bytes>`), then re-runs the affected mounts.
// On `compile-error`: renders a dev overlay with the error message.

const HMR_ENDPOINT = '/_sui/dev/hmr'
const OVERLAY_ID = '__sui_hmr_error_overlay'

type ModuleChangedEvent = {
  kind: 'module_changed'
  target: string
  out_path: string
  bytes: number
}

type CompileErrorEvent = {
  kind: 'compile_error'
  target: string
  message: string
  line?: number
  col?: number
  file?: string
}

type HmrEvent = ModuleChangedEvent | CompileErrorEvent

export interface HmrHandle {
  close(): void
}

/**
 * Subscribe to bundler HMR events from `/_sui/dev/hmr`.
 *
 * Returns a handle with `close()` to unsubscribe. The EventSource
 * reconnects automatically on network errors.
 *
 * @param opts.onChanged  Called when a bundle is rebuilt. Default: reload the page.
 * @param opts.onError    Called on compile errors. Default: show an overlay.
 */
export function subscribeHmr(opts?: {
  onChanged?: (event: ModuleChangedEvent) => void | Promise<void>
  onError?: (event: CompileErrorEvent) => void
}): HmrHandle {
  if (typeof EventSource === 'undefined') {
    return { close() {} }
  }

  const es = new EventSource(HMR_ENDPOINT)

  es.addEventListener('module-changed', (raw) => {
    let event: HmrEvent
    try {
      event = JSON.parse((raw as MessageEvent).data)
    } catch {
      return
    }
    const e = event as ModuleChangedEvent
    clearErrorOverlay()
    if (opts?.onChanged) {
      void opts.onChanged(e)
    } else {
      void defaultOnChanged(e)
    }
  })

  es.addEventListener('compile-error', (raw) => {
    let event: HmrEvent
    try {
      event = JSON.parse((raw as MessageEvent).data)
    } catch {
      return
    }
    const e = event as CompileErrorEvent
    if (opts?.onError) {
      opts.onError(e)
    } else {
      showErrorOverlay(e)
    }
  })

  es.addEventListener('error', () => {
    // Browser handles reconnect automatically; suppress noise.
  })

  return {
    close() {
      es.close()
      clearErrorOverlay()
    },
  }
}

// ── Default handlers ──────────────────────────────────────────────────────────

async function defaultOnChanged(event: ModuleChangedEvent): Promise<void> {
  // Re-import the updated bundle with a cache-bust, then remount.
  // The daemon serves bundles at `/_sui/views/<name>/view.js` for views
  // and at `/_sui/sui-components/<bundle>.js` for separate-bundle components.
  const urlBase = targetToUrl(event.target)
  if (!urlBase) {
    // Unknown target — fall back to full reload.
    window.location.reload()
    return
  }
  const url = `${urlBase}?v=${event.bytes}`
  try {
    await import(/* @vite-ignore */ url)
    // After dynamic import, the module's side effects (customElements.define,
    // mount calls) re-run. For views this means the reactive tree is replaced.
    console.info(`[sui:hmr] updated ${event.target}`)
  } catch {
    // If the dynamic import fails, fall back to a full reload.
    window.location.reload()
  }
}

function targetToUrl(target: string): string | null {
  if (target.startsWith('views/')) {
    // views/agents/view → /_sui/views/agents/view.js
    return `/_sui/${target}.js`
  }
  if (target.startsWith('sui-components/')) {
    // sui-components/sui-nav-top → /_sui/sui-components/sui-nav-top.js
    const name = target.slice('sui-components/'.length)
    return `/_sui/sui-components/${name}.js`
  }
  return null
}

// ── Error overlay ─────────────────────────────────────────────────────────────

function showErrorOverlay(event: CompileErrorEvent) {
  clearErrorOverlay()
  const div = document.createElement('div')
  div.id = OVERLAY_ID
  div.style.cssText = [
    'position:fixed;top:0;left:0;right:0;bottom:0;z-index:99999',
    'background:rgba(0,0,0,0.85);color:#ff6b6b;font-family:monospace',
    'padding:2rem;overflow:auto;white-space:pre-wrap;font-size:14px',
  ].join(';')

  const location = [event.file, event.line, event.col].filter(Boolean).join(':')
  div.textContent = [
    `[sui:hmr] compile error in ${event.target}`,
    location ? `  at ${location}` : '',
    '',
    event.message,
  ]
    .filter((l) => l !== undefined)
    .join('\n')

  div.addEventListener('click', clearErrorOverlay)
  document.body.appendChild(div)
}

function clearErrorOverlay() {
  document.getElementById(OVERLAY_ID)?.remove()
}
