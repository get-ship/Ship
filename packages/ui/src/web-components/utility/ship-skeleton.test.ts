import { afterEach, describe, expect, it } from 'vitest'
import './ship-skeleton'
import type { ShipSkeleton } from './ship-skeleton'
import { cleanupTestDom } from '../util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipSkeleton
  await el.updateComplete
  return el
}

describe('ship-skeleton', () => {
  afterEach(cleanupTestDom)

  it('renders one bar by default', async () => {
    const el = await mount('<ship-skeleton></ship-skeleton>')
    expect(el.shadowRoot!.querySelectorAll('.bar').length).toBe(1)
  })

  it('renders N lines for text variant', async () => {
    const el = await mount('<ship-skeleton variant="text" lines="4"></ship-skeleton>')
    expect(el.shadowRoot!.querySelectorAll('.bar').length).toBe(4)
  })

  it('applies width and height', async () => {
    const el = await mount('<ship-skeleton variant="rect" width="120px" height="60px"></ship-skeleton>')
    const bar = el.shadowRoot!.querySelector('.bar') as HTMLElement
    expect(bar.style.width).toBe('120px')
    expect(bar.style.height).toBe('60px')
  })

  it('reflects variant attribute', async () => {
    const el = await mount('<ship-skeleton variant="circle" width="40px" height="40px"></ship-skeleton>')
    expect(el.getAttribute('variant')).toBe('circle')
  })
})
