/**
 * Lazy-load helpers for <ship-terminal>.
 *
 * Handles dynamic import of xterm.js and its addons, plus injection of
 * the minimal xterm CSS into a shadow root. Kept separate from the
 * component class to stay under the 300-line cap.
 */

import type { Terminal } from '@xterm/xterm'
import type { FitAddon } from '@xterm/addon-fit'
import type { SearchAddon } from '@xterm/addon-search'
import { MONO_STACK } from '../core/util'
import { buildTheme } from './ship-terminal.theme'

export interface TerminalInitOptions {
  container: HTMLDivElement
  host: Element
  rows: number
  cols: number
  fontSize: number
  readonly: boolean
  onData: (data: string) => void
  onSearchToggle: () => void
}

export interface TerminalBundle {
  terminal: Terminal
  fitAddon: FitAddon
  searchAddon: SearchAddon
}

/**
 * Inject the minimal xterm.js CSS into a shadow root via adoptedStyleSheets.
 * xterm.js needs certain positioning rules for its canvas layers.
 */
export async function injectXtermCSS(root: ShadowRoot): Promise<void> {
  const sheet = new CSSStyleSheet()
  try {
    await sheet.replace(`
      .xterm { position: relative; user-select: none; }
      .xterm.focus, .xterm:focus { outline: none; }
      .xterm .xterm-helpers { position: absolute; top: 0; z-index: 5; }
      .xterm .xterm-helper-textarea {
        padding: 0; border: 0; margin: 0;
        position: absolute; opacity: 0; left: -9999em; top: 0;
        width: 0; height: 0; z-index: -5;
        white-space: nowrap; overflow: hidden; resize: none;
      }
      .xterm .composition-view {
        display: none; position: absolute;
        white-space: nowrap; z-index: 1;
      }
      .xterm .xterm-viewport {
        background-color: transparent !important;
        overflow-y: scroll; cursor: default;
        position: absolute; right: 0; left: 0; top: 0; bottom: 0;
      }
      .xterm .xterm-screen { position: relative; }
      .xterm .xterm-screen canvas { position: absolute; left: 0; top: 0; }
      .xterm .xterm-decoration-container {
        position: absolute; top: 0; left: 0; right: 0;
      }
    `)
    root.adoptedStyleSheets = [...root.adoptedStyleSheets, sheet]
  } catch { /* adoptedStyleSheets may not be available */ }
}

/**
 * Lazy-load xterm.js + addons, create a Terminal instance, open it in
 * the container, and wire up input/keyboard handlers.
 */
export async function createTerminal(opts: TerminalInitOptions): Promise<TerminalBundle> {
  const [
    { Terminal },
    { FitAddon },
    { WebLinksAddon },
    { SearchAddon },
    webglMod,
  ] = await Promise.all([
    import('@xterm/xterm'),
    import('@xterm/addon-fit'),
    import('@xterm/addon-web-links'),
    import('@xterm/addon-search'),
    import('@xterm/addon-webgl').catch(() => null),
  ])

  const term = new Terminal({
    rows: opts.rows,
    cols: opts.cols,
    fontSize: opts.fontSize,
    fontFamily: MONO_STACK,
    theme: buildTheme(opts.host),
    cursorBlink: !opts.readonly,
    disableStdin: opts.readonly,
    allowProposedApi: true,
    convertEol: true,
  })

  const fitAddon = new FitAddon()
  const searchAddon = new SearchAddon()

  term.loadAddon(fitAddon)
  term.loadAddon(new WebLinksAddon())
  term.loadAddon(searchAddon)

  // Prefer WebGL renderer — avoids canvas renderer's first-row corruption bug.
  // Falls back to canvas if WebGL is unavailable (old hardware, headless, etc).
  if (webglMod) {
    try {
      const webgl = new webglMod.WebglAddon()
      webgl.onContextLoss(() => { webgl.dispose() })
      term.loadAddon(webgl)
    } catch { /* WebGL unavailable — canvas fallback */ }
  }

  // Delay open() until the container has a layout — xterm's canvas renderer
  // produces garbled first-row glyphs if it initializes before the container
  // has computed dimensions.
  await new Promise<void>((resolve) => requestAnimationFrame(() => {
    term.open(opts.container)
    resolve()
  }))

  // Emit user input data for interactive mode
  if (!opts.readonly) {
    term.onData(opts.onData)
  }

  // Ctrl+F / Cmd+F for search
  term.attachCustomKeyEventHandler((ev: KeyboardEvent) => {
    if ((ev.ctrlKey || ev.metaKey) && ev.key === 'f' && ev.type === 'keydown') {
      ev.preventDefault()
      opts.onSearchToggle()
      return false
    }
    return true
  })

  return { terminal: term, fitAddon, searchAddon }
}
