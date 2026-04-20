/**
 * <ship-auth-dialog>
 *
 * Domain wrapper for authentication modals. Composes:
 *   - <ship-dialog>   as the modal chrome (backdrop, focus trap, scroll lock)
 *   - <ship-tabs>     to switch between Sign in / Sign up
 *   - <ship-button>   for the primary submit and the OAuth buttons
 *
 * Usage:
 *   <ship-auth-dialog open mode="sign-in">
 *     <span slot="providers">
 *       <ship-button variant="outline">Continue with GitHub</ship-button>
 *       <ship-button variant="outline">Continue with Google</ship-button>
 *     </span>
 *     <span slot="terms">By continuing you agree to the Terms &amp; Privacy.</span>
 *   </ship-auth-dialog>
 *
 * Attributes:
 *   - open       — boolean, reflects the visible state
 *   - mode       — 'sign-in' | 'sign-up' (default 'sign-in')
 *   - loading    — disables the submit button, shows spinner
 *   - backdrop   — forwarded to <ship-dialog> (default | blur | solid)
 *
 * Slots:
 *   - header     — title row (default: "Sign in" / "Sign up" derived from mode)
 *   - providers  — OAuth buttons, rendered above the email form
 *   - terms      — fine-print below the submit button
 *
 * Events:
 *   - ship-auth-submit — CustomEvent<{ mode, email, password }>, fired when
 *                        the primary form is submitted. The consumer is
 *                        responsible for validation + network IO.
 *   - ship-mode-change — CustomEvent<{ mode }>, fired when the user switches
 *                        tabs between sign-in and sign-up.
 *   - ship-close       — re-dispatched from the nested <ship-dialog>.
 *
 * Parts:
 *   - dialog — the underlying <ship-dialog>
 *   - form   — the email/password form element
 *   - submit — the primary submit button
 */

import { LitElement, css, html, nothing } from 'lit'
import '../core/ship-dialog'
import '../core/ship-tabs'
import '../core/ship-button'
import '../core/ship-input'
import '../core/ship-separator'

type AuthMode = 'sign-in' | 'sign-up'

export class ShipAuthDialog extends LitElement {
  static styles = css`
    /* Closed state: the dialog is inert and its slotted providers/terms
     * must not leak into the host document's flow. Opening the dialog
     * flips the host to display:contents so the nested <ship-dialog>
     * can render its modal chrome normally. */
    :host { display: none; }
    :host([open]) { display: contents; }

    .body {
      display: flex;
      flex-direction: column;
      gap: 16px;
      padding: 4px 0 0;
      font-family: var(--font-body);
      color: var(--foreground);
    }

    .providers {
      display: flex;
      flex-direction: column;
      gap: 8px;
    }
    ::slotted(*) { width: 100%; }

    .or {
      display: flex;
      align-items: center;
      gap: 12px;
      color: var(--muted-foreground);
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }
    .or::before, .or::after {
      content: '';
      flex: 1;
      height: 1px;
      background: color-mix(in oklch, var(--border) 55%, transparent);
    }

    form {
      display: flex;
      flex-direction: column;
      gap: 12px;
    }
    label {
      display: flex;
      flex-direction: column;
      gap: 6px;
      font-size: 12px;
      color: var(--muted-foreground);
      text-transform: uppercase;
      letter-spacing: 0.06em;
      font-weight: 500;
    }
    input {
      height: 38px;
      padding: 0 12px;
      border: 1px solid var(--border);
      border-radius: 10px;
      background: var(--card);
      color: var(--foreground);
      font: inherit;
      font-size: 14px;
      transition: border-color 120ms ease, box-shadow 120ms ease;
    }
    input:focus {
      outline: none;
      border-color: var(--accent);
      box-shadow: 0 0 0 3px color-mix(in oklch, var(--accent) 22%, transparent);
    }

    .submit-row {
      display: flex;
      flex-direction: column;
      gap: 10px;
      margin-top: 4px;
    }
    .submit-row ship-button { width: 100%; }

    .terms {
      color: var(--muted-foreground);
      font-size: 11.5px;
      text-align: center;
      line-height: 1.5;
    }

    .switch {
      color: var(--muted-foreground);
      font-size: 12.5px;
      text-align: center;
    }
    .switch button {
      appearance: none;
      background: transparent;
      border: 0;
      padding: 0;
      color: var(--accent);
      font: inherit;
      font-weight: 600;
      cursor: pointer;
      text-decoration: underline;
      text-underline-offset: 3px;
    }
  `

  static properties = {
    open: { type: Boolean, reflect: true },
    mode: { type: String, reflect: true },
    loading: { type: Boolean, reflect: true },
    backdrop: { type: String, reflect: true },
  }

