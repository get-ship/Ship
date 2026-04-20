/**
 * <ship-stat>
 *
 * Single metric display — label on top, big value beneath, optional
 * unit and trailing slot for a chip/badge. Pairs of these line up
 * into a stats strip in dashboards and focus views.
 *
 * Attributes:
 *   - label: the metric name (always rendered uppercase via CSS).
 *   - value: the primary number or text. Use the default slot for
 *     rich content (e.g. a <ship-timestamp>).
 *   - unit: optional suffix rendered smaller (e.g. "%", "ms").
 *   - size: sm | md | lg (default: md).
 *
 * Slots:
 *   - (default) rich value content (alternative to `value` attr)
 *   - trailing: chip/badge rendered next to the value
 *
 * Parts:
 *   - label — the label span
 *   - value — the value row
 */

import { LitElement, css, html } from 'lit'
import { defineOnce } from '../util'

export class ShipStat extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      gap: 4px;
      padding: 14px 16px;
      border-radius: var(--radius-lg);
      border: 1px solid color-mix(in oklch, var(--border) 45%, transparent);
      background: color-mix(in oklch, var(--card) 35%, transparent);
    }
    :host([size='sm']) { padding: 10px 12px; gap: 2px; }
    :host([size='lg']) { padding: 18px 20px; gap: 6px; }

    [part='label'] {
      font-size: 10px;
      color: var(--muted-foreground);
      text-transform: uppercase;
      letter-spacing: 0.06em;
      font-weight: var(--font-weight-medium);
    }
    [part='value'] {
      display: flex;
      align-items: baseline;
      gap: 6px;
      font-variant-numeric: tabular-nums;
      color: var(--foreground);
      letter-spacing: -0.01em;
      line-height: 1.1;
    }
    .num {
      font-size: var(--text-xl);
      font-weight: var(--font-weight-semibold);
    }
    :host([size='sm']) .num { font-size: var(--text-base); }
    :host([size='lg']) .num { font-size: var(--text-2xl); }
    .unit {
      font-size: var(--text-xs);
      font-weight: var(--font-weight-medium);
      color: var(--muted-foreground);
    }
    .trailing {
      margin-left: auto;
      display: inline-flex;
      align-items: center;
    }
  `

  static properties = {
    label: { type: String, reflect: true },
    value: { type: String, reflect: true },
    unit: { type: String, reflect: true },
    size: { type: String, reflect: true },
  }

  label = ''
  value = ''
  unit = ''
  size: 'sm' | 'md' | 'lg' = 'md'

  render() {
    const hasSlottedValue = this._hasSlottedDefault()
    return html`
      <span part="label">${this.label}</span>
      <div part="value">
        ${this.value && !hasSlottedValue ? html`<span class="num">${this.value}</span>` : null}
        ${this.unit ? html`<span class="unit">${this.unit}</span>` : null}
        <slot></slot>
        <span class="trailing"><slot name="trailing"></slot></span>
      </div>
    `
  }

  private _hasSlottedDefault(): boolean {
    const slot = this.renderRoot.querySelector('slot:not([name])') as HTMLSlotElement | null
    if (!slot) return false
    return slot.assignedNodes({ flatten: true }).length > 0
  }
}

defineOnce('ship-stat', ShipStat)
