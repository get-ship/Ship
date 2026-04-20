import { afterEach, describe, expect, it } from 'vitest'
import './ship-link'
import type { ShipLink } from './ship-link'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipLink
  await el.updateComplete
  return el
}

describe('ship-link', () => {
  afterEach(cleanupTestDom)

  it('renders anchor with href and slotted label', async () => {
    const el = await mount('<ship-link href="/home">Home</ship-link>')
    const a = el.shadowRoot!.querySelector('a')!
    expect(a.getAttribute('href')).toBe('/home')
    expect(el.textContent).toBe('Home')
  })

  it('external adds target=_blank, rel=noopener, and icon', async () => {
    const el = await mount('<ship-link href="https://x.com" external>X</ship-link>')
    const a = el.shadowRoot!.querySelector('a')!
    expect(a.getAttribute('target')).toBe('_blank')
    expect(a.getAttribute('rel')).toContain('noopener')
    expect(el.shadowRoot!.querySelector('.ext')).not.toBeNull()
  })

  it('no external icon by default', async () => {
    const el = await mount('<ship-link href="/a">A</ship-link>')
    expect(el.shadowRoot!.querySelector('.ext')).toBeNull()
  })

  it('reflects tone attribute', async () => {
    const el = await mount('<ship-link tone="muted" href="/a">A</ship-link>')
    expect(el.getAttribute('tone')).toBe('muted')
  })
})
