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
})
