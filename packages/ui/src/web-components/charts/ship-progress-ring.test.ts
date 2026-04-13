import { afterEach, describe, expect, it } from 'vitest'
import './ship-progress-ring'
import type { ShipProgressRing } from './ship-progress-ring'
import { cleanupTestDom } from '../util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipProgressRing
  await el.updateComplete
  return el
}

describe('ship-progress-ring', () => {
  afterEach(cleanupTestDom)

  it('renders progressbar role + aria-valuenow', async () => {
    const el = await mount('<ship-progress-ring value="25"></ship-progress-ring>')
    const svg = el.shadowRoot!.querySelector('svg')!
    expect(svg.getAttribute('role')).toBe('progressbar')
    expect(svg.getAttribute('aria-valuenow')).toBe('25')
  })

  it('clamps value to 0..100', async () => {
    const el = await mount('<ship-progress-ring value="250"></ship-progress-ring>')
    const svg = el.shadowRoot!.querySelector('svg')!
    expect(svg.getAttribute('aria-valuenow')).toBe('100')
  })

  it('reacts to value changes', async () => {
    const el = await mount('<ship-progress-ring value="10"></ship-progress-ring>')
    const before = el.shadowRoot!.querySelector('.bar')!.getAttribute('stroke-dashoffset')
    el.value = 80
    await el.updateComplete
    const after = el.shadowRoot!.querySelector('.bar')!.getAttribute('stroke-dashoffset')
    expect(after).not.toEqual(before)
  })
})
