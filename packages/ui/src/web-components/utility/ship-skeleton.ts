/**
 * <ship-skeleton>
 *
 * Animated shimmer placeholder for loading states.
 *
 * Attributes:
 *   - variant: text | circle | rect (default text)
 *   - width: CSS width value
 *   - height: CSS height value
 *   - lines: integer — number of text lines (variant=text only, default 1)
 */

import { LitElement, css, html } from 'lit'

export type SkeletonVariant = 'text' | 'circle' | 'rect'

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
      border-radius: var(--radius-sm);
    }
    :host([variant='circle']) .bar { border-radius: var(--radius-full); }
    :host([variant='rect']) .bar { border-radius: var(--radius-md); }
    :host([variant='text']) .bar { height: 0.9em; }
    :host([variant='text']) .bar + .bar { margin-top: 6px; }
    :host([variant='text']) .bar:last-child { width: 65%; }
    :host([variant='text']) .bar:only-child { width: 100%; }
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
  }

  variant: SkeletonVariant = 'text'
  width = ''
  height = ''
  lines = 1

  connectedCallback() {
    super.connectedCallback()
    if (!this.hasAttribute('role')) this.setAttribute('role', 'status')
    if (!this.hasAttribute('aria-live')) this.setAttribute('aria-live', 'polite')
    if (!this.hasAttribute('aria-busy')) this.setAttribute('aria-busy', 'true')
    if (!this.hasAttribute('aria-label')) this.setAttribute('aria-label', 'Loading')
  }

  private styleFor(): string {
    const parts: string[] = []
    if (this.width) parts.push(`width:${this.width}`)
    if (this.height) parts.push(`height:${this.height}`)
    return parts.join(';')
  }

  render() {
    const count = this.variant === 'text' ? Math.max(1, this.lines || 1) : 1
    const s = this.styleFor()
    return html`${Array.from({ length: count }).map(() => html`
      <span class="bar" part="bar" style=${s}></span>
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
