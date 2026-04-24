import { afterEach, describe, expect, it } from 'vitest'
import './ship-sparkline'
import type { ShipSparkline } from './ship-sparkline'
import { cleanupTestDom } from '../core/util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipSparkline
  await el.updateComplete
  return el
}

describe('ship-sparkline', () => {
  afterEach(cleanupTestDom)

  it('renders a path from comma-separated data', async () => {
    const el = await mount('<ship-sparkline data="1,2,3,4,5" tone="info"></ship-sparkline>')
    const paths = el.shadowRoot!.querySelectorAll('path')
    expect(paths.length).toBe(1)
    expect(paths[0].getAttribute('d')).toMatch(/^M/)
  })

  it('reacts to attribute change', async () => {
    const el = await mount('<ship-sparkline data="1,2,3"></ship-sparkline>')
    const before = el.shadowRoot!.querySelector('path')!.getAttribute('d')
    el.setAttribute('data', '5,4,3,2,1')
    await el.updateComplete
    const after = el.shadowRoot!.querySelector('path')!.getAttribute('d')
    expect(after).not.toEqual(before)
  })

  it('renders area fill when fill attr set', async () => {
    const el = await mount('<ship-sparkline data="1,2,3,4" fill></ship-sparkline>')
    expect(el.shadowRoot!.querySelectorAll('path').length).toBe(2)
  })

  it('renders empty svg for <2 points (no crash)', async () => {
    const el = await mount('<ship-sparkline data="1"></ship-sparkline>')
    expect(el.shadowRoot!.querySelector('svg')).toBeTruthy()
    expect(el.shadowRoot!.querySelectorAll('path').length).toBe(0)
  })
})
