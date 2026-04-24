import { afterEach, describe, expect, it } from 'vitest'
import './ship-list-tile'
import { ShipListTile } from './ship-list-tile'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipListTile
  await el.updateComplete
  return el
}

describe('ship-list-tile', () => {
  afterEach(cleanupTestDom)

  it('renders as role=listitem by default', async () => {
    const el = await mount('<ship-list-tile>Item</ship-list-tile>')
    expect(el.getAttribute('role')).toBe('listitem')
    expect(el.shadowRoot!.querySelector('.tile')).toBeTruthy()
  })

  it('reflects selected to aria-selected', async () => {
    const el = await mount('<ship-list-tile selected>Item</ship-list-tile>')
    expect(el.getAttribute('aria-selected')).toBe('true')
  })

  it('sets role=button when interactive', async () => {
    const el = await mount('<ship-list-tile interactive>Item</ship-list-tile>')
    expect(el.getAttribute('role')).toBe('button')
    const inner = el.shadowRoot!.querySelector('.tile') as HTMLElement
    expect(inner.getAttribute('tabindex')).toBe('0')
  })

  it('dispatches ship-select on click when interactive', async () => {
    const el = await mount('<ship-list-tile interactive>Item</ship-list-tile>')
    let fired = 0
    el.addEventListener('ship-select', () => fired++)
    ;(el.shadowRoot!.querySelector('.tile') as HTMLElement).click()
    expect(fired).toBe(1)
  })

  it('does not dispatch ship-select when not interactive', async () => {
    const el = await mount('<ship-list-tile>Item</ship-list-tile>')
    let fired = 0
    el.addEventListener('ship-select', () => fired++)
    ;(el.shadowRoot!.querySelector('.tile') as HTMLElement).click()
    expect(fired).toBe(0)
  })

  it('dispatches ship-select on Enter and Space keys', async () => {
    const el = await mount('<ship-list-tile interactive>Item</ship-list-tile>')
    let fired = 0
    el.addEventListener('ship-select', () => fired++)
    const inner = el.shadowRoot!.querySelector('.tile') as HTMLElement
    inner.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    inner.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }))
    expect(fired).toBe(2)
  })

  it('blocks ship-select dispatch when disabled', async () => {
    const el = await mount('<ship-list-tile interactive disabled>Item</ship-list-tile>')
    let fired = 0
    el.addEventListener('ship-select', () => fired++)
    ;(el.shadowRoot!.querySelector('.tile') as HTMLElement).click()
    const inner = el.shadowRoot!.querySelector('.tile') as HTMLElement
    inner.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect(fired).toBe(0)
    expect(el.getAttribute('aria-disabled')).toBe('true')
  })

  it('reflects variant and density attributes', async () => {
    const el = await mount('<ship-list-tile variant="card" density="compact">Item</ship-list-tile>')
    expect(el.getAttribute('variant')).toBe('card')
    expect(el.getAttribute('density')).toBe('compact')
  })

  it('variant="card" carries the card visual treatment rules in shadow styles', async () => {
    const el = await mount('<ship-list-tile variant="card">Item</ship-list-tile>')
    // The tile root carries the .tile class hook the CSS selectors target.
    const tile = el.shadowRoot!.querySelector('.tile') as HTMLElement
    expect(tile).toBeTruthy()
    expect(el.getAttribute('variant')).toBe('card')
    // jsdom doesn't resolve color-mix or adoptedStyleSheets reliably, so
    // we inspect the element's static styles directly. The card-variant
    // rules introduced by the polish pass must remain wired in: softer
    // default border, hover lift via translateY, stronger hover border.
    const staticStyles = ShipListTile.styles as unknown as Array<{ cssText: string }>
    const css = staticStyles.map((s) => s.cssText || String(s)).join('\n')
    expect(css).toContain(`[variant='card']`)
    expect(css).toContain('translateY(-1px)')
    expect(css).toContain('color-mix')
  })

  describe('navigable', () => {
    it('renders .tile as an anchor with href when navigable is set', async () => {
      const el = await mount('<ship-list-tile navigable="/agents/foo">Foo</ship-list-tile>')
      const tile = el.shadowRoot!.querySelector('.tile') as HTMLAnchorElement
      expect(tile).toBeTruthy()
      expect(tile.tagName).toBe('A')
      expect(tile.getAttribute('href')).toBe('/agents/foo')
      expect(tile.classList.contains('tile')).toBe(true)
    })

    it('sets role=link when navigable is set', async () => {
      const el = await mount('<ship-list-tile navigable="/x">X</ship-list-tile>')
      expect(el.getAttribute('role')).toBe('link')
    })

    it('link role wins over button when both navigable and interactive are set', async () => {
      const el = await mount('<ship-list-tile navigable="/x" interactive>X</ship-list-tile>')
      expect(el.getAttribute('role')).toBe('link')
    })

    it('passes target and rel through to the anchor', async () => {
      const el = await mount(
        '<ship-list-tile navigable="/x" target="_blank" rel="noopener noreferrer">X</ship-list-tile>',
      )
      const a = el.shadowRoot!.querySelector('a.tile') as HTMLAnchorElement
      expect(a.getAttribute('target')).toBe('_blank')
      expect(a.getAttribute('rel')).toBe('noopener noreferrer')
    })

    it('does not fire ship-select on plain click when only navigable (no interactive)', async () => {
      const el = await mount('<ship-list-tile navigable="/x">X</ship-list-tile>')
      let fired = 0
      el.addEventListener('ship-select', () => fired++)
      const a = el.shadowRoot!.querySelector('a.tile') as HTMLAnchorElement
      a.click()
      expect(fired).toBe(0)
    })

    it('fires ship-select when navigable + interactive and click is plain', async () => {
      const el = await mount('<ship-list-tile navigable="/x" interactive>X</ship-list-tile>')
      let fired = 0
      el.addEventListener('ship-select', () => fired++)
      const a = el.shadowRoot!.querySelector('a.tile') as HTMLAnchorElement
      a.click()
      expect(fired).toBe(1)
    })

    it('preventDefault on ship-select blocks anchor navigation', async () => {
      const el = await mount('<ship-list-tile navigable="/x" interactive>X</ship-list-tile>')
      el.addEventListener('ship-select', (e) => e.preventDefault())
      const a = el.shadowRoot!.querySelector('a.tile') as HTMLAnchorElement
      const click = new MouseEvent('click', { bubbles: true, cancelable: true })
      a.dispatchEvent(click)
      expect(click.defaultPrevented).toBe(true)
    })

    it('without preventDefault, anchor navigation is not intercepted', async () => {
      const el = await mount('<ship-list-tile navigable="/x" interactive>X</ship-list-tile>')
      const a = el.shadowRoot!.querySelector('a.tile') as HTMLAnchorElement
      const click = new MouseEvent('click', { bubbles: true, cancelable: true })
      a.dispatchEvent(click)
      expect(click.defaultPrevented).toBe(false)
    })

    it('Enter on navigable relies on native anchor behavior (no duplicate select)', async () => {
      const el = await mount('<ship-list-tile navigable="/x" interactive>X</ship-list-tile>')
      let fired = 0
      el.addEventListener('ship-select', () => fired++)
      const a = el.shadowRoot!.querySelector('a.tile') as HTMLAnchorElement
      a.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }))
      // Anchor handles Enter natively — we skip dispatching ship-select
      // to avoid double navigation when a consumer doesn't preventDefault.
      expect(fired).toBe(0)
    })

    it('Space on navigable + interactive still fires ship-select', async () => {
      const el = await mount('<ship-list-tile navigable="/x" interactive>X</ship-list-tile>')
      let fired = 0
      el.addEventListener('ship-select', () => fired++)
      const a = el.shadowRoot!.querySelector('a.tile') as HTMLAnchorElement
      a.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true, cancelable: true }))
      expect(fired).toBe(1)
    })
  })

  describe('inter-sibling dividers (adopted stylesheet)', () => {
    it('installs adopted stylesheet on first connect', async () => {
      const el = await mount('<ship-list-tile variant="row">A</ship-list-tile>')
      const sheets = document.adoptedStyleSheets
      const hasDividerSheet = sheets.some(s => {
        try { return [...s.cssRules].some(r => r.cssText.includes('ship-list-tile')) } catch { return false }
      })
      // If constructable stylesheets not supported we check the fallback <style>
      const hasFallback = !!document.querySelector('style[data-ship-list-tile-dividers]')
      expect(hasDividerSheet || hasFallback).toBe(true)
      // cleanup
      el.remove()
    })
  })
})
