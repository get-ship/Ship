/**
 * <ship-dialog>
 *
 * Modal dialog built on the native `<dialog>` element. Matches shadcn/ui +
 * Radix Dialog in visual polish: fade + scale entrance, backdrop with
 * optional blur, size channel, focus trap, scroll lock, and focus return
 * on close.
 *
 * Open the dialog via the `open` attribute, `el.show()`, or `el.showModal()`.
 * Close it via Escape, backdrop click (when `dismissible`), the × button,
 * or `el.close()`.
 *
 * Attributes:
 *   - open: boolean — reflects the visible modal state.
 *   - size: sm | md | lg | full — max-width channel (default md = 500px).
 *   - backdrop: default | blur | solid — backdrop treatment.
 *   - dismissible: boolean — clicking the backdrop closes (default true).
 *   - no-close-button: boolean — hides the × close button.
 *   - no-header / no-footer: boolean — suppress the respective region
 *     when its slot is unused.
 *   - role-override: dialog | alertdialog — overrides the role (used by
 *     <ship-alert-dialog>). Default: 'dialog'.
 *
 * Slots:
 *   - header — title row content. Wrap a description in
 *     <span class="ship-dialog-description"> for muted subtitle styling.
 *   - (default) — body content
 *   - footer — action row, right-aligned by default
 *
 * Parts:
 *   - dialog — the native <dialog>
 *   - header / body / footer — structural regions
 *   - close — the × close button
 *
 * Events:
 *   - ship-open — dispatched after the dialog becomes visible
 *   - ship-close — dispatched after the dialog fully closes
 *   - ship-dismiss — dispatched when the × close button is clicked
 */

import { LitElement, html } from 'lit'
import { shipDialogStyles } from './ship-dialog.styles'
import { FocusReturn, focusableWithin } from './focus'

type Size = 'sm' | 'md' | 'lg' | 'full'
type Backdrop = 'default' | 'blur' | 'solid'

let scrollLockCount = 0
let savedBodyOverflow: string | null = null
let savedBodyPaddingRight: string | null = null

function acquireScrollLock(): void {
  if (typeof document === 'undefined') return
  scrollLockCount += 1
  if (scrollLockCount > 1) return
  const body = document.body
  savedBodyOverflow = body.style.overflow
  savedBodyPaddingRight = body.style.paddingRight
  // Preserve layout when the scrollbar disappears on lock.
  const scrollbarW = window.innerWidth - document.documentElement.clientWidth
  if (scrollbarW > 0) {
    const current = parseFloat(getComputedStyle(body).paddingRight || '0') || 0
    body.style.paddingRight = `${current + scrollbarW}px`
  }
  body.style.overflow = 'hidden'
}

function releaseScrollLock(): void {
  if (typeof document === 'undefined') return
  if (scrollLockCount === 0) return
  scrollLockCount -= 1
  if (scrollLockCount > 0) return
  const body = document.body
  body.style.overflow = savedBodyOverflow ?? ''
  body.style.paddingRight = savedBodyPaddingRight ?? ''
  savedBodyOverflow = null
  savedBodyPaddingRight = null
}

export class ShipDialog extends LitElement {
  static styles = shipDialogStyles

  static properties = {
    open: { type: Boolean, reflect: true },
    size: { type: String, reflect: true },
    backdrop: { type: String, reflect: true },
    dismissible: { type: Boolean },
    noCloseButton: { type: Boolean, reflect: true, attribute: 'no-close-button' },
    noHeader: { type: Boolean, reflect: true, attribute: 'no-header' },
    noFooter: { type: Boolean, reflect: true, attribute: 'no-footer' },
    dialogRole: { type: String, attribute: 'role-override' },
  }

  open = false
  size: Size = 'md'
  backdrop: Backdrop = 'default'
  dismissible = true
  noCloseButton = false
  noHeader = false
  noFooter = false
  dialogRole: 'dialog' | 'alertdialog' = 'dialog'

  private static _idSeq = 0
  private uid = ++ShipDialog._idSeq
  private focusReturn = new FocusReturn()
  private hasScrollLock = false
  private get headerId(): string { return `ship-dialog-header-${this.uid}` }
  private get bodyId(): string { return `ship-dialog-body-${this.uid}` }

  get dialogEl(): HTMLDialogElement {
    return this.renderRoot.querySelector('dialog') as HTMLDialogElement
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    if (this.hasScrollLock) {
      releaseScrollLock()
      this.hasScrollLock = false
    }
  }

  updated(changed: Map<string, unknown>) {
    this.setAttribute('data-state', this.open ? 'open' : 'closed')
    if (!changed.has('open') || !this.dialogEl) return

    if (this.open && !this.dialogEl.open) {
      this.focusReturn.save()
      if (typeof this.dialogEl.showModal === 'function') {
        try { this.dialogEl.showModal() } catch { /* jsdom / re-entry */ }
      }
      if (!this.hasScrollLock) {
        acquireScrollLock()
        this.hasScrollLock = true
      }
      // Move focus to the first focusable element after the entrance tick.
      queueMicrotask(() => this.focusInitial())
      this.dispatchEvent(new CustomEvent('ship-open', { bubbles: true, composed: true }))
    } else if (!this.open && this.dialogEl.open) {
      this.dialogEl.close()
    }
  }

