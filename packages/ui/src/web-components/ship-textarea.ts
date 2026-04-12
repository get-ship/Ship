/**
 * <ship-textarea>
 *
 * Wraps a native `<textarea>`. Use `variant="code"` for monospace; resize
 * controlled via `resize` attribute.
 *
 * Attributes:
 *   - value, placeholder, name, required, disabled, readonly
 *   - rows: number (default 4)
 *   - variant: default | code (default: default)
 *   - size: sm | md (default: md)
 *   - resize: none | vertical | horizontal | both (default: vertical)
 *
 * Parts:
 *   - textarea — the wrapped native element
 *
 * Events:
 *   - ship-input — CustomEvent { detail: { value } }
 */

import { LitElement, css, html } from 'lit'
import { MONO } from './util'

export class ShipTextarea extends LitElement {
  static styles = css`
    :host { display: block; }
    textarea {
      width: 100%;
      box-sizing: border-box;
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 12px;
      background: var(--background);
      color: var(--foreground);
      font: inherit;
      font-family: var(--font-body);
      line-height: 1.5;
    }
    textarea:focus {
      outline: 2px solid var(--primary);
      outline-offset: -1px;
      border-color: transparent;
    }
    textarea::placeholder { color: var(--muted-foreground); }
    textarea:disabled { cursor: not-allowed; opacity: 0.6; }

    :host([variant='code']) textarea {
      font-family: ${MONO};
      font-size: 13px;
    }
    :host([size='sm']) textarea {
      padding: 4px 8px;
      font-size: 12px;
    }
  `

  static properties = {
    value: { type: String },
    placeholder: { type: String },
    name: { type: String },
    rows: { type: Number },
    required: { type: Boolean, reflect: true },
    disabled: { type: Boolean, reflect: true },
    readonly: { type: Boolean, reflect: true },
    variant: { type: String, reflect: true },
    size: { type: String, reflect: true },
    resize: { type: String },
  }

  value = ''
  placeholder = ''
  name = ''
  rows = 4
  required = false
  disabled = false
  readonly = false
  variant: 'default' | 'code' = 'default'
  size: 'sm' | 'md' = 'md'
  resize: 'none' | 'vertical' | 'horizontal' | 'both' = 'vertical'
  private onInput(e: Event) {
    this.value = (e.target as HTMLTextAreaElement).value
    this.dispatchEvent(new CustomEvent('ship-input', {
      detail: { value: this.value }, bubbles: true, composed: true,
    }))
  }

  render() {
    return html`
      <textarea
        part="textarea"
        .value=${this.value}
        placeholder=${this.placeholder}
        name=${this.name}
        rows=${this.rows}
        ?required=${this.required}
        ?disabled=${this.disabled}
        ?readonly=${this.readonly}
        style="resize: ${this.resize}"
        @input=${this.onInput}
      ></textarea>
    `
  }
}


if (typeof customElements !== 'undefined' && !customElements.get('ship-textarea')) {
  customElements.define('ship-textarea', ShipTextarea)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-textarea': ShipTextarea
  }
}
