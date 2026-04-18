/**
 * <ship-terminal> — Lit component wrapping xterm.js. Lazy-bundled
 * (`bundle:terminal`). Renders a terminal emulator for agent output
 * streaming, build logs, shell sessions, and pipeline step output.
 *
 * Attrs:
 *   rows      — terminal rows (default 24)
 *   cols      — terminal cols (default 80)
 *   font-size — font size override in px (default uses --text-sm)
 *   readonly  — disable user input (display-only mode)
 *   theme     — light | dark | auto (default auto)
 *   fit       — auto-fit to container dimensions
 *
 * Properties (JS):
 *   write(data: string)  — write text to terminal
 *   writeln(data: string) — write + newline
 *   clear()              — clear screen
 *   terminal             — raw xterm.js Terminal instance
 *
 * Events:
 *   ship-data  — { detail: { data } } when user types
 *   ship-ready — fired when terminal is initialized
 *
 * Slots: none
 * Parts: wrap (the terminal container)
 */

import { LitElement, html, nothing } from 'lit'
import { shipTerminalStyles } from './ship-terminal.styles'
import { buildTheme } from './ship-terminal.theme'
import { createTerminal, injectXtermCSS } from './ship-terminal.init'
import { defineOnce } from './util'

export class ShipTerminal extends LitElement {
  static styles = shipTerminalStyles

  static properties = {
    rows: { type: Number },
    cols: { type: Number },
    fontSize: { type: Number, attribute: 'font-size' },
    readonly: { type: Boolean, reflect: true },
    theme: { type: String, reflect: true },
    fit: { type: Boolean, reflect: true },
  }

  rows = 24
  cols = 80
  fontSize = 0
  readonly = false
  theme: 'light' | 'dark' | 'auto' = 'auto'
  fit = false

  private _terminal: import('@xterm/xterm').Terminal | null = null
  private _fitAddon: import('@xterm/addon-fit').FitAddon | null = null
  private _searchAddon: import('@xterm/addon-search').SearchAddon | null = null
  private _resizeObserver: ResizeObserver | null = null
  private _searchOpen = false
  private _searchQuery = ''

  /** Access the raw xterm.js Terminal instance. */
  get terminal(): import('@xterm/xterm').Terminal | null {
    return this._terminal
  }

  /** Write text to the terminal. */
  write(data: string): void { this._terminal?.write(data) }

  /** Write text + newline to the terminal. */
  writeln(data: string): void { this._terminal?.writeln(data) }

  /** Clear the terminal screen. */
  clear(): void { this._terminal?.clear() }

  disconnectedCallback(): void {
    super.disconnectedCallback()
    this._resizeObserver?.disconnect()
    this._resizeObserver = null
    if (this._terminal) {
      try { this._terminal.dispose() } catch { /* teardown race */ }
    }
    this._terminal = null
    this._fitAddon = null
    this._searchAddon = null
  }

  firstUpdated(): void { void this._init() }

  updated(changed: Map<string, unknown>): void {
    if (!this._terminal) return
    if (changed.has('theme')) {
      this._terminal.options.theme = buildTheme(this)
    }
    if (changed.has('fontSize') && this.fontSize > 0) {
      this._terminal.options.fontSize = this.fontSize
      this._fitAddon?.fit()
    }
    if (changed.has('fit')) {
      if (this.fit) {
        this._fitAddon?.fit()
        this._attachResizeObserver()
      } else {
        this._resizeObserver?.disconnect()
        this._resizeObserver = null
      }
    }
  }