  show() { this.open = true }
  showModal() { this.open = true }
  close() { this.open = false }

  /** Collect focusables from both the shadow dialog (close button) and the
   * host's light-DOM children (slotted body/footer controls). Sentinels
   * are excluded. Order: light-DOM (natural tab order for body/footer)
   * first, then the shadow close button. */
  private collectFocusables(): HTMLElement[] {
    const shadowRoot = this.renderRoot.querySelector('dialog')
    const shadowFocusables = shadowRoot
      ? focusableWithin(shadowRoot).filter((el) => !el.classList.contains('sentinel'))
      : []
    const lightFocusables = focusableWithin(this)
    return [...lightFocusables, ...shadowFocusables]
  }

  private focusInitial(): void {
    const root = this.dialogEl
    if (!root) return
    // Prefer an element the caller marked with autofocus (light or shadow).
    const lightAutofocus = this.querySelector<HTMLElement>('[autofocus]')
    const shadowAutofocus = root.querySelector<HTMLElement>('[autofocus]')
    const autofocus = lightAutofocus ?? shadowAutofocus
    if (autofocus) { autofocus.focus(); return }
    const focusables = this.collectFocusables()
    // Skip the close button as the initial focus target — Radix does the
    // same: it's rarely the user's intent and it's always reachable by Tab.
    const primary = focusables.find((el) => !el.classList.contains('close')) ?? focusables[0]
    if (primary) primary.focus()
    else root.focus()
  }

  private onClose = () => {
    // Fired by the native <dialog> on Escape and programmatic .close().
    this.open = false
    if (this.hasScrollLock) {
      releaseScrollLock()
      this.hasScrollLock = false
    }
    this.focusReturn.restore()
    this.dispatchEvent(new CustomEvent('ship-close', { bubbles: true, composed: true }))
  }

  private onBackdropClick = (e: MouseEvent) => {
    if (!this.dismissible) return
    // Click landed directly on <dialog> (outside any child) = backdrop click.
    if (e.target === this.dialogEl) this.close()
  }

  /** Sentinel-based focus trap. Two invisible focusable divs bracket the
   * dialog contents — if Tab lands on either sentinel we bounce focus to
   * the opposite end of the ring. Works across shadow boundaries without
   * requiring us to enumerate cross-boundary focusables. */
  private onSentinelFocus = (which: 'start' | 'end') => (_e: FocusEvent) => {
    const items = this.collectFocusables()
    if (items.length === 0) return
    const target = which === 'start' ? items[items.length - 1] : items[0]
    // Prefer focusing the target directly; if it's a custom element without
    // delegated focus, fall back to the first native focusable descendant.
    this.focusWithFallback(target)
  }

  private focusWithFallback(el: HTMLElement): void {
    el.focus()
    // If the focus landed back on body (host element wasn't focusable), dig
    // into its shadow or descendants to find the real leaf.
    if (document.activeElement === document.body || document.activeElement === el) {
      const leaf =
        (el as HTMLElement & { shadowRoot?: ShadowRoot | null }).shadowRoot?.querySelector<HTMLElement>(
          'button, input, select, textarea, [tabindex]:not([tabindex^="-"])'
        ) ??
        el.querySelector<HTMLElement>(
          'button, input, select, textarea, [tabindex]:not([tabindex^="-"])'
        )
      leaf?.focus()
    }
  }

  private onDismiss = () => {
    this.dispatchEvent(new CustomEvent('ship-dismiss', { bubbles: true, composed: true }))
    this.close()
  }

  render() {
    return html`
      <dialog
        part="dialog"
        role=${this.dialogRole}
        aria-modal="true"
        aria-labelledby=${this.headerId}
        aria-describedby=${this.bodyId}
        @close=${this.onClose}
        @click=${this.onBackdropClick}
      >
        <span tabindex="0" class="sentinel" aria-hidden="true"
          @focus=${this.onSentinelFocus('start')}></span>
        <div class="wrap">
          <button
            class="close"
            part="close"
            type="button"
            aria-label="Close dialog"
            @click=${this.onDismiss}
          >
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor"
              stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"
              aria-hidden="true" focusable="false">
              <path d="M3.5 3.5l9 9M12.5 3.5l-9 9"></path>
            </svg>
          </button>
          <div class="header" part="header" id=${this.headerId}>
            <slot name="header"></slot>
          </div>
          <div class="body" part="body" id=${this.bodyId}>
            <slot></slot>
          </div>
          <div class="footer" part="footer">
            <slot name="footer"></slot>
          </div>
        </div>
        <span tabindex="0" class="sentinel" aria-hidden="true"
          @focus=${this.onSentinelFocus('end')}></span>
      </dialog>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-dialog')) {
  customElements.define('ship-dialog', ShipDialog)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-dialog': ShipDialog
  }
}
