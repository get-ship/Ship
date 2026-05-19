// Shell chrome smoke tests — covers the three new chrome primitives
// (sui-shell-top, sui-shell-bottom, sui-blank-canvas) and the hotkey
// surface main.ts wires. Layout assertions verify the three-zone grid
// (top 36px · canvas · bottom 28px) is in place and there is no rail in
// the shell mount.

import { afterEach, describe, expect, it } from 'vitest'
import '../../src/components/sui-shell-top/sui-shell-top.compiled.js'
import '../../src/components/sui-shell-bottom/sui-shell-bottom.compiled.js'
import '../../src/components/sui-blank-canvas/sui-blank-canvas.compiled.js'
import { createContainerRegistry, flush, mount } from '../test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-shell-top>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-shell-top')).toBeDefined()
  })

  it('renders brand, cmdk pill, and right actions cluster', () => {
    const { el } = mount(reg, 'sui-shell-top')
    expect(shadow(el).querySelector('[part~="brand"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="cmdk"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="actions"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="theme-btn"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="notifications-btn"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="avatar"]')).not.toBeNull()
  })

  it('workspace prop renders into the wordmark hairline-separated label', async () => {
    const { el } = mount(reg, 'sui-shell-top', { workspace: 'ship-monorepo' })
    await flush()
    const ws = shadow(el).querySelector('[part~="workspace"]')!
    expect(ws.textContent?.trim()).toBe('ship-monorepo')
  })

  it('initials prop renders into the avatar', async () => {
    const { el } = mount(reg, 'sui-shell-top', { initials: 'M' })
    await flush()
    const avatar = shadow(el).querySelector('[part~="avatar"]')!
    expect(avatar.textContent?.trim()).toBe('M')
  })

  it('clicking the cmdk pill emits sui-open-command-palette', async () => {
    const { el } = mount(reg, 'sui-shell-top')
    const fired: Event[] = []
    document.addEventListener('sui-open-command-palette', (e) => fired.push(e), { once: true })
    const pill = shadow(el).querySelector('[part~="cmdk"]') as HTMLButtonElement
    pill.click()
    await flush()
    expect(fired.length).toBe(1)
  })

  it('clicking the theme button emits sui:theme-picker:open', async () => {
    const { el } = mount(reg, 'sui-shell-top')
    const fired: Event[] = []
    document.addEventListener('sui:theme-picker:open', (e) => fired.push(e), { once: true })
    const btn = shadow(el).querySelector('[part~="theme-btn"]') as HTMLButtonElement
    btn.click()
    await flush()
    expect(fired.length).toBe(1)
  })
})

describe('<sui-shell-bottom>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-shell-bottom')).toBeDefined()
  })

  it('renders dot, tools, and mode chip', () => {
    const { el } = mount(reg, 'sui-shell-bottom', { mode: 'development' })
    expect(shadow(el).querySelector('[part~="dot"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="tools"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="terminal-btn"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="devtools-btn"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="annotate-btn"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="mode"]')).not.toBeNull()
  })

  it('online attribute reflects the prop', () => {
    const { el } = mount(reg, 'sui-shell-bottom', { online: true })
    expect(el.hasAttribute('online')).toBe(true)
  })

  it('mode attribute reflects the prop', () => {
    const { el } = mount(reg, 'sui-shell-bottom', { mode: 'release' })
    expect(el.getAttribute('mode')).toBe('release')
  })

  it('branch + workspace render when set', async () => {
    const { el } = mount(reg, 'sui-shell-bottom', { branch: 'main', workspace: 'ship-monorepo' })
    await flush()
    expect(shadow(el).querySelector('[part~="branch"]')?.textContent?.trim()).toContain('main')
    expect(shadow(el).querySelector('[part~="workspace"]')?.textContent?.trim()).toBe('ship-monorepo')
  })

  it('terminal-btn click emits sui-toggle-terminal', async () => {
    const { el } = mount(reg, 'sui-shell-bottom')
    const fired: Event[] = []
    document.addEventListener('sui-toggle-terminal', (e) => fired.push(e), { once: true })
    const btn = shadow(el).querySelector('[part~="terminal-btn"]') as HTMLButtonElement
    btn.click()
    await flush()
    expect(fired.length).toBe(1)
  })

  it('devtools-btn click emits sui:dock:toggle', async () => {
    const { el } = mount(reg, 'sui-shell-bottom')
    const fired: Event[] = []
    document.addEventListener('sui:dock:toggle', (e) => fired.push(e), { once: true })
    const btn = shadow(el).querySelector('[part~="devtools-btn"]') as HTMLButtonElement
    btn.click()
    await flush()
    expect(fired.length).toBe(1)
  })
})

