/**
 * <ship-skeleton>
 *
 * Shimmer placeholder for loading states.
 *
 * Attributes:
 *   - variant: text | circle | rect (default text)
 *   - width: CSS width value (default "100%")
 *   - height: CSS height value (text=16px, circle=40px, rect=100px)
 *   - lines: number of text lines (variant=text only, default 1)
 *   - animate: boolean, default true — set to false to disable shimmer
 */

import { LitElement, css, html } from 'lit'

export type SkeletonVariant = 'text' | 'circle' | 'rect'

const HEIGHTS: Record<SkeletonVariant, string> = {
  text: '16px',
  circle: '40px',
  rect: '100px',
}

const LINE_WIDTHS = ['100%', '80%', '60%']

export class ShipSkeleton extends LitElement {
  static styles = css`
    :host { display: block; }
    .bar {
      display: block;
      background: linear-gradient(
        90deg,
        var(--muted) 0%,
        color-mix(in oklch, var(--muted-foreground) 14%, var(--muted)) 50%,
        var(--muted) 100%
      );
      background-size: 200% 100%;
      animation: shimmer 1.8s ease-in-out infinite;
      border-radius: 9999px;
    }
    :host(:not([animate])) .bar,
    :host([animate='false']) .bar { animation: none; }
    :host([variant='circle']) .bar { border-radius: var(--radius-full); }
    :host([variant='rect']) .bar { border-radius: var(--radius-md); }
    :host([variant='text']) .bar + .bar { margin-top: 6px; }
    @keyframes shimmer {
      0% { background-position: 200% 0; }
      100% { background-position: -200% 0; }
    }
    @media (prefers-reduced-motion: reduce) {
      .bar { animation: none; }
    }
  `

  static properties = {
    variant: { type: String, reflect: true },
    width: { type: String },
    height: { type: String },
    lines: { type: Number },
    animated: { type: Boolean, attribute: 'animate', reflect: true },
  }

  variant: SkeletonVariant = 'text'
  width = ''
  height = ''
  lines = 1
  animated = true

  connectedCallback() {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'status')
    if (!this.hasAttribute('aria-live')) this.setAttribute('aria-live', 'polite')
    if (!this.hasAttribute('aria-busy')) this.setAttribute('aria-busy', 'true')
    if (!this.hasAttribute('aria-label')) this.setAttribute('aria-label', 'Loading')
  }

  private barStyle(index: number, count: number): string {
    const parts: string[] = []
    const w = this.width || (this.variant === 'text' && count > 1
      ? LINE_WIDTHS[Math.min(index, LINE_WIDTHS.length - 1)]
      : '100%')
    parts.push(`width:${w}`)
    parts.push(`height:${this.height || HEIGHTS[this.variant]}`)
    return parts.join(';')
  }

  render() {
    const count = this.variant === 'text' ? Math.max(1, this.lines || 1) : 1
    return html`${Array.from({ length: count }, (_, i) => html`
      <span class="bar" part="bar" style=${this.barStyle(i, count)}></span>
    `)}`
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-skeleton')) {
  customElements.define('ship-skeleton', ShipSkeleton)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-skeleton': ShipSkeleton
  }
}
