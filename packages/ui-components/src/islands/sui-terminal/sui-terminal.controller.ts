/**
 * Paired controller for <sui-terminal>.
 *
 * Pattern established here for all future imperative-engine SUI ports
 * (markdown-editor, code-editor, etc.):
 *
 *   1. bind:this={ref} in the template → Signal<El|null> in Scope.
 *   2. controller(def).mount runs AFTER template renders; access the ref via
 *      `(state as unknown as Scope).ref()` — the $scope proxy keeps signals
 *      callable even though MutableState<Scope> unwraps them to value types.
 *   3. Wire engine events → host.dispatchEvent (Option A bridge).
 *   4. Return the cleanup function from mount; the compiler wires it into
 *      __cleanups so disconnect tears everything down.
 *   5. Use effect() inside mount for reactive prop changes (theme, fontSize).
 *
 * Search overlay is DEFERRED: it needs controller.on handlers to mutate the
 * searchOpen/searchQuery state signals, which requires the on-handler writable
 * proxy to see $scope's signals at call time. The wiring is in place (ctx.scope
 * is passed before setup runs) but search requires the SearchAddon to survive
 * across multiple handler invocations — a WeakMap on host could bridge that.
 * Filed as a follow-up for the next imperative-engine port.
 */

import { controller, effect } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'

// Import Scope from the compiler-emitted .d.ts (produced alongside the .sui).
// This module resolves after `node scripts/build-bundles.mjs` compiles the
// template; typecheck fails before first build — that is expected.
import type { Scope } from './sui-terminal.sui'

// xterm.js is bundled eagerly into sui-components.js by esbuild. Dynamic
// imports are used in the Lit original to keep ship-terminal.js lazy; here
// the component is part of the components bundle so eager import is fine.
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { WebglAddon } from '@xterm/addon-webgl'

const MONO_STACK =
  "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace"

// Minimal xterm-internal CSS needed for canvas layer positioning inside a
// shadow root. xterm.js injects its own sheet into `document.head` which
// is unreachable from a closed shadow tree; we adopt an equivalent sheet.
const XTERM_SHADOW_CSS = `
.xterm { position: relative; user-select: none; }
.xterm.focus, .xterm:focus { outline: none; }
.xterm .xterm-helpers { position: absolute; top: 0; z-index: 5; }
.xterm .xterm-helper-textarea {
  padding: 0; border: 0; margin: 0;
  position: absolute; opacity: 0; left: -9999em; top: 0;
  width: 0; height: 0; z-index: -5;
  white-space: nowrap; overflow: hidden; resize: none;
}
.xterm .composition-view { display: none; position: absolute; white-space: nowrap; z-index: 1; }
.xterm .xterm-viewport {
  background-color: transparent !important;
  overflow-y: scroll; cursor: default;
  position: absolute; right: 0; left: 0; top: 0; bottom: 0;
}
.xterm .xterm-screen { position: relative; }
.xterm .xterm-screen canvas { position: absolute; left: 0; top: 0; }
.xterm .xterm-decoration-container { position: absolute; top: 0; left: 0; right: 0; }
`

/** Fallback palette when CSS custom properties are unavailable (JSDOM). */
const THEME_FALLBACKS: Record<string, string> = {
  background: '#fafaf9',
  foreground: '#1c1917',
  cursor: '#c2410c',
  cursorAccent: '#fafaf9',
  black: '#78716c',
  red: '#dc2626',
  green: '#16a34a',
  yellow: '#ca8a04',
  blue: '#2563eb',
  magenta: '#9333ea',
  white: '#1c1917',
  brightBlack: '#a8a29e',
  brightRed: '#ef4444',
  brightGreen: '#22c55e',
  brightYellow: '#eab308',
  brightBlue: '#3b82f6',
  brightMagenta: '#a855f7',
  brightCyan: '#06b6d4',
  brightWhite: '#fafaf9',
}

const TOKEN_MAP: Record<string, string> = {
  background: '--background',
  foreground: '--foreground',
  cursor: '--primary',
  cursorAccent: '--background',
  black: '--muted-foreground',
  red: '--destructive',
  green: '--success',
  yellow: '--warning',
  blue: '--info',
  magenta: '--accent',
  white: '--foreground',
}

function buildTheme(host: HTMLElement): Record<string, string> {
  const theme: Record<string, string> = { ...THEME_FALLBACKS }
  const cs = getComputedStyle(host)
  for (const [key, cssVar] of Object.entries(TOKEN_MAP)) {
    const val = cs.getPropertyValue(cssVar).trim()
    if (val) theme[key] = val
  }
  const primary = theme.cursor || THEME_FALLBACKS.cursor!
  theme.selectionBackground = `color-mix(in oklch, ${primary} 30%, transparent)`
  theme.cyan = 'oklch(0.7 0.15 200)'
  theme.brightBlack = theme.white!
  theme.brightRed = theme.red!
  theme.brightGreen = theme.green!
  theme.brightYellow = theme.yellow!
  theme.brightBlue = theme.blue!
  theme.brightMagenta = theme.magenta!
  theme.brightCyan = theme.cyan
  theme.brightWhite = theme.background!
  return theme
}

