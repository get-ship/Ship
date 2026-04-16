/**
 * <ship-toast>
 *
 * Transient notification card. Polished to shadcn Sonner / Radix Toast
 * quality: tone channel with auto-icon, size channel, entrance animation,
 * hover-to-pause auto-dismiss, optional progress bar, and an exit
 * animation available via `.close()` for UI call-sites that want it.
 *
 * Attributes:
 *   - tone: default | info | success | warning | destructive | accent | muted
 *   - size: sm | md | lg (default md)
 *   - duration: ms before auto-dismiss; 0 = persistent. Default 5000.
 *   - dismissible: boolean, shows the close button (default true).
 *   - show-progress: boolean, renders a bottom progress bar tied to duration.
 *   - closing: reflected while the exit animation plays; auto-cleared on
 *     removal. Not typically set by consumers.
 *
 * Slots:
 *   - icon — leading icon. If empty, a tone-appropriate SVG is auto-filled.
 *   - title — the heading line.
 *   - (default) body text.
 *   - actions — trailing action row (one CTA recommended).
 *
 * Parts:
 *   - toast — the card surface element.
 *   - close — the dismiss button.
 *
 * Events:
 *   - ship-dismiss — fires when the toast is removed (manually, via timer,
 *     or programmatic .close()). detail: { reason: 'manual'|'timeout'|'programmatic' }
 *
 * Public methods:
 *   - dismiss(reason?) — fires ship-dismiss and removes the host synchronously.
 *     Used by the close button so existing tests keep sync remove semantics.
 *   - close(reason?) — plays exit animation, then dismisses. Call from
 *     call-sites that want the polish; falls back to sync dismiss if
 *     reduced-motion is active.
 */

import { LitElement, html } from 'lit'
import { shipToastStyles } from './ship-toast.styles'
import type { Size, Tone } from '../util'

// Re-export the container from this module so existing callers that
// `import './ship-toast'` (and the colocated test) still find
// ShipToastContainer. The class itself lives in its own file.
export { ShipToastContainer } from './ship-toast-container'
import './ship-toast-container'

type DismissReason = 'manual' | 'timeout' | 'programmatic'

// Tone → inline SVG icon (Lucide-style 24×24 strokes at 2px). Kept inline
// so the toast renders correctly in daemon-rendered views where no icon
// font is guaranteed. Consumers can override via the `icon` slot.
const TONE_ICON: Record<string, string> = {
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 8h.01"/><path d="M11 12h1v4h1"/>',
  success: '<circle cx="12" cy="12" r="9"/><path d="m8.5 12.5 2.5 2.5 4.5-5"/>',
  warning:
    '<path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"/><line x1="12" x2="12" y1="9" y2="13"/><line x1="12" x2="12.01" y1="17" y2="17"/>',
  destructive:
    '<circle cx="12" cy="12" r="9"/><line x1="15" x2="9" y1="9" y2="15"/><line x1="9" x2="15" y1="9" y2="15"/>',
  accent:
    '<path d="M12 2 3 14h7l-1 8 10-12h-7l1-8Z"/>',
}

export class ShipToast extends LitElement {
  static styles = shipToastStyles

  static properties = {
    tone: { type: String, reflect: true },
    size: { type: String, reflect: true },
    duration: { type: Number },
    dismissible: { type: Boolean, reflect: true },
    showProgress: { type: Boolean, reflect: true, attribute: 'show-progress' },
  }

  tone?: Tone
  size: Size = 'md'
  duration = 5000
  dismissible = true
  showProgress = false

  private _timer: ReturnType<typeof setTimeout> | null = null
  private _remaining = 0
  private _startedAt = 0
  private _removed = false

  connectedCallback() {
    super.connectedCallback()
    // Announce politely by default; destructive/warning escalate to
    // aria-live="assertive" via the template role/aria-live attrs.
    if (!this.hasAttribute('tabindex')) this.tabIndex = 0
    this._scheduleDismiss()
    this.addEventListener('mouseenter', this._onPause)
    this.addEventListener('mouseleave', this._onResume)
    this.addEventListener('focusin', this._onPause)
    this.addEventListener('focusout', this._onResume)
    this.addEventListener('keydown', this._onKeydown)
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this._clearTimer()
    this.removeEventListener('mouseenter', this._onPause)
    this.removeEventListener('mouseleave', this._onResume)
    this.removeEventListener('focusin', this._onPause)
    this.removeEventListener('focusout', this._onResume)
    this.removeEventListener('keydown', this._onKeydown)
  }

  private _scheduleDismiss() {
    this._clearTimer()
    if (this.duration > 0) {
      this._remaining = this.duration
      this._startedAt = Date.now()
      this._timer = setTimeout(() => this.dismiss('timeout'), this.duration)
      // Expose duration to the CSS progress bar keyframe.
      this.style.setProperty('--toast-duration', `${this.duration}ms`)
    }
  }

