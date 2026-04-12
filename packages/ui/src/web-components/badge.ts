import { TOKENS, FONT_STACK, defineOnce } from './tokens'

type Variant = 'default' | 'success' | 'warning' | 'destructive' | 'info' | 'muted'
type Size = 'sm' | 'md'

const VARIANT_COLORS: Record<Variant, { bg: string; fg: string; border: string }> = {
  default: { bg: TOKENS.primary, fg: TOKENS.primaryForeground, border: 'transparent' },
  success: { bg: `color-mix(in oklch, ${TOKENS.success} 18%, transparent)`, fg: TOKENS.success, border: `color-mix(in oklch, ${TOKENS.success} 35%, transparent)` },
  warning: { bg: `color-mix(in oklch, ${TOKENS.warning} 20%, transparent)`, fg: TOKENS.warning, border: `color-mix(in oklch, ${TOKENS.warning} 40%, transparent)` },
  destructive: { bg: `color-mix(in oklch, ${TOKENS.destructive} 18%, transparent)`, fg: TOKENS.destructive, border: `color-mix(in oklch, ${TOKENS.destructive} 35%, transparent)` },
  info: { bg: `color-mix(in oklch, ${TOKENS.info} 18%, transparent)`, fg: TOKENS.info, border: `color-mix(in oklch, ${TOKENS.info} 35%, transparent)` },
  muted: { bg: TOKENS.muted, fg: TOKENS.mutedForeground, border: TOKENS.border },
}

export class ShipBadge extends HTMLElement {
  static get observedAttributes() { return ['variant', 'size'] }

  private root: ShadowRoot

  constructor() {
    super()
    this.root = this.attachShadow({ mode: 'open' })
  }

  connectedCallback() { this.render() }
  attributeChangedCallback() { if (this.root) this.render() }

  private render() {
    const variant = (this.getAttribute('variant') as Variant) || 'default'
    const size = (this.getAttribute('size') as Size) || 'md'
    const v = VARIANT_COLORS[variant] || VARIANT_COLORS.default
    const padding = size === 'sm' ? '1px 6px' : '2px 8px'
    const fontSize = size === 'sm' ? '11px' : '12px'

    this.root.innerHTML = `
      <style>
        :host { display: inline-flex; }
        .badge {
          display: inline-flex;
          align-items: center;
          gap: 4px;
          padding: ${padding};
          font-family: ${FONT_STACK};
          font-size: ${fontSize};
          font-weight: 500;
          line-height: 1.4;
          border-radius: calc(${TOKENS.radius} - 4px);
          background: ${v.bg};
          color: ${v.fg};
          border: 1px solid ${v.border};
          white-space: nowrap;
        }
      </style>
      <span class="badge"><slot></slot></span>
    `
  }
}

export function registerBadge() {
  defineOnce('ship-badge', ShipBadge)
}
