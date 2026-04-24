/**
 * <ship-drawer>
 *
 * Side-sheet drawer built on the native `<dialog>` element. Mirrors
 * ship-dialog but slides in from a side edge rather than centering.
 *
 * Attributes:
 *   - open: boolean (reflect) — drawer visibility state.
 *   - side: 'right' | 'left' — which edge to slide from (default 'right').
 *   - size: 'sm' | 'md' | 'lg' — panel width channel (default 'md').
 *   - dismissible: boolean — Escape/backdrop click close (default true).
 *
 * Slots:
 *   - title — header title text
 *   - description — muted subtitle under title
 *   - (default) — body content (scrolls)
 *   - footer — sticky bottom actions
 *
 * Parts: dialog, header, body, footer, close
 *
 * Events:
 *   - ship-close — fires when user dismisses via Escape, backdrop, or ×
 *   - ship-dismiss — fires when the × button is clicked specifically
 *   - ship-open — fires after the drawer becomes visible
 */

import { LitElement, html } from 'lit'
import { shipDrawerStyles } from './ship-drawer.styles'
import { FocusReturn, focusableWithin } from './focus'

type Side = 'right' | 'left'
type Size = 'sm' | 'md' | 'lg'

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

export class ShipDrawer extends LitElement {
  static styles = shipDrawerStyles

  static properties = {
    open: { type: Boolean, reflect: true },
    side: { type: String, reflect: true },
    size: { type: String, reflect: true },
    dismissible: { type: Boolean },
  }

  open = false
  side: Side = 'right'
  size: Size = 'md'
  dismissible = true

  private static _idSeq = 0
  private uid = ++ShipDrawer._idSeq
  private focusReturn = new FocusReturn()
  private hasScrollLock = false
  private get headerId(): string { return `ship-drawer-header-${this.uid}` }
  private get bodyId(): string { return `ship-drawer-body-${this.uid}` }

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
      try { this.dialogEl.showModal() } catch { /* jsdom */ }
      if (!this.hasScrollLock) {
        acquireScrollLock()
        this.hasScrollLock = true
      }
      queueMicrotask(() => this.focusInitial())
      this.dispatchEvent(new CustomEvent('ship-open', { bubbles: true, composed: true }))
    } else if (!this.open && this.dialogEl.open) {
      this.dialogEl.close()
    }
  }

  show() { this.open = true }
  close() { this.open = false }

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
    const lightAutofocus = this.querySelector<HTMLElement>('[autofocus]')
    const shadowAutofocus = root.querySelector<HTMLElement>('[autofocus]')
    const autofocus = lightAutofocus ?? shadowAutofocus
    if (autofocus) { autofocus.focus(); return }
    const focusables = this.collectFocusables()
    const primary = focusables.find((el) => !el.classList.contains('close')) ?? focusables[0]
    if (primary) primary.focus()
    else root.focus()
  }

  private onClose = () => {
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
    if (e.target === this.dialogEl) this.close()
  }

  private onSentinelFocus = (which: 'start' | 'end') => (_e: FocusEvent) => {
    const items = this.collectFocusables()
    if (items.length === 0) return
    const target = which === 'start' ? items[items.length - 1] : items[0]
    target.focus()
    // Fall back to first native focusable descendant if element itself isn't focusable.
    if (document.activeElement === document.body) {
      const leaf =
        (target as HTMLElement & { shadowRoot?: ShadowRoot | null }).shadowRoot?.querySelector<HTMLElement>(
          'button, input, select, textarea, [tabindex]:not([tabindex^="-"])'
        ) ??
        target.querySelector<HTMLElement>(
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
        role="dialog"
        aria-modal="true"
        aria-labelledby=${this.headerId}
        aria-describedby=${this.bodyId}
        @close=${this.onClose}
        @click=${this.onBackdropClick}
      >
        <span tabindex="0" class="sentinel" aria-hidden="true"
          @focus=${this.onSentinelFocus('start')}></span>
        <div class="header" part="header" id=${this.headerId}>
          <div class="header-content">
            <div class="title"><slot name="title"></slot></div>
            <div class="description"><slot name="description"></slot></div>
          </div>
          <button
            class="close"
            part="close"
            type="button"
            aria-label="Close drawer"
            @click=${this.onDismiss}
          >
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor"
              stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"
              aria-hidden="true" focusable="false">
              <path d="M3.5 3.5l9 9M12.5 3.5l-9 9"></path>
            </svg>
          </button>
        </div>
        <div class="body" part="body" id=${this.bodyId}>
          <slot></slot>
        </div>
        <div class="footer" part="footer">
          <slot name="footer"></slot>
        </div>
        <span tabindex="0" class="sentinel" aria-hidden="true"
          @focus=${this.onSentinelFocus('end')}></span>
      </dialog>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-drawer')) {
  customElements.define('ship-drawer', ShipDrawer)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-drawer': ShipDrawer
  }
}