  private async _init(): Promise<void> {
    const wrap = this.renderRoot.querySelector<HTMLDivElement>('[data-terminal]')
    if (!wrap) return

    await injectXtermCSS(this.renderRoot as ShadowRoot)

    const resolvedFontSize = this.fontSize > 0
      ? this.fontSize
      : this._resolveTokenFontSize()

    let bundle
    try {
      bundle = await createTerminal({
        container: wrap,
        host: this,
        rows: this.rows,
        cols: this.cols,
        fontSize: resolvedFontSize,
        readonly: this.readonly,
        onData: (data) => {
          this.dispatchEvent(new CustomEvent('ship-data', {
            detail: { data }, bubbles: true, composed: true,
          }))
        },
        onSearchToggle: () => this._toggleSearch(),
      })
    } catch {
      // xterm.js requires a real browser with canvas support.
      // In JSDOM or SSR environments, init will fail — this is expected.
      return
    }

    this._terminal = bundle.terminal
    this._fitAddon = bundle.fitAddon
    this._searchAddon = bundle.searchAddon

    if (this.fit) {
      bundle.fitAddon.fit()
      this._attachResizeObserver()
    }

    // xterm canvas renderer paints garbled glyphs on the first write (shadow
    // DOM bug). Workaround: sacrificial blank line + CSS clips row 0.
    // WebGL renderer doesn't have this issue, so only apply when on canvas.
    const isWebGL = !!(bundle.terminal as any)._core?._renderService?._renderer?.value?.constructor?.name?.includes('Webgl')
    if (!isWebGL) {
      bundle.terminal.writeln('')
      // Measure actual rendered row height for precise clipping.
      const wrap = this.renderRoot.querySelector('[data-terminal]') as HTMLElement
      const xtermRows = wrap?.querySelector('.xterm-rows')
      const firstRow = xtermRows?.children[0] as HTMLElement
      const secondRow = xtermRows?.children[1] as HTMLElement
      const rowH = (firstRow && secondRow)
        ? secondRow.offsetTop - firstRow.offsetTop + 8
        : Math.ceil(opts.fontSize * 1.6)
      this.style.setProperty('--xterm-row-h', `${rowH}px`)
      this.classList.add('canvas-fallback')
    }

    this.dispatchEvent(new CustomEvent('ship-ready', {
      bubbles: true, composed: true,
    }))
  }

  private _resolveTokenFontSize(): number {
    const raw = getComputedStyle(this).getPropertyValue('--text-sm').trim()
    const parsed = parseFloat(raw)
    return parsed > 0 ? parsed : 12
  }

  private _attachResizeObserver(): void {
    if (this._resizeObserver) return
    this._resizeObserver = new ResizeObserver(() => {
      this._fitAddon?.fit()
    })
    this._resizeObserver.observe(this)
  }

  private _toggleSearch(): void {
    this._searchOpen = !this._searchOpen
    this.requestUpdate()
    if (this._searchOpen) {
      requestAnimationFrame(() => {
        const input = this.renderRoot.querySelector<HTMLInputElement>('.search-bar input')
        input?.focus()
      })
    }
  }

  private _onSearchInput = (e: Event): void => {
    this._searchQuery = (e.target as HTMLInputElement).value
    if (this._searchQuery) this._searchAddon?.findNext(this._searchQuery)
  }

  private _onSearchKeydown = (e: KeyboardEvent): void => {
    if (e.key === 'Escape') {
      e.preventDefault()
      this._searchOpen = false
      this._searchQuery = ''
      this.requestUpdate()
      this._terminal?.focus()
    } else if (e.key === 'Enter') {
      e.preventDefault()
      if (e.shiftKey) {
        this._searchAddon?.findPrevious(this._searchQuery)
      } else {
        this._searchAddon?.findNext(this._searchQuery)
      }
    }
  }

  private _searchNext = (): void => {
    this._searchAddon?.findNext(this._searchQuery)
  }

  private _searchPrev = (): void => {
    this._searchAddon?.findPrevious(this._searchQuery)
  }

  render() {
    return html`
      <div class="terminal-wrap" part="wrap" data-terminal
        role=${this.readonly ? 'log' : 'terminal'}
        aria-label=${this.readonly ? 'Log output' : 'Terminal'}
        aria-readonly=${this.readonly ? 'true' : nothing}
      ></div>
      <div class="search-bar" data-open=${this._searchOpen ? 'true' : 'false'}>
        <input
          type="text"
          placeholder="Search..."
          aria-label="Search terminal"
          .value=${this._searchQuery}
          @input=${this._onSearchInput}
          @keydown=${this._onSearchKeydown}
        />
        <button @click=${this._searchPrev} aria-label="Previous match"
          title="Previous (Shift+Enter)">&#x2191;</button>
        <button @click=${this._searchNext} aria-label="Next match"
          title="Next (Enter)">&#x2193;</button>
      </div>
    `
  }
}

defineOnce('ship-terminal', ShipTerminal)

declare global {
  interface HTMLElementTagNameMap { 'ship-terminal': ShipTerminal }
}
