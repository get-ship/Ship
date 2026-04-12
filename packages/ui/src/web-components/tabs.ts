import { TOKENS, FONT_STACK, defineOnce } from './tokens'

export class ShipTabs extends HTMLElement {
  static get observedAttributes() { return ['active'] }

  private root: ShadowRoot
  constructor() { super(); this.root = this.attachShadow({ mode: 'open' }) }

  connectedCallback() {
    this.render()
    this.syncChildren()
  }

  attributeChangedCallback() {
    if (this.root) {
      this.render()
      this.syncChildren()
    }
  }

  private get tabs(): ShipTab[] {
    return Array.from(this.querySelectorAll(':scope > ship-tab')) as ShipTab[]
  }

  private syncChildren() {
    const active = this.getAttribute('active')
    for (const tab of this.tabs) {
      const isActive = tab.getAttribute('name') === active
      tab.toggleAttribute('active', isActive)
    }
  }

  private handleClick(name: string) {
    this.setAttribute('active', name)
    this.dispatchEvent(new CustomEvent('ship-tab-change', {
      detail: { name }, bubbles: true, composed: true,
    }))
  }

  private render() {
    const active = this.getAttribute('active')
    const tabs = this.tabs.map(t => ({
      name: t.getAttribute('name') || '',
      label: t.getAttribute('label') || t.getAttribute('name') || '',
    }))

    this.root.innerHTML = `
      <style>
        :host { display: block; font-family: ${FONT_STACK}; }
        .tablist {
          display: flex; gap: 2px;
          border-bottom: 1px solid ${TOKENS.border};
          margin-bottom: 12px;
        }
        button {
          font-family: inherit;
          font-size: 13px; font-weight: 500;
          padding: 8px 14px;
          background: transparent;
          color: ${TOKENS.mutedForeground};
          border: none;
          border-bottom: 2px solid transparent;
          margin-bottom: -1px;
          cursor: pointer;
          transition: color 120ms ease, border-color 120ms ease;
        }
        button:hover { color: ${TOKENS.foreground}; }
        button[aria-selected='true'] {
          color: ${TOKENS.foreground};
          border-bottom-color: ${TOKENS.primary};
        }
        button:focus-visible {
          outline: 2px solid ${TOKENS.primary};
          outline-offset: -2px;
          border-radius: 4px;
        }
        .panels { display: block; }
      </style>
      <div class="tablist" role="tablist">
        ${tabs.map(t => `
          <button role="tab"
            data-name="${t.name}"
            aria-selected="${t.name === active}"
            tabindex="${t.name === active ? '0' : '-1'}"
          >${t.label}</button>
        `).join('')}
      </div>
      <div class="panels"><slot></slot></div>
    `

    this.root.querySelectorAll('button[role=tab]').forEach(btn => {
      btn.addEventListener('click', () => {
        const name = (btn as HTMLButtonElement).dataset.name || ''
        this.handleClick(name)
      })
      btn.addEventListener('keydown', (e) => {
        const ke = e as KeyboardEvent
        if (ke.key !== 'ArrowRight' && ke.key !== 'ArrowLeft') return
        ke.preventDefault()
        const names = tabs.map(t => t.name)
        const current = names.indexOf(this.getAttribute('active') || '')
        const next = ke.key === 'ArrowRight'
          ? (current + 1) % names.length
          : (current - 1 + names.length) % names.length
        this.handleClick(names[next])
      })
    })
  }
}

export class ShipTab extends HTMLElement {
  static get observedAttributes() { return ['active'] }

  private root: ShadowRoot
  constructor() { super(); this.root = this.attachShadow({ mode: 'open' }) }
  connectedCallback() { this.render() }
  attributeChangedCallback() { if (this.root) this.render() }

  private render() {
    const active = this.hasAttribute('active')
    this.setAttribute('role', 'tabpanel')
    this.root.innerHTML = `
      <style>
        :host { display: ${active ? 'block' : 'none'}; }
      </style>
      <slot></slot>
    `
  }
}

export function registerTabs() {
  defineOnce('ship-tab', ShipTab)
  defineOnce('ship-tabs', ShipTabs)
}