  open = false
  mode: AuthMode = 'sign-in'
  loading = false
  backdrop: 'default' | 'blur' | 'solid' = 'blur'

  private onTabChange = (e: Event) => {
    const detail = (e as CustomEvent<{ name: string }>).detail
    if (!detail) return
    const next = detail.name === 'sign-up' ? 'sign-up' : 'sign-in'
    if (next === this.mode) return
    this.mode = next
    this.dispatchEvent(new CustomEvent('ship-mode-change', {
      detail: { mode: next }, bubbles: true, composed: true,
    }))
  }

  private onSubmit = (e: Event) => {
    e.preventDefault()
    const form = e.currentTarget as HTMLFormElement
    const email = (form.elements.namedItem('email') as HTMLInputElement | null)?.value ?? ''
    const password = (form.elements.namedItem('password') as HTMLInputElement | null)?.value ?? ''
    this.dispatchEvent(new CustomEvent('ship-auth-submit', {
      detail: { mode: this.mode, email, password },
      bubbles: true,
      composed: true,
    }))
  }

  private switchTo(next: AuthMode) {
    if (next === this.mode) return
    this.mode = next
    this.dispatchEvent(new CustomEvent('ship-mode-change', {
      detail: { mode: next }, bubbles: true, composed: true,
    }))
  }

  private onDialogClose = (e: Event) => {
    this.open = false
    this.dispatchEvent(new CustomEvent('ship-close', {
      detail: (e as CustomEvent).detail ?? null,
      bubbles: true,
      composed: true,
    }))
  }

  private defaultTitle() {
    return this.mode === 'sign-up' ? 'Create your account' : 'Welcome back'
  }

  private submitLabel() {
    if (this.loading) return this.mode === 'sign-up' ? 'Creating account…' : 'Signing in…'
    return this.mode === 'sign-up' ? 'Create account' : 'Sign in'
  }

  render() {
    const otherMode: AuthMode = this.mode === 'sign-up' ? 'sign-in' : 'sign-up'
    const otherLabel = otherMode === 'sign-up' ? 'Create an account' : 'Sign in instead'

    return html`
      <ship-dialog
        part="dialog"
        ?open=${this.open}
        size="sm"
        backdrop=${this.backdrop}
        @ship-close=${this.onDialogClose}
      >
        <div slot="header">
          <slot name="header">${this.defaultTitle()}</slot>
        </div>

        <div class="body">
          <ship-tabs
            variant="segmented"
            active=${this.mode}
            @ship-tab-change=${this.onTabChange}
          >
            <ship-tab name="sign-in" label="Sign in"></ship-tab>
            <ship-tab name="sign-up" label="Sign up"></ship-tab>
          </ship-tabs>

          <div class="providers">
            <slot name="providers"></slot>
          </div>

          ${this.hasProviderChildren() ? html`<div class="or">or</div>` : nothing}

          <form part="form" @submit=${this.onSubmit} novalidate>
            <label>
              Email
              <input
                name="email"
                type="email"
                autocomplete="email"
                inputmode="email"
                required
                placeholder="you@company.com"
              />
            </label>
            <label>
              Password
              <input
                name="password"
                type="password"
                autocomplete=${this.mode === 'sign-up' ? 'new-password' : 'current-password'}
                required
                minlength="8"
              />
            </label>

            <div class="submit-row">
              <ship-button
                part="submit"
                tone="accent"
                variant="solid"
                size="md"
                type="submit"
                ?loading=${this.loading}
                ?disabled=${this.loading}
              >${this.submitLabel()}</ship-button>
              <p class="terms"><slot name="terms"></slot></p>
            </div>
          </form>

          <p class="switch">
            ${this.mode === 'sign-up' ? 'Already have an account?' : 'New here?'}
            <button
              type="button"
              @click=${() => this.switchTo(otherMode)}
            >${otherLabel}</button>
          </p>
        </div>
      </ship-dialog>
    `
  }

  private hasProviderChildren() {
    const slot = this.renderRoot.querySelector('slot[name="providers"]') as HTMLSlotElement | null
    if (!slot) return false
    return slot.assignedNodes({ flatten: true }).some((n) => {
      if (n.nodeType === Node.TEXT_NODE) return (n.textContent || '').trim().length > 0
      return n.nodeType === Node.ELEMENT_NODE
    })
  }

  /** Imperative helpers mirroring ship-dialog. */
  show() { this.open = true }
  showModal() { this.open = true }
  close() { this.open = false }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-auth-dialog')) {
  customElements.define('ship-auth-dialog', ShipAuthDialog)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-auth-dialog': ShipAuthDialog
  }
}
