import { afterEach, describe, expect, it } from 'vitest'
import './ship-progress'
import type { ShipProgress } from './ship-progress'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipProgress
  await el.updateComplete
  return el
}

describe('ship-progress', () => {
  afterEach(cleanupTestDom)

  it('sets aria role + values when determinate', async () => {
    const el = await mount('<ship-progress value="40"></ship-progress>')
    const track = el.shadowRoot!.querySelector('.track')!
    expect(track.getAttribute('role')).toBe('progressbar')
    expect(track.getAttribute('aria-valuenow')).toBe('40')
    const bar = track.querySelector('.bar') as HTMLElement
    expect(bar.style.width).toBe('40%')
  })

  it('indeterminate when value omitted', async () => {
    const el = await mount('<ship-progress></ship-progress>')
    const bar = el.shadowRoot!.querySelector('.bar') as HTMLElement
    expect(bar.style.width).toBe('')
  })
})