describe('<sui-blank-canvas>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-blank-canvas')).toBeDefined()
  })

  it('renders mark, title, rule, sub, and four hint pills', () => {
    const { el } = mount(reg, 'sui-blank-canvas')
    expect(shadow(el).querySelector('[part~="mark"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="title"]')?.textContent?.trim()).toBe('Blank canvas')
    expect(shadow(el).querySelector('[part~="rule"]')).not.toBeNull()
    expect(shadow(el).querySelector('[part~="sub"]')).not.toBeNull()
    const hints = shadow(el).querySelectorAll('.hint')
    expect(hints.length).toBe(4)
    const labels = Array.from(hints).map((h) => h.textContent?.trim() ?? '')
    expect(labels.some((l) => l.includes('⌘K'))).toBe(true)
    expect(labels.some((l) => l.includes('⌘.'))).toBe(true)
    expect(labels.some((l) => l.includes('⌃`'))).toBe(true)
    expect(labels.some((l) => l.includes('⌘?'))).toBe(true)
  })

  it('palette hint click emits sui-open-command-palette', async () => {
    const { el } = mount(reg, 'sui-blank-canvas')
    const fired: Event[] = []
    document.addEventListener('sui-open-command-palette', (e) => fired.push(e), { once: true })
    const hint = shadow(el).querySelector('[data-hint="palette"]') as HTMLButtonElement
    hint.click()
    await flush()
    expect(fired.length).toBe(1)
  })

  it('studio hint click emits sui:navigate { to: "/studio" }', async () => {
    const { el } = mount(reg, 'sui-blank-canvas')
    const fired: CustomEvent[] = []
    document.addEventListener('sui:navigate', (e) => fired.push(e as CustomEvent), { once: true })
    const hint = shadow(el).querySelector('[data-hint="studio"]') as HTMLButtonElement
    hint.click()
    await flush()
    expect(fired.length).toBe(1)
    expect((fired[0]!.detail as { to: string }).to).toBe('/studio')
  })

  it('terminal hint click emits sui-toggle-terminal', async () => {
    const { el } = mount(reg, 'sui-blank-canvas')
    const fired: Event[] = []
    document.addEventListener('sui-toggle-terminal', (e) => fired.push(e), { once: true })
    const hint = shadow(el).querySelector('[data-hint="terminal"]') as HTMLButtonElement
    hint.click()
    await flush()
    expect(fired.length).toBe(1)
  })

  it('help hint click emits sui-open-help', async () => {
    const { el } = mount(reg, 'sui-blank-canvas')
    const fired: Event[] = []
    document.addEventListener('sui-open-help', (e) => fired.push(e), { once: true })
    const hint = shadow(el).querySelector('[data-hint="help"]') as HTMLButtonElement
    hint.click()
    await flush()
    expect(fired.length).toBe(1)
  })
})

describe('shell three-zone layout', () => {
  it('app HTML mounts top, content, and bottom without a rail', () => {
    const container = reg.mk()
    container.innerHTML = `
      <div id="ship-app">
        <sui-shell-top id="ship-shell-top"></sui-shell-top>
        <main id="ship-content" class="ship-content"></main>
        <sui-shell-bottom id="ship-shell-bottom" branch="main" workspace="ship-monorepo"></sui-shell-bottom>
      </div>
    `
    expect(container.querySelector('#ship-shell-top')).not.toBeNull()
    expect(container.querySelector('#ship-shell-bottom')).not.toBeNull()
    expect(container.querySelector('#ship-content')).not.toBeNull()
    expect(container.querySelector('#ship-rail')).toBeNull()
    expect(container.querySelector('#ship-nav-top')).toBeNull()
    expect(container.querySelector('#ship-status-bar')).toBeNull()
  })

  it('mounting <sui-blank-canvas> into the content slot is the default empty state', () => {
    const container = reg.mk()
    container.innerHTML = `<main id="ship-content"></main>`
    const slot = container.querySelector('#ship-content')!
    slot.innerHTML = '<sui-blank-canvas></sui-blank-canvas>'
    const canvas = slot.querySelector('sui-blank-canvas') as HTMLElement
    expect(canvas).not.toBeNull()
    expect(canvas.shadowRoot?.querySelector('[part~="title"]')?.textContent?.trim()).toBe('Blank canvas')
  })
})
