import { TOKENS, FONT_STACK, defineOnce } from './tokens'

/**
 * Icon name maps to a unicode/text glyph. Kept tiny and dependency-free;
 * consumers can slot richer icons by passing children instead of using `icon`.
 */
const ICON_GLYPHS: Record<string, string> = {
  inbox: '⌯',
  search: '⌕',
  file: '▤',
  folder: '▦',
  alert: '!',
  check: '✓',
  plus: '+',
  sparkles: '✦',
}

export class ShipEmptyState extends HTMLElement {
  static get observedAttributes() { return ['icon', 'title', 'description'] }

  private root: ShadowRoot
  constructor() { super(); this.root = this.attachShadow({ mode: 'open' }) }
  connectedCallback() { this.render() }
  attributeChangedCallback() { if (this.root) this.render() }

  private render() {
    const icon = this.getAttribute('icon')
    const title = this.getAttribute('title') || ''
    const description = this.getAttribute('description')
    const glyph = icon ? (ICON_GLYPHS[icon] || '•') : null

    this.root.innerHTML = `
      <style>
        :host { display: block; }
        .empty {
          display: flex; flex-direction: column; align-items: center;
          justify-content: center; gap: 10px;
          padding: 32px 16px;
          text-align: center;
          font-family: ${FONT_STACK};
          color: ${TOKENS.mutedForeground};
          border: 1px dashed ${TOKENS.border};
          border-radius: ${TOKENS.radius};
          background: color-mix(in oklch, ${TOKENS.muted} 40%, transparent);
        }
        .icon {
          width: 44px; height: 44px;
          display: inline-flex; align-items: center; justify-content: center;
          font-size: 22px;
          color: ${TOKENS.mutedForeground};
          background: ${TOKENS.muted};
          border-radius: 999px;
        }
        .title {
          font-size: 14px; font-weight: 600;
          color: ${TOKENS.foreground};
        }
        .desc { font-size: 13px; max-width: 42ch; line-height: 1.5; }
        .actions { margin-top: 4px; display: flex; gap: 8px; justify-content: center; }
        ::slotted([slot='action']) { cursor: pointer; }
      </style>
      <div class="empty">
        ${glyph ? `<div class="icon" aria-hidden="true">${glyph}</div>` : ''}
        ${title ? `<div class="title">${title}</div>` : ''}
        ${description ? `<div class="desc">${description}</div>` : ''}
        <div class="actions"><slot name="action"></slot></div>
      </div>
    `
  }
}

export function registerEmptyState() {
  defineOnce('ship-empty-state', ShipEmptyState)
}