function injectXtermCSS(shadow: ShadowRoot): void {
  try {
    const sheet = new CSSStyleSheet()
    sheet.replaceSync(XTERM_SHADOW_CSS)
    shadow.adoptedStyleSheets = [...shadow.adoptedStyleSheets, sheet]
  } catch {
    // CSSStyleSheet constructor or adoptedStyleSheets may not be available
  }
}

// Cast helper: bind:this refs are Signal<T> in Scope but MutableState<Scope>
// unwraps them to T. The $scope proxy in the lifecycle wiring keeps them
// callable, so we cast back to Scope to get the right call signature.
type ScopeView = Scope

export default controller<Scope>({
  mount(state, host) {
    const s = state as unknown as ScopeView

    const hostEl = (s.termHost as Signal<HTMLDivElement | null>)()
    if (!hostEl) {
      console.warn('[sui-terminal] bind:this ref is null — xterm.js not initialized')
      return
    }

    const shadow = host.shadowRoot
    if (shadow) injectXtermCSS(shadow)

    const readonly = (s.readonly as Signal<boolean>)()
    const rows = (s.rows as Signal<number>)()
    const cols = (s.cols as Signal<number>)()
    const rawSize = (s.fontSize as Signal<number>)()
    const fontSize = rawSize > 0 ? rawSize : resolveTokenFontSize(host)

    const term = new Terminal({
      rows,
      cols,
      fontSize,
      fontFamily: MONO_STACK,
      theme: buildTheme(host),
      cursorBlink: !readonly,
      disableStdin: readonly,
      allowProposedApi: true,
      convertEol: true,
    })

    const fit = new FitAddon()
    term.loadAddon(fit)
    term.loadAddon(new WebLinksAddon())

    // Prefer WebGL renderer to avoid canvas first-row corruption in shadow DOM.
    // Guard with an explicit WebGL2 probe first — the addon activation error
    // propagates asynchronously through xterm's event bus (can't be caught with
    // try/catch after the fact) so we must skip the addon entirely in headless /
    // no-WebGL environments.
    const __testCanvas = document.createElement('canvas')
    if (__testCanvas.getContext('webgl2')) {
      try {
        const webgl = new WebglAddon()
        webgl.onContextLoss(() => { webgl.dispose() })
        term.loadAddon(webgl)
      } catch {
        // WebGL init failed — canvas fallback
      }
    }

    // Delay open() until the container has layout to avoid garbled glyphs.
    requestAnimationFrame(() => {
      if (!host.isConnected) return
      term.open(hostEl)
      if ((s.fit as Signal<boolean>)()) fit.fit()
    })

    // Bridge engine events → host CustomEvents (Option A pattern).
    const dataDisposer = term.onData((data) =>
      host.dispatchEvent(
        new CustomEvent('sui-terminal-data', { detail: { data }, bubbles: true, composed: true }),
      ),
    )
    const keyDisposer = term.onKey(({ key, domEvent }) =>
      host.dispatchEvent(
        new CustomEvent('sui-terminal-key', { detail: { key, domEvent }, bubbles: true, composed: true }),
      ),
    )

    // Expose imperative API on the host element.
    const el = host as HTMLElement & {
      write?: (s: string) => void
      writeln?: (s: string) => void
      clear?: () => void
      focus?: () => void
      terminal?: Terminal | null
    }
    el.write = (str: string) => term.write(str)
    el.writeln = (str: string) => term.writeln(str)
    el.clear = () => term.clear()
    el.focus = () => term.focus()
    el.terminal = term

    // Reactive prop updates via effect() — re-runs when signals change.
    const disposers: Array<() => void> = []

    if (!readonly) {
      const ro = new ResizeObserver(() => fit.fit())
      const fitSig = s.fit as Signal<boolean>
      disposers.push(effect(() => {
        if (fitSig()) {
          ro.observe(host)
        } else {
          ro.disconnect()
        }
      }))
      disposers.push(() => ro.disconnect())
    }

    disposers.push(effect(() => {
      term.options.theme = buildTheme(host)
      // Reading theme signal subscribes to changes.
      void (s.theme as Signal<string>)()
    }))

    disposers.push(effect(() => {
      const size = (s.fontSize as Signal<number>)()
      if (size > 0) {
        term.options.fontSize = size
        fit.fit()
      }
    }))

    host.dispatchEvent(
      new CustomEvent('sui-terminal-ready', { bubbles: true, composed: true }),
    )

    return () => {
      for (const d of disposers) d()
      dataDisposer.dispose()
      keyDisposer.dispose()
      el.write = undefined
      el.writeln = undefined
      el.clear = undefined
      el.focus = undefined
      el.terminal = null
      try { term.dispose() } catch { /* teardown race */ }
    }
  },
})

function resolveTokenFontSize(host: HTMLElement): number {
  const raw = getComputedStyle(host).getPropertyValue('--text-sm').trim()
  const parsed = parseFloat(raw)
  return parsed > 0 ? parsed : 12
}
