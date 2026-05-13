import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-list-tile/sui-list-tile.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-list-tile>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-list-tile')).toBeDefined()
  })

  it('renders a shadow root with .tile element', () => {
    const { el } = mount(reg, 'sui-list-tile', {}, 'Item')
    expect(shadow(el).querySelector('.tile')).not.toBeNull()
  })

  it('role=listitem by default', async () => {
    const { el } = mount(reg, 'sui-list-tile', {}, 'Item')
    await flush()
    expect(el.getAttribute('role')).toBe('listitem')
  })

  it('role=button when interactive', async () => {
    const { el } = mount(reg, 'sui-list-tile', { interactive: true }, 'Item')
    await flush()
    expect(el.getAttribute('role')).toBe('button')
  })

  it('role=link when navigable', async () => {
    const { el } = mount(reg, 'sui-list-tile', { navigable: '/x' }, 'Item')
    await flush()
    expect(el.getAttribute('role')).toBe('link')
  })

  it('link role wins when both navigable and interactive', async () => {
    const { el } = mount(reg, 'sui-list-tile', { navigable: '/x', interactive: true }, 'Item')
    await flush()
    expect(el.getAttribute('role')).toBe('link')
  })

  it('reflects selected to aria-selected', async () => {
    const { el } = mount(reg, 'sui-list-tile', { selected: true }, 'Item')
    await flush()
    expect(el.getAttribute('aria-selected')).toBe('true')
  })

  it('aria-selected=false when not selected', async () => {
    const { el } = mount(reg, 'sui-list-tile', {}, 'Item')
    await flush()
    expect(el.getAttribute('aria-selected')).toBe('false')
  })

  it('reflects disabled to aria-disabled', async () => {
    const { el } = mount(reg, 'sui-list-tile', { disabled: true }, 'Item')
    await flush()
    expect(el.getAttribute('aria-disabled')).toBe('true')
  })

  it('interactive tile gets tabindex=0', async () => {
    const { el } = mount(reg, 'sui-list-tile', { interactive: true }, 'Item')
    await flush()
    const tile = shadow(el).querySelector('.tile') as HTMLElement
    expect(tile.getAttribute('tabindex')).toBe('0')
  })

  it('non-interactive tile has no tabindex', async () => {
    const { el } = mount(reg, 'sui-list-tile', {}, 'Item')
    await flush()
    const tile = shadow(el).querySelector('.tile') as HTMLElement
    expect(tile.getAttribute('tabindex')).toBeNull()
  })

  it('dispatches sui-select on click when interactive', async () => {
    const { el } = mount(reg, 'sui-list-tile', { interactive: true }, 'Item')
    await flush()
    let fired = 0
    el.addEventListener('sui-select', () => fired++)
    const tile = shadow(el).querySelector('.tile') as HTMLElement
    tile.click()
    expect(fired).toBe(1)
  })

  it('does not dispatch sui-select when not interactive', async () => {
    const { el } = mount(reg, 'sui-list-tile', {}, 'Item')
    await flush()
    let fired = 0
    el.addEventListener('sui-select', () => fired++)
    const tile = shadow(el).querySelector('.tile') as HTMLElement
    tile.click()
    expect(fired).toBe(0)
  })

  it('dispatches sui-select on Enter key when interactive', async () => {
    const { el } = mount(reg, 'sui-list-tile', { interactive: true }, 'Item')
    await flush()
    let fired = 0
    el.addEventListener('sui-select', () => fired++)
    const tile = shadow(el).querySelector('.tile') as HTMLElement
    tile.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(fired).toBe(1)
  })

  it('dispatches sui-select on Space key when interactive', async () => {
    const { el } = mount(reg, 'sui-list-tile', { interactive: true }, 'Item')
    await flush()
    let fired = 0
    el.addEventListener('sui-select', () => fired++)
    const tile = shadow(el).querySelector('.tile') as HTMLElement
    tile.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    expect(fired).toBe(1)
  })

  it('disabled interactive tile does not dispatch sui-select', async () => {
    const { el } = mount(reg, 'sui-list-tile', { interactive: true, disabled: true }, 'Item')
    await flush()
    let fired = 0
    el.addEventListener('sui-select', () => fired++)
    const tile = shadow(el).querySelector('.tile') as HTMLElement
    tile.click()
    tile.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(fired).toBe(0)
  })

  it('variant and density reflect as attributes', () => {
    const { el } = mount(reg, 'sui-list-tile', { variant: 'card', density: 'compact' }, 'Item')
    expect(el.getAttribute('variant')).toBe('card')
    expect(el.getAttribute('density')).toBe('compact')
  })

  describe('navigable', () => {
    it('renders <a> element when navigable is set', async () => {
      const { el } = mount(reg, 'sui-list-tile', { navigable: '/path' }, 'Nav')
      await flush()
      const a = shadow(el).querySelector('a.tile') as HTMLAnchorElement
      expect(a).not.toBeNull()
      expect(a.getAttribute('href')).toBe('/path')
    })

    it('renders <div> when navigable is not set', async () => {
      const { el } = mount(reg, 'sui-list-tile', {}, 'Item')
      await flush()
      expect(shadow(el).querySelector('div.tile')).not.toBeNull()
      expect(shadow(el).querySelector('a.tile')).toBeNull()
    })

    it('passes target and rel to the anchor', async () => {
      const { el } = mount(reg, 'sui-list-tile', {
        navigable: '/x', target: '_blank', rel: 'noopener',
      }, 'X')
      await flush()
      const a = shadow(el).querySelector('a.tile') as HTMLAnchorElement
      expect(a.getAttribute('target')).toBe('_blank')
      expect(a.getAttribute('rel')).toBe('noopener')
    })

    it('does not dispatch sui-select on plain navigable click (not interactive)', async () => {
      const { el } = mount(reg, 'sui-list-tile', { navigable: '/x' }, 'X')
      await flush()
      let fired = 0
      el.addEventListener('sui-select', () => fired++)
      const a = shadow(el).querySelector('a.tile') as HTMLAnchorElement
      a.click()
      expect(fired).toBe(0)
    })

    it('dispatches sui-select when navigable + interactive on click', async () => {
      const { el } = mount(reg, 'sui-list-tile', { navigable: '/x', interactive: true }, 'X')
      await flush()
      let fired = 0
      el.addEventListener('sui-select', () => fired++)
      const a = shadow(el).querySelector('a.tile') as HTMLAnchorElement
      a.click()
      expect(fired).toBe(1)
    })

    it('preventDefault on sui-select blocks anchor navigation', async () => {
      const { el } = mount(reg, 'sui-list-tile', { navigable: '/x', interactive: true }, 'X')
      await flush()
      el.addEventListener('sui-select', e => e.preventDefault())
      const a = shadow(el).querySelector('a.tile') as HTMLAnchorElement
      const click = new MouseEvent('click', { bubbles: true, cancelable: true })
      a.dispatchEvent(click)
      expect(click.defaultPrevented).toBe(true)
    })

    it('Enter on navigable skips sui-select (anchor handles natively)', async () => {
      const { el } = mount(reg, 'sui-list-tile', { navigable: '/x', interactive: true }, 'X')
      await flush()
      let fired = 0
      el.addEventListener('sui-select', () => fired++)
      const a = shadow(el).querySelector('a.tile') as HTMLAnchorElement
      a.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
      expect(fired).toBe(0)
    })

    it('Space on navigable + interactive fires sui-select', async () => {
      const { el } = mount(reg, 'sui-list-tile', { navigable: '/x', interactive: true }, 'X')
      await flush()
      let fired = 0
      el.addEventListener('sui-select', () => fired++)
      const a = shadow(el).querySelector('a.tile') as HTMLAnchorElement
      a.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
      expect(fired).toBe(1)
    })
  })

  describe('divider adopted stylesheet', () => {
    it('installs adopted stylesheet or fallback style on first connect', async () => {
      const { el } = mount(reg, 'sui-list-tile', { variant: 'row' }, 'A')
      await flush()
      // JSDOM may or may not support adoptedStyleSheets; check both paths
      const sheets = document.adoptedStyleSheets ?? []
      const hasDividerSheet = sheets.some((s: CSSStyleSheet) => {
        try { return [...s.cssRules].some(r => r.cssText.includes('sui-list-tile')) } catch { return false }
      })
      const hasFallback = !!document.querySelector('style[data-sui-list-tile-dividers]')
      expect(hasDividerSheet || hasFallback).toBe(true)
      el.remove()
    })
  })
})
