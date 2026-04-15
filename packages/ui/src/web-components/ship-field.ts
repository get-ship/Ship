/**
 * <ship-field>
 *
 * Vertical stack for a label + control + optional hint/error. Matches the
 * specs view `.meta-section` layout: uppercase micro-label above, tight
 * stacked control(s) below.
 *
 * Slots:
 *   - label — label element (e.g. <ship-label>)
 *   - hint — muted helper text under the control
 *   - error — destructive-toned error text (replaces hint when invalid)
 *   - (default) the control
 *
 * Attributes:
 *   - invalid: boolean — reveals the error slot
 */

import { LitElement, css, html } from 'lit'
export class ShipField extends LitElement {
  static styles = css`
    :host { display: flex; flex-direction: column; gap: 6px; }
    .hint, .error { font-size: 11px; line-height: 1.4; }
    .hint { color: var(--muted-foreground); }
    .error { color: var(--destructive); }
    :host(:not([invalid])) .error { display: none; }
    :host([invalid]) .hint { display: none; }
  `

  static properties = {
    invalid: { type: Boolean, reflect: true },
  }

  invalid = false
  private static _idSeq = 0
  private uid = ++ShipField._idSeq
  private hintId = `ship-field-hint-${this.uid}`
  private errorId = `ship-field-error-${this.uid}`

  private wireControl = () => {
    const slot = this.renderRoot.querySelector('slot:not([name])') as HTMLSlotElement | null
    const first = slot?.assignedElements({ flatten: true })[0] as HTMLElement | undefined
    if (!first) return
    const ids: string[] = []
    if (this.invalid) ids.push(this.errorId)
    else ids.push(this.hintId)
    const prior = (first.getAttribute('aria-describedby') || '')
      .split(/\s+/)
      .filter(id => id && id !== this.hintId && id !== this.errorId)
    const final = [...prior, ...ids].join(' ').trim()
    if (final) first.setAttribute('aria-describedby', final)
    if ('invalid' in first) (first as unknown as { invalid: boolean }).invalid = this.invalid
    else first.setAttribute('aria-invalid', this.invalid ? 'true' : 'false')
  }

  updated() { this.wireControl() }

  render() {
    return html`
      <slot name="label"></slot>
      <slot @slotchange=${this.wireControl}></slot>
      <div id=${this.hintId} class="hint"><slot name="hint"></slot></div>
      <div id=${this.errorId} class="error" role="alert" aria-live="polite"><slot name="error"></slot></div>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-field')) {
  customElements.define('ship-field', ShipField)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-field': ShipField
  }
}