  private _clearTimer() {
    if (this._timer) {
      clearTimeout(this._timer)
      this._timer = null
    }
  }

  private _onPause = () => {
    if (!this._timer) return
    const elapsed = Date.now() - this._startedAt
    this._remaining = Math.max(0, this._remaining - elapsed)
    this._clearTimer()
    this.toggleAttribute('data-paused', true)
  }

  private _onResume = () => {
    if (this._removed || this.duration <= 0) return
    if (this._remaining <= 0) return
    this.toggleAttribute('data-paused', false)
    this._startedAt = Date.now()
    this._timer = setTimeout(() => this.dismiss('timeout'), this._remaining)
  }

  private _onKeydown = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && this.dismissible) {
      e.stopPropagation()
      this.dismiss('manual')
    }
  }

  /**
   * Synchronously fire `ship-dismiss` and detach the host. Used by the
   * close button and auto-timeout. Matches the contract tested in
   * ship-toast.test.ts (isConnected → false immediately after click).
   */
  dismiss(reason: DismissReason = 'manual') {
    if (this._removed) return
    this._removed = true
    this._clearTimer()
    this.dispatchEvent(new CustomEvent('ship-dismiss', {
      detail: { reason },
      bubbles: true,
      composed: true,
    }))
    this.remove()
  }

  /**
   * Animated dismissal — plays the exit keyframe, then removes. Useful for
   * galleries and long-lived notifications where the extra 180ms feels
   * more considered than a pop-out. Falls back to sync dismiss when
   * prefers-reduced-motion is set.
   */
  async close(reason: DismissReason = 'manual') {
    if (this._removed) return
    const prefersReduce =
      typeof window !== 'undefined' &&
      window.matchMedia?.('(prefers-reduced-motion: reduce)').matches
    if (prefersReduce) {
      this.dismiss(reason)
      return
    }
    this._clearTimer()
    this.toggleAttribute('data-closing', true)
    await new Promise<void>((resolve) => {
      const onEnd = () => {
        this.removeEventListener('animationend', onEnd)
        resolve()
      }
      this.addEventListener('animationend', onEnd)
      // Safety net — if animationend never fires (e.g. display:none), resolve
      // after the medium duration + a frame. Matches --duration-medium.
      setTimeout(() => resolve(), 260)
    })
    this.dismiss(reason)
  }

  private _onCloseClick = () => this.dismiss('manual')

  private _handleSlotChange = (e: Event) => {
    const slot = e.target as HTMLSlotElement
    const name = slot.name || 'default'
    const filled = slot.assignedNodes({ flatten: true }).some((n) => {
      if (n.nodeType === Node.ELEMENT_NODE) return true
      return n.textContent?.trim().length ?? 0 > 0
    })
    if (name === 'icon') this.toggleAttribute('data-has-slotted-icon', filled)
    if (name === 'title') this.toggleAttribute('data-no-title', !filled)
    if (name === 'actions') this.toggleAttribute('data-no-actions', !filled)
  }

  private _iconMarkup() {
    const key = (this.tone ?? 'default') as keyof typeof TONE_ICON
    const svg = TONE_ICON[key]
    if (!svg) return html`<slot name="icon"></slot>`
    return html`
      <slot name="icon" @slotchange=${this._handleSlotChange}>
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
          .innerHTML=${svg}
        ></svg>
      </slot>
    `
  }

  render() {
    const assertive = this.tone === 'destructive' || this.tone === 'warning'
    const role = assertive ? 'alert' : 'status'
    const live = assertive ? 'assertive' : 'polite'
    // When tone is default/muted and no icon is slotted, collapse the icon
    // column so the content takes over. The auto-icon only activates for
    // semantic tones (info/success/warning/destructive/accent).
    const hasAutoIcon = !!this.tone && this.tone !== 'default' && this.tone !== 'muted'
    this.toggleAttribute('data-no-icon', !hasAutoIcon && !this.hasAttribute('data-has-slotted-icon'))

    return html`
      <div class="toast" role=${role} aria-live=${live} aria-atomic="true" part="toast">
        <span class="icon">${this._iconMarkup()}</span>
        <div class="content">
          <div class="title">
            <slot name="title" @slotchange=${this._handleSlotChange}></slot>
          </div>
          <div class="body"><slot></slot></div>
          <div class="actions">
            <slot name="actions" @slotchange=${this._handleSlotChange}></slot>
          </div>
        </div>
        ${this.dismissible
          ? html`
              <button
                class="close"
                part="close"
                type="button"
                aria-label="Dismiss notification"
                @click=${this._onCloseClick}
              >
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            `
          : null}
        ${this.showProgress && this.duration > 0
          ? html`<span class="progress" aria-hidden="true"></span>`
          : null}
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-toast')) {
  customElements.define('ship-toast', ShipToast)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-toast': ShipToast
  }
}
