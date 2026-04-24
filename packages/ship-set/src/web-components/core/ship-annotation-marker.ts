/**
 * <ship-annotation-marker>
 *
 * Visual pin or box marker for a staged / sent annotation. No input
 * logic lives here — `<ship-annotation-layer>` places markers and
 * listens for their `click`.
 *
 * Attributes:
 *   - kind: point | region (required)
 *   - x, y: numbers — point mode (content coords, px)
 *   - rect: "x,y,w,h" — region mode
 *   - active: boolean — highlights the marker
 *   - index: number — ordinal badge inside point pins
 *
 * Parts:
 *   - pin   — the circular point marker
 *   - box   — the region rectangle
 */

import { LitElement, css, html } from 'lit'

type Kind = 'point' | 'region'

export class ShipAnnotationMarker extends LitElement {
  static styles = css`
    :host {
      position: absolute;
      pointer-events: auto;
      font-family: var(--font-body);
      font-size: var(--text-xs);
    }
    :host([hidden]) { display: none; }

    [part='pin'] {
      position: absolute;
      width: 20px;
      height: 20px;
      border-radius: 50%;
      background: var(--primary);
      color: var(--primary-foreground);
      display: inline-flex;
      align-items: center;
      justify-content: center;
      font-weight: var(--font-weight-medium);
      box-shadow: var(--shadow-sm);
      cursor: pointer;
      transform: translate(-50%, -50%);
      border: 2px solid var(--background);
      transition: filter var(--duration-fast) var(--ease-out);
    }
    :host([active]) [part='pin'] {
      filter: brightness(1.2);
      outline: 2px solid var(--ring-color);
      outline-offset: 2px;
    }

    [part='box'] {
      position: absolute;
      border: 2px dashed var(--primary);
      background: var(--tone-primary-bg);
      border-radius: var(--radius-sm);
      cursor: pointer;
      box-sizing: border-box;
    }
    :host([active]) [part='box'] {
      background: var(--tone-primary-bg);
    }
  `

  static properties = {
    kind: { type: String, reflect: true },
    x: { type: Number },
    y: { type: Number },
    rect: { type: String, reflect: true },
    active: { type: Boolean, reflect: true },
    index: { type: Number },
  }

  kind: Kind = 'point'
  x = 0
  y = 0
  rect: string | null = null
  active = false
  index = 0

  private parsedRect(): [number, number, number, number] | null {
    if (!this.rect) return null
    const parts = this.rect.split(',').map((s) => Number(s.trim()))
    if (parts.length !== 4 || parts.some((n) => !Number.isFinite(n))) return null
    return parts as [number, number, number, number]
  }

  render() {
    if (this.kind === 'region') {
      const r = this.parsedRect()
      if (!r) return null
      const [x, y, w, h] = r
      return html`<div
        part="box"
        role="button"
        tabindex="0"
        style=${`left:${x}px;top:${y}px;width:${w}px;height:${h}px`}
      ></div>`
    }
    return html`<div
      part="pin"
      role="button"
      tabindex="0"
      style=${`left:${this.x}px;top:${this.y}px`}
    >${this.index + 1}</div>`
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-annotation-marker')) {
  customElements.define('ship-annotation-marker', ShipAnnotationMarker)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-annotation-marker': ShipAnnotationMarker
  }
}
