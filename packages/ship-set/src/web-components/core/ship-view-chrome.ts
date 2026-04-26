/**
 * <ship-view-chrome>
 *
 * A view's declarative contribution to the shell's topbar + statusbar.
 * Views never render their own breadcrumb, branch pill, or status bar —
 * they emit a single `<ship-view-chrome>` and the shell paints the chrome
 * on their behalf. Keeps first-party and third-party views symmetric.
 *
 * Usage:
 *
 *   <ship-view-chrome title="Git" branch="main" ahead="0" behind="0">
 *     <ship-button slot="actions" ...>Refresh</ship-button>
 *     <ship-chip slot="status-end" size="sm">22 changed</ship-chip>
 *   </ship-view-chrome>
 *
 * Attributes:
 *   - title    — overrides the breadcrumb leaf.
 *   - branch   — short branch name rendered in the statusbar.
 *   - ahead    — commits ahead of upstream; renders as ↑N.
 *   - behind   — commits behind; renders as ↓N.
 *   - tone     — tints the statusbar status dot: default | success | warning | destructive | accent | muted
 *   - busy     — boolean; pulses the status dot.
 *
 * Slots:
 *   - actions       — appended to the right of the topbar.
 *   - status-start  — inserted after branch in the statusbar.
 *   - status-end    — inserted at the right of the statusbar.
 *
 * Mechanism:
 *   Same-DOM: the element physically moves its slot children into
 *   shell-owned containers (#view-topbar-actions, #view-status-start,
 *   #view-status-end) via appendChild. Direct DOM move preserves
 *   Datastar + Lit bindings since they attach to elements, not positions.
 *   Attribute-driven fields are written into #breadcrumbs-title,
 *   #status-branch, #status-ab. On disconnect (route change wipes the
 *   view root), contributed nodes and text are cleared.
 *
 * No-shell fallback: when no slot containers exist in the document, the
 * element is invisible and no-ops — safe to include in previews or tests.
 */

import { LitElement } from 'lit'

type SlotName = 'actions' | 'status-start' | 'status-end'

const SLOT_TARGETS: Record<SlotName, string> = {
  actions: '#view-topbar-actions',
  'status-start': '#view-status-start',
  'status-end': '#view-status-end',
}

const ATTR_TARGETS = {
  title: '#breadcrumbs-title',
  branch: '#status-branch',
} as const

export class ShipViewChrome extends LitElement {
  static properties = {
    title: { type: String, reflect: true },
    branch: { type: String, reflect: true },
    ahead: { type: Number, reflect: true },
    behind: { type: Number, reflect: true },
    tone: { type: String, reflect: true },
    busy: { type: Boolean, reflect: true },
  }

  // Lit disables Shadow DOM so slot children remain in the light DOM and
  // can be moved by us. We render nothing.
  createRenderRoot() { return this }

  title = ''
  branch = ''
  ahead = 0
  behind = 0
  tone: 'default' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
  busy = false

  private _moved: Partial<Record<SlotName, HTMLElement[]>> = {}

  connectedCallback() {
    super.connectedCallback()
    // Move on next frame so child nodes are parsed first (important for
    // HTML fragments inserted via innerHTML).
    queueMicrotask(() => this._contribute())
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this._withdraw()
  }

  updated() {
    // Re-apply attribute-driven text regions on every reactive update.
    this._applyAttrs()
  }

  private _contribute() {
    this.style.display = 'none'
    for (const name of Object.keys(SLOT_TARGETS) as SlotName[]) {
      const target = document.querySelector<HTMLElement>(SLOT_TARGETS[name])
      if (!target) continue
      const sourced = Array.from(this.querySelectorAll<HTMLElement>(`[slot="${name}"]`))
      if (sourced.length === 0) continue
      for (const node of sourced) target.appendChild(node)
      this._moved[name] = sourced
    }
    this._applyAttrs()
  }

  private _withdraw() {
    // Clear only the nodes we contributed. Do not remove host-installed
    // chrome nodes from other views (they've already unmounted by now,
    // but be defensive).
    for (const name of Object.keys(this._moved) as SlotName[]) {
      const nodes = this._moved[name] ?? []
      for (const n of nodes) n.remove()
    }
    this._moved = {}
    this._clearAttrs()
  }

  private _applyAttrs() {
    if (Object.keys(this._moved).length === 0 && !this.isConnected) return

    if (this.title) {
      const t = document.querySelector<HTMLElement>(ATTR_TARGETS.title)
      if (t) t.textContent = this.title
    }

    // Only overwrite chrome fields the view actually declared.
    // A view that doesn't set `branch=`/`ahead=`/etc. should leave the
    // shell-owned defaults (current git branch, theme name) intact —
    // otherwise it wipes them on mount.
    //
    // We can't use `hasAttribute('branch')` here because Lit reflects
    // the default empty-string property to the attribute on construction,
    // so the attr is always present. Instead, treat empty/zero/default
    // values as "view did not contribute this field."
    if (this.branch) {
      const branch = document.querySelector<HTMLElement>(ATTR_TARGETS.branch)
      if (branch) branch.textContent = this.branch
    }

    if (this.ahead > 0 || this.behind > 0) {
      const ab = document.querySelector<HTMLElement>('#status-ab')
      if (ab) {
        const parts: string[] = []
        if (this.ahead > 0) parts.push(`↑${this.ahead}`)
        if (this.behind > 0) parts.push(`↓${this.behind}`)
        ab.textContent = parts.join(' ')
      }
    }

    if (this.tone !== 'default' || this.busy) {
      const dot = document.querySelector<HTMLElement>('#status-dot')
      if (dot) {
        dot.dataset.tone = this.tone
        if (this.busy) dot.setAttribute('data-busy', '')
        else dot.removeAttribute('data-busy')
      }
    }
  }

  private _clearAttrs() {
    // Mirror _applyAttrs: only clear chrome fields this view actually
    // contributed. The shell will repopulate its owned fields (branch,
    // theme) on the next render — but only if we didn't blank them.
    // Empty defaults => view did not contribute that field.
    const t = document.querySelector<HTMLElement>(ATTR_TARGETS.title)
    if (t) t.textContent = ''
    if (this.branch) {
      const branch = document.querySelector<HTMLElement>(ATTR_TARGETS.branch)
      if (branch) branch.textContent = ''
    }
    if (this.ahead > 0 || this.behind > 0) {
      const ab = document.querySelector<HTMLElement>('#status-ab')
      if (ab) ab.textContent = ''
    }
    if (this.tone !== 'default' || this.busy) {
      const dot = document.querySelector<HTMLElement>('#status-dot')
      if (dot) {
        delete dot.dataset.tone
        dot.removeAttribute('data-busy')
      }
    }
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-view-chrome')) {
  customElements.define('ship-view-chrome', ShipViewChrome)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-view-chrome': ShipViewChrome
  }
}
