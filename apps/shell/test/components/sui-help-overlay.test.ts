import { afterEach, describe, expect, it } from 'vitest'
import '../../src/components/sui-help-overlay/sui-help-overlay.compiled.js'
import { createContainerRegistry, flush, mount } from '../test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

interface HelpEl extends HTMLElement {
  show(): void
  hide(): void
}

describe('<sui-help-overlay>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-help-overlay')).toBeDefined()
  })

  it('renders a dialog headed "Keyboard shortcuts"', async () => {
    const { el } = mount(reg, 'sui-help-overlay')
    await flush()
    const dialog = shadow(el).querySelector('dialog')!
    expect(dialog).not.toBeNull()
    expect(dialog.getAttribute('aria-modal')).toBe('true')
    expect(shadow(el).querySelector('[part~="title"]')!.textContent).toContain('Keyboard shortcuts')
  })

  it('renders the Global and Editing groups with at least one row each', async () => {
    const { el } = mount(reg, 'sui-help-overlay')
    await flush()
    const groups = [...shadow(el).querySelectorAll('section.group')]
    const labels = groups.map((g) => g.querySelector('.group-h')!.textContent)
    expect(labels).toContain('Global')
    expect(labels).toContain('Editing')
    for (const g of groups) {
      expect(g.querySelectorAll('.row').length).toBeGreaterThan(0)
    }
  })

  it('lists ⌘K → Open command palette and Esc → return to canvas', async () => {
    const { el } = mount(reg, 'sui-help-overlay')
    await flush()
    const rows = [...shadow(el).querySelectorAll('.row')].map((r) => ({
      // Native <kbd class="kbd"> since d04fdd672 cut the sui-kbd primitive.
      keys: [...r.querySelectorAll('kbd.kbd')].map((k) => k.textContent).join(''),
      desc: r.querySelector('.desc')!.textContent,
    }))
    expect(rows).toContainEqual({ keys: '⌘K', desc: 'Open command palette' })
    expect(rows).toContainEqual({ keys: 'Esc', desc: 'Return to canvas / close overlay' })
  })

  it('renders chord keys joined by + separators', async () => {
    const { el } = mount(reg, 'sui-help-overlay')
    await flush()
    // ⇧⌥E is a three-key chord → two "+" separators between three kbds.
    const chord = [...shadow(el).querySelectorAll('.row')].find(
      (r) => r.querySelector('.desc')!.textContent === 'Toggle annotation mode',
    )!
    expect(chord.querySelectorAll('kbd.kbd').length).toBe(3)
    expect(chord.querySelectorAll('.plus').length).toBe(2)
  })

  it('opens via the reflected open attribute / show(), closes via hide()', async () => {
    const { el } = mount(reg, 'sui-help-overlay')
    await flush()
    const help = el as HelpEl
    help.show()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)

    let closed = false
    el.addEventListener('sui-help-close', () => { closed = true })
    help.hide()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
    expect(closed).toBe(true)
  })

  it('the header × button closes and emits sui-help-close', async () => {
    const { el } = mount(reg, 'sui-help-overlay')
    await flush()
    ;(el as HelpEl).show()
    await flush()
    let closed = false
    el.addEventListener('sui-help-close', () => { closed = true })
    shadow(el).querySelector<HTMLButtonElement>('[part~="close"]')!.click()
    await flush()
    expect(closed).toBe(true)
    expect(el.hasAttribute('open')).toBe(false)
  })
})
