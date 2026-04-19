/**
 * <ship-toast-container>
 *
 * Fixed-position stacking host for <ship-toast> children. Handles:
 *
 *   - position channel (six slots: top/bottom × left/center/right)
 *   - aria-live region semantics on the host (role=region, aria-label)
 *   - max-visible overflow policy — older toasts past `max-visible` call
 *     their own .close() to animate out before removal
 *   - per-position slide-direction hinting (data-slide-from on each child)
 *
 * Public methods:
 *   - show(options): Programmatic toast API. Creates a <ship-toast>, slots
 *     the title/body/actions, appends to the container, and returns the
 *     element so call-sites can await `.close()` or listen to ship-dismiss.
 *
 * Attributes:
 *   - position: top-left | top-center | top-right
 *               bottom-left | bottom-center | bottom-right (default bottom-right)
 *   - max-visible: integer, extra toasts are closed from the oldest end.
 *                  Default 5. 0 = unlimited.
 *
 * Slots:
 *   - (default) <ship-toast> children, stacked.
 *
 * Events (re-emitted from children):
 *   - ship-dismiss — bubbles from individual toasts.
 */

import { LitElement, html } from 'lit'
import { shipToastContainerStyles } from './ship-toast-container.styles'
import type { ShipToast } from './ship-toast'

export type ToastPosition =
  | 'top-left' | 'top-center' | 'top-right'
  | 'bottom-left' | 'bottom-center' | 'bottom-right'

export interface ShowToastOptions {
  tone?: ShipToast['tone']
  size?: ShipToast['size']
  duration?: number
  dismissible?: boolean
  showProgress?: boolean
  title?: string
  description?: string
  /** Optional HTML string for a trailing action. Slotted into `actions`. */
  actionHtml?: string
}

// Map container position → toast slide-in direction. Top/bottom trumps
// left/right because the vertical motion reads more naturally as "entering
// the stack."
const SLIDE_FROM: Record<ToastPosition, 'top' | 'bottom' | 'left' | 'right'> = {
  'top-left': 'top',
  'top-center': 'top',
  'top-right': 'top',
  'bottom-left': 'bottom',
  'bottom-center': 'bottom',
  'bottom-right': 'bottom',
}

export class ShipToastContainer extends LitElement {
  static styles = shipToastContainerStyles

  static properties = {
    position: { type: String, reflect: true },
    maxVisible: { type: Number, reflect: true, attribute: 'max-visible' },
  }

  position: ToastPosition = 'top-right'
  maxVisible = 5

  connectedCallback() {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'region')
    if (!this.hasAttribute('aria-label'))
      this.setAttribute('aria-label', 'Notifications')
    // Polite live-region by default. Individual toasts escalate to
    // role="alert"/aria-live="assertive" on destructive tones.
    if (!this.hasAttribute('aria-live')) this.setAttribute('aria-live', 'polite')
  }

  /** Propagate slide-direction hints to every child so animations are
   *  position-aware, and enforce max-visible. */
  private _onSlotChange = (e: Event) => {
    const slot = e.target as HTMLSlotElement
    const dir = SLIDE_FROM[this.position] ?? 'bottom'
    const children = slot
      .assignedElements({ flatten: true })
      .filter((n): n is ShipToast => n.tagName?.toLowerCase() === 'ship-toast')
    for (const t of children) {
      t.setAttribute('data-slide-from', dir)
    }
    this._enforceMaxVisible(children)
  }

  private _enforceMaxVisible(children: ShipToast[]) {
    if (this.maxVisible <= 0 || children.length <= this.maxVisible) return
    // Oldest = first child in DOM order. For bottom stacks the visual top
    // is the last child, but DOM order is what callers control so we stick
    // with "first = oldest." Trim from the start.
    const overflow = children.length - this.maxVisible
    for (let i = 0; i < overflow; i++) {
      const victim = children[i]
      // Prefer animated close when available; fall back to sync dismiss.
      if (typeof victim.close === 'function') victim.close('programmatic')
      else victim.dismiss?.('programmatic')
    }
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has('position')) {
      // Re-apply slide-from to already-rendered children after a position
      // change. New children pick it up via slotchange.
      const dir = SLIDE_FROM[this.position] ?? 'bottom'
      for (const child of this.querySelectorAll('ship-toast')) {
        child.setAttribute('data-slide-from', dir)
      }
    }
  }

  /**
   * Programmatic toast API. Creates a <ship-toast>, populates slots from
   * the options, appends to the container, and returns the element so
   * call-sites can `await toast.close()` or subscribe to ship-dismiss.
   *
   * Matches the sonner/shadcn ergonomics while staying framework-free so
   * daemon-rendered views can call it from Datastar-emitted inline scripts.
   */
  show(opts: ShowToastOptions = {}): ShipToast {
    const doc = this.ownerDocument ?? document
    const t = doc.createElement('ship-toast') as ShipToast
    if (opts.tone) t.setAttribute('tone', opts.tone)
    if (opts.size) t.setAttribute('size', opts.size)
    if (typeof opts.duration === 'number') t.duration = opts.duration
    t.dismissible = opts.dismissible !== false
    if (opts.dismissible === false) t.removeAttribute('dismissible')
    else t.setAttribute('dismissible', '')
    if (opts.showProgress) t.setAttribute('show-progress', '')
    if (opts.title) {
      const title = doc.createElement('span')
      title.setAttribute('slot', 'title')
      title.textContent = opts.title
      t.appendChild(title)
    }
    if (opts.description) {
      t.appendChild(doc.createTextNode(opts.description))
    }
    if (opts.actionHtml) {
      const wrap = doc.createElement('span')
      wrap.setAttribute('slot', 'actions')
      wrap.innerHTML = opts.actionHtml
      t.appendChild(wrap)
    }
    this.appendChild(t)
    return t
  }

  render() {
    return html`<slot @slotchange=${this._onSlotChange}></slot>`
  }
}

if (
  typeof customElements !== 'undefined' &&
  !customElements.get('ship-toast-container')
) {
  customElements.define('ship-toast-container', ShipToastContainer)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-toast-container': ShipToastContainer
  }
}
